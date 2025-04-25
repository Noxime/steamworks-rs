use std::path::Path;

pub use sys::ScreenshotHandle;

use super::*;

/// Access to the steam screenshots interface
pub struct Screenshots {
    pub(crate) screenshots: *mut sys::ISteamScreenshots,
    pub(crate) _inner: Arc<Inner>,
}

impl Screenshots {
    /// Toggles whether the overlay handles screenshots when the user presses the screenshot hotkey, or if the game handles them.
    ///
    /// Hooking is disabled by default, and only ever enabled if you do so with this function.
    ///
    /// If the hooking is enabled, then the [`ScreenshotRequested`] callback will be sent if the user presses the hotkey or when [`Self::trigger_screenshot`] is called,
    /// and then the game is expected to call `WriteScreenshot` or [`Self::add_screenshot_to_library`] in response.
    ///
    /// You can check if hooking is enabled with [`Self::is_screenshots_hooked`].
    pub fn hook_screenshots(&self, hook: bool) {
        unsafe {
            sys::SteamAPI_ISteamScreenshots_HookScreenshots(self.screenshots, hook);
        }
    }

    /// Checks if the app is hooking screenshots, or if the Steam Overlay is handling them.
    ///
    /// This can be toggled with [`Self::hook_screenshots`].
    ///
    /// Returns
    /// - `true` if the game is hooking screenshots and is expected to handle them; otherwise, `false`.
    pub fn is_screenshots_hooked(&self) -> bool {
        unsafe { sys::SteamAPI_ISteamScreenshots_IsScreenshotsHooked(self.screenshots) }
    }

    /// Either causes the Steam Overlay to take a screenshot, or tells your screenshot manager that a screenshot needs to be taken.
    /// Depending on the value of [`Self::is_screenshots_hooked`].
    ///
    /// - Triggers a [`ScreenshotRequested`] callback.
    /// - Triggers a [`ScreenshotReady`] callback.
    /// - Only causes [`ScreenshotRequested`] if hooking has been enabled with [`Self::hook_screenshots`].
    /// - Otherwise [`ScreenshotReady`] will be called when the screenshot has been saved and added to the library.
    pub fn trigger_screenshot(&self) {
        unsafe {
            sys::SteamAPI_ISteamScreenshots_TriggerScreenshot(self.screenshots);
        }
    }

    /// Adds a screenshot to the user's Steam screenshot library from disk.
    ///
    /// Triggers a [`ScreenshotReady`] callback.
    /// The handle to this screenshot that is valid for the duration of the game process and can be used to apply tags.
    ///
    /// This call is asynchronous, a [`ScreenshotReady`] callback will be sent when the screenshot has finished writing to disk.
    pub fn add_screenshot_to_library(
        &self,
        filename: &Path,
        thumbnail_filename: Option<&Path>,
        width: i32,
        height: i32,
    ) -> Result<ScreenshotHandle, ScreenshotLibraryAddError> {
        let filename =
            path_to_absolute_cstring(filename).ok_or(ScreenshotLibraryAddError::InvalidPath)?;

        let thumbnail_filename = if let Some(thumbnail_filename) = thumbnail_filename {
            Some(
                path_to_absolute_cstring(thumbnail_filename)
                    .ok_or(ScreenshotLibraryAddError::InvalidPath)?,
            )
        } else {
            None
        };

        unsafe {
            let handle = sys::SteamAPI_ISteamScreenshots_AddScreenshotToLibrary(
                self.screenshots,
                filename.as_ptr(),
                thumbnail_filename.map_or(std::ptr::null(), |s| s.as_ptr()),
                width,
                height,
            );

            if handle != sys::INVALID_SCREENSHOT_HANDLE {
                Ok(handle)
            } else {
                Err(ScreenshotLibraryAddError::SavingFailed)
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum ScreenshotLibraryAddError {
    /// Steam failed to save the file for an unspecified reason.
    #[error("The screenshot file could not be saved")]
    SavingFailed,
    /// One of the paths provided was invalid.
    #[error("Invalid path")]
    InvalidPath,
}

/// A screenshot has been requested by the user from the Steam screenshot hotkey.
/// This will only be called if [`Screenshots::hook_screenshots`] has been enabled, in which case Steam will not take the screenshot itself.
#[derive(Clone, Debug)]
pub struct ScreenshotRequested;

unsafe impl Callback for ScreenshotRequested {
    const ID: i32 = sys::ScreenshotRequested_t__bindgen_ty_1::k_iCallback as _;

    unsafe fn from_raw(_: *mut c_void) -> Self {
        Self
    }
}

#[derive(Clone, Debug, Error)]
pub enum ScreenshotReadyError {
    /// The screenshot could not be loaded or parsed.
    #[error("The screenshot could not be loaded or parsed")]
    Fail,
    /// The screenshot could not be saved to the disk.
    #[error("The screenshot could not be saved to the disk")]
    IoFailure,
}

/// A screenshot successfully written or otherwise added to the library and can now be tagged.
#[derive(Clone, Debug)]
pub struct ScreenshotReady {
    /// The screenshot handle that has been written.
    pub local_handle: Result<ScreenshotHandle, ScreenshotReadyError>,
}

unsafe impl Callback for ScreenshotReady {
    const ID: i32 = sys::ScreenshotReady_t__bindgen_ty_1::k_iCallback as _;

    unsafe fn from_raw(raw: *mut c_void) -> Self {
        let status = *(raw as *mut sys::ScreenshotReady_t);
        let local_handle = match status.m_eResult {
            sys::EResult::k_EResultOK => Ok(status.m_hLocal),
            sys::EResult::k_EResultIOFailure => Err(ScreenshotReadyError::Fail),
            _ => Err(ScreenshotReadyError::Fail),
        };

        Self { local_handle }
    }
}

fn path_to_absolute_cstring(filename: &Path) -> Option<CString> {
    let filename = filename.canonicalize().ok()?;
    Some(CString::new(filename.to_str()?).unwrap())
}
