use super::*;

/// An id for a steam app/game
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AppId(pub u32);
impl From<u32> for AppId {
    fn from(id: u32) -> Self {
        AppId(id)
    }
}

/// Access to the steam apps interface
pub struct Apps<Manager> {
    pub(crate) apps: *mut sys::ISteamApps,
    pub(crate) _inner: Arc<Inner<Manager>>,
}

impl<Manager> Apps<Manager> {
    /// Returns whether the user currently has the app with the given
    /// ID currently installed.
    ///
    /// This does not mean the user owns the game.
    pub fn is_app_installed(&self, app_id: AppId) -> bool {
        unsafe { sys::SteamAPI_ISteamApps_BIsAppInstalled(self.apps, app_id.0) }
    }

    /// Returns whether the user owns the specific dlc and has it
    /// installed.
    pub fn is_dlc_installed(&self, app_id: AppId) -> bool {
        unsafe { sys::SteamAPI_ISteamApps_BIsDlcInstalled(self.apps, app_id.0) }
    }

    /// Returns whether the user is subscribed to the app with the given
    /// ID.
    ///
    /// This should only be used to check ownership of a game related to
    /// yours (e.g. demo).
    pub fn is_subscribed_app(&self, app_id: AppId) -> bool {
        unsafe { sys::SteamAPI_ISteamApps_BIsSubscribedApp(self.apps, app_id.0) }
    }

    /// Returns whether the user is subscribed via a free weekend
    pub fn is_subscribed_from_free_weekend(&self) -> bool {
        unsafe { sys::SteamAPI_ISteamApps_BIsSubscribedFromFreeWeekend(self.apps) }
    }

    /// Returns whether the user has a VAC ban on their account.
    pub fn is_vac_banned(&self) -> bool {
        unsafe { sys::SteamAPI_ISteamApps_BIsVACBanned(self.apps) }
    }

    /// Returns whether the license for the current app ID
    /// is for cyber cafes.
    pub fn is_cybercafe(&self) -> bool {
        unsafe { sys::SteamAPI_ISteamApps_BIsCybercafe(self.apps) }
    }

    /// Returns whether the license for the current app ID
    /// provides low violence depots.
    pub fn is_low_violence(&self) -> bool {
        unsafe { sys::SteamAPI_ISteamApps_BIsLowViolence(self.apps) }
    }

    /// Returns whether the user is subscribed to the current app ID
    pub fn is_subscribed(&self) -> bool {
        unsafe { sys::SteamAPI_ISteamApps_BIsSubscribed(self.apps) }
    }

    /// Returns the build id of this app.
    pub fn app_build_id(&self) -> i32 {
        unsafe { sys::SteamAPI_ISteamApps_GetAppBuildId(self.apps) as i32 }
    }

    /// Returns the installation folder of the app with the given ID.
    ///
    /// This works even if the app isn't installed, returning where it
    /// would be installed in the default location.
    pub fn app_install_dir(&self, app_id: AppId) -> String {
        unsafe {
            let mut buffer = vec![0; 2048];
            sys::SteamAPI_ISteamApps_GetAppInstallDir(
                self.apps,
                app_id.0,
                buffer.as_mut_ptr(),
                buffer.len() as u32,
            );
            let path = CStr::from_ptr(buffer.as_ptr());
            path.to_string_lossy().into_owned()
        }
    }

    /// Returns the steam id of the original owner of the app.
    ///
    /// Differs from the current user if the app is borrowed.
    pub fn app_owner(&self) -> SteamId {
        unsafe { SteamId(sys::SteamAPI_ISteamApps_GetAppOwner(self.apps)) }
    }

    /// Returns a list of languages that the current app supports.
    pub fn available_game_languages(&self) -> Vec<String> {
        unsafe {
            let langs = sys::SteamAPI_ISteamApps_GetAvailableGameLanguages(self.apps);
            let langs = CStr::from_ptr(langs);
            let langs = langs.to_string_lossy();
            langs.split(',').map(|v| v.to_owned()).collect()
        }
    }

    /// Returns the language the user has set for the current game.
    ///
    /// If the language hasn't been set this returns the language
    /// used for the steam UI.
    pub fn current_game_language(&self) -> String {
        unsafe {
            let lang = sys::SteamAPI_ISteamApps_GetCurrentGameLanguage(self.apps);
            let lang = CStr::from_ptr(lang);
            lang.to_string_lossy().into_owned()
        }
    }

    /// Returns the current beta name if any.
    ///
    /// If the user isn't playing on a beta branch then this
    /// returns `None`
    pub fn current_beta_name(&self) -> Option<String> {
        unsafe {
            let mut buffer = vec![0; 256];
            if sys::SteamAPI_ISteamApps_GetCurrentBetaName(
                self.apps,
                buffer.as_mut_ptr(),
                buffer.len() as _,
            ) {
                let path = CStr::from_ptr(buffer.as_ptr());
                Some(path.to_string_lossy().into_owned())
            } else {
                None
            }
        }
    }
}
