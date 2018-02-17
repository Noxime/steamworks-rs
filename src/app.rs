use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppId(pub u32);

pub struct Apps {
    pub(crate) apps: *mut sys::ISteamApps,
    pub(crate) _client: Arc<ClientInner>,
}

impl Apps {

    pub fn is_app_installed(&self, app_id: AppId) -> bool {
        unsafe {
            sys::SteamAPI_ISteamApps_BIsAppInstalled(self.apps, app_id.0) != 0
        }
    }

    pub fn is_dlc_installed(&self, app_id: AppId) -> bool {
        unsafe {
            sys::SteamAPI_ISteamApps_BIsDlcInstalled(self.apps, app_id.0) != 0
        }
    }

    pub fn is_subscribed_app(&self, app_id: AppId) -> bool {
        unsafe {
            sys::SteamAPI_ISteamApps_BIsSubscribedApp(self.apps, app_id.0) != 0
        }
    }

    pub fn is_subscribed_from_free_weekend(&self) -> bool {
        unsafe {
            sys::SteamAPI_ISteamApps_BIsSubscribedFromFreeWeekend(self.apps) != 0
        }
    }

    pub fn is_vac_banned(&self) -> bool {
        unsafe {
            sys::SteamAPI_ISteamApps_BIsVACBanned(self.apps) != 0
        }
    }

    pub fn is_cybercafe(&self) -> bool {
        unsafe {
            sys::SteamAPI_ISteamApps_BIsCybercafe(self.apps) != 0
        }
    }

    pub fn is_low_violence(&self) -> bool {
        unsafe {
            sys::SteamAPI_ISteamApps_BIsLowViolence(self.apps) != 0
        }
    }

    pub fn is_subscribed(&self) -> bool {
        unsafe {
            sys::SteamAPI_ISteamApps_BIsSubscribed(self.apps) != 0
        }
    }

    pub fn app_build_id(&self) -> i32 {
        unsafe {
            sys::SteamAPI_ISteamApps_GetAppBuildId(self.apps) as i32
        }
    }

    pub fn app_install_dir(&self, app_id: AppId) -> String {
        unsafe {
            let buffer = vec![0; 2048];
            sys::SteamAPI_ISteamApps_GetAppInstallDir(self.apps, app_id.0, buffer.as_ptr(), buffer.len() as u32);
            let path = CStr::from_ptr(buffer.as_ptr());
            path.to_string_lossy().into_owned()
        }
    }

    pub fn app_owner(&self) -> SteamId {
        unsafe {
            SteamId(sys::SteamAPI_ISteamApps_GetAppOwner(self.apps))
        }
    }

    pub fn available_game_languages(&self) -> Vec<String> {
        unsafe {
            let langs = sys::SteamAPI_ISteamApps_GetAvailableGameLanguages(self.apps);
            let langs = CStr::from_ptr(langs);
            let langs = langs.to_string_lossy();
            langs.split(',')
                .map(|v| v.to_owned())
                .collect()
        }
    }

    pub fn current_game_language(&self) -> Cow<str> {
        unsafe {
            let lang = sys::SteamAPI_ISteamApps_GetCurrentGameLanguage(self.apps);
            let lang = CStr::from_ptr(lang);
            lang.to_string_lossy()
        }
    }

    pub fn current_beta_name(&self) -> Option<String> {
        unsafe {
            let buffer = vec![0; 256];
            if sys::SteamAPI_ISteamApps_GetCurrentBetaName(self.apps, buffer.as_ptr(), buffer.len() as _) != 0 {
                let path = CStr::from_ptr(buffer.as_ptr());
                Some(path.to_string_lossy().into_owned())
            } else {
                None
            }
        }
    }
}