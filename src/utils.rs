
use super::*;

/// Access to the steam utils interface
pub struct Utils {
    pub(crate) utils: *mut sys::ISteamUtils,
    pub(crate) _client: Arc<ClientInner>,
}

pub enum NotificationPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Utils {
    /// Returns the app ID of the current process
    pub fn app_id(&self) -> AppId {
        unsafe {
            AppId(sys::SteamAPI_ISteamUtils_GetAppID(self.utils))
        }
    }

    /// Returns the language the steam client is currently
    /// running in.
    ///
    /// Generally you want `Apps::current_game_language` instead of this
    pub fn ui_language(&self) -> Cow<str> {
        unsafe {
            let lang = sys::SteamAPI_ISteamUtils_GetSteamUILanguage(self.utils);
            let lang = CStr::from_ptr(lang);
            lang.to_string_lossy()
        }
    }

    /// Sets the position on the screen where popups from the steam overlay
    /// should appear and display themselves in.
    pub fn set_overlay_notification_position(&self, position: NotificationPosition) {
        unsafe {
            let position = match position {
                NotificationPosition::TopLeft => sys::NotificationPosition::TopLeft,
                NotificationPosition::TopRight => sys::NotificationPosition::TopRight,
                NotificationPosition::BottomLeft => sys::NotificationPosition::BottomLeft,
                NotificationPosition::BottomRight => sys::NotificationPosition::BottomRight,
            };
            sys::SteamAPI_ISteamUtils_SetOverlayNotificationPosition(self.utils, position);
        }
    }
}