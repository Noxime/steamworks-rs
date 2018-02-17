
use super::*;

/// Access to the steam utils interface
pub struct Utils {
    pub(crate) utils: *mut sys::ISteamUtils,
    pub(crate) _client: Arc<ClientInner>,
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
}