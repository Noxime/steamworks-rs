use std::path::Path;

pub use sys::ScreenshotHandle;

use super::*;

/// Access to the steam screenshots interface
pub struct Screenshots<Manager> {
    pub(crate) screenshots: *mut sys::ISteamScreenshots,
    pub(crate) _inner: Arc<Inner<Manager>>,
}

impl<Manager> Screenshots<Manager> {
    /// Toggles whether the overlay handles screenshots when the user presses the screenshot hotkey, or if the game handles them.
    ///
    /// Hooking is disabled by default, and only ever enabled if you do so with this function.
    ///
    /// If the hooking is enabled, then the [`ScreenshotRequested`] callback will be sent if the user presses the hotkey or when [`trigger_screenshot`] is called,
    /// and then the game is expected to call `write_screenshot` or [`add_screenshot_to_library`] in response.
    ///
    /// You can check if hooking is enabled with `is_screenshots_hooked``.
    pub fn hook_screenshots(&self, hook: bool) {
        unsafe {
            sys::SteamAPI_ISteamScreenshots_HookScreenshots(self.screenshots, hook);
        }
    }

    /// Checks if the app is hooking screenshots, or if the Steam Overlay is handling them.
    ///
    /// This can be toggled with [`hook_screenshots`].
    ///
    /// Returns
    /// - `true` if the game is hooking screenshots and is expected to handle them; otherwise, `false`.
    pub fn is_screenshots_hooked(&self) -> bool {
        unsafe { sys::SteamAPI_ISteamScreenshots_IsScreenshotsHooked(self.screenshots) }
    }

    /// Either causes the Steam Overlay to take a screenshot, or tells your screenshot manager that a screenshot needs to be taken.
    /// Depending on the value of IsScreenshotsHooked.
    ///
    /// Triggers a [`ScreenshotRequested`] callback.
    /// Triggers a [`ScreenshotReady`] callback.
    /// Only causes [`ScreenshotRequested`] if hooking has been enabled with [`hook_screenshots`].
    /// Otherwise [`ScreenshotReady`] will be called when the screenshot has been saved and added to the library.
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
    /// Steam failed to save the file for an unspecified reason
    #[error("The screenshot file could not be saved")]
    SavingFailed,
    /// One of the paths provided was invalid
    #[error("Invalid path")]
    InvalidPath,
}

#[derive(Clone, Debug)]
pub struct ScreenshotRequested;

unsafe impl Callback for ScreenshotRequested {
    const ID: i32 = sys::ScreenshotRequested_t__bindgen_ty_1::k_iCallback as _;
    const SIZE: i32 = std::mem::size_of::<sys::ScreenshotRequested_t>() as _;

    unsafe fn from_raw(_: *mut c_void) -> Self {
        Self
    }
}

#[derive(Clone, Debug, Error)]
pub enum ScreenshotReadyError {
    #[error("The screenshot could not be loaded or parsed")]
    Fail,
    #[error("The screenshot could not be saved to the disk")]
    IoFailure,
}

#[derive(Clone, Debug)]
pub struct ScreenshotReady {
    pub handle: Result<ScreenshotHandle, ScreenshotReadyError>,
}

unsafe impl Callback for ScreenshotReady {
    const ID: i32 = sys::ScreenshotReady_t__bindgen_ty_1::k_iCallback as _;
    const SIZE: i32 = std::mem::size_of::<sys::ScreenshotReady_t>() as _;

    unsafe fn from_raw(raw: *mut c_void) -> Self {
        let status = *(raw as *mut sys::ScreenshotReady_t);
        let handle = match status.m_eResult {
            sys::EResult::k_EResultOK => Ok(status.m_hLocal),
            sys::EResult::k_EResultIOFailure => Err(ScreenshotReadyError::Fail),
            _ => Err(ScreenshotReadyError::Fail),
        };

        Self { handle }
    }
}

fn path_to_absolute_cstring(filename: &Path) -> Option<CString> {
    let filename = filename.canonicalize().ok()?;
    Some(CString::new(filename.to_str()?).unwrap())
}
