
use super::*;

use libc::c_char;
use std::ffi::CStr;
use std::panic;
use std::process::abort;
use std::sync::RwLock;

/// Access to the steam utils interface
pub struct Utils<Manager> {
    pub(crate) utils: *mut sys::ISteamUtils,
    pub(crate) _inner: Arc<Inner<Manager>>,
}

pub enum NotificationPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

lazy_static!{
    /// Global rust warning callback
    static ref WARNING_CALLBACK: RwLock<Option<Box<Fn(i32, &CStr) + Send + Sync>>> = RwLock::new(None);
}

/// C function to pass as the real callback, which forwards to the `WARNING_CALLBACK` if any
unsafe extern "cdecl" fn c_warning_callback(level: i32, msg: *const c_char) {
    let lock = WARNING_CALLBACK.read().expect("warning func lock poisoned");
    let cb = match lock.as_ref() {
        Some(cb) => cb,
        None => { return; }
    };

    let s = CStr::from_ptr(msg);

    let res = panic::catch_unwind(panic::AssertUnwindSafe(||
        cb(level, s)
    ));
    if let Err(err) = res {
        if let Some(err) = err.downcast_ref::<&str>() {
            println!("Steam warning callback paniced: {}", err);
        } else if let Some(err) = err.downcast_ref::<String>() {
            println!("Steam warning callback paniced: {}", err);
        } else {
            println!("Steam warning callback paniced");
        }
        abort();
    }
}

impl <Manager> Utils<Manager> {
    /// Returns the app ID of the current process
    pub fn app_id(&self) -> AppId {
        unsafe {
            AppId(sys::SteamAPI_ISteamUtils_GetAppID(self.utils).0)
        }
    }

    /// Returns the language the steam client is currently
    /// running in.
    ///
    /// Generally you want `Apps::current_game_language` instead of this
    pub fn ui_language(&self) -> String {
        unsafe {
            let lang = sys::SteamAPI_ISteamUtils_GetSteamUILanguage(self.utils);
            let lang = CStr::from_ptr(lang);
            lang.to_string_lossy().into_owned()
        }
    }

    /// Sets the position on the screen where popups from the steam overlay
    /// should appear and display themselves in.
    pub fn set_overlay_notification_position(&self, position: NotificationPosition) {
        unsafe {
            let position = match position {
                NotificationPosition::TopLeft => sys::ENotificationPosition::EPositionTopLeft,
                NotificationPosition::TopRight => sys::ENotificationPosition::EPositionTopRight,
                NotificationPosition::BottomLeft => sys::ENotificationPosition::EPositionBottomLeft,
                NotificationPosition::BottomRight => sys::ENotificationPosition::EPositionBottomRight,
            };
            sys::SteamAPI_ISteamUtils_SetOverlayNotificationPosition(self.utils, position);
        }
    }

    /// Sets the Steam warning callback, which is called to emit warning messages.
    ///
    /// The passed-in function takes two arguments: a severity level (0 = info, 1 = warning) and
    /// the message itself.
    ///
    /// See [Steamwork's debugging page](https://partner.steamgames.com/doc/sdk/api/debugging) for more info.
    pub fn set_warning_callback<F>(&self, cb: F)
        where F: Fn(i32, &CStr) + Send + Sync + 'static
    {
        let mut lock = WARNING_CALLBACK.write().expect("warning func lock poisoned");
        *lock = Some(Box::new(cb));
        unsafe {
            sys::SteamAPI_ISteamUtils_SetWarningMessageHook(self.utils, c_warning_callback);
        }
    }
}
