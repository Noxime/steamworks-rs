
use super::*;

pub struct Utils {
    pub(crate) utils: *mut sys::ISteamUtils,
    pub(crate) _client: Rc<ClientInner>,
}

impl Utils {
    pub fn app_id(&self) -> AppId {
        unsafe {
            AppId(sys::SteamAPI_ISteamUtils_GetAppID(self.utils))
        }
    }

    pub fn ui_language(&self) -> Cow<str> {
        unsafe {
            let lang = sys::SteamAPI_ISteamUtils_GetSteamUILanguage(self.utils);
            let lang = CStr::from_ptr(lang);
            lang.to_string_lossy()
        }
    }
}