use super::*;
use std::ptr::NonNull;

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
    pub(crate) apps: NonNull<sys::ISteamApps>,
    pub(crate) _inner: Arc<Inner<Manager>>,
}

impl<Manager> Apps<Manager> {
    /// Returns whether the user currently has the app with the given
    /// ID currently installed.
    ///
    /// This does not mean the user owns the game.
    pub fn is_app_installed(&self, app_id: AppId) -> bool {
        unsafe { sys::SteamAPI_ISteamApps_BIsAppInstalled(self.apps.as_ptr(), app_id.0) }
    }

    /// Returns whether the user owns the specific dlc and has it
    /// installed.
    pub fn is_dlc_installed(&self, app_id: AppId) -> bool {
        unsafe { sys::SteamAPI_ISteamApps_BIsDlcInstalled(self.apps.as_ptr(), app_id.0) }
    }

    /// Returns whether the user is subscribed to the app with the given
    /// ID.
    ///
    /// This should only be used to check ownership of a game related to
    /// yours (e.g. demo).
    pub fn is_subscribed_app(&self, app_id: AppId) -> bool {
        unsafe { sys::SteamAPI_ISteamApps_BIsSubscribedApp(self.apps.as_ptr(), app_id.0) }
    }

    /// Returns whether the user is subscribed via a free weekend
    pub fn is_subscribed_from_free_weekend(&self) -> bool {
        unsafe { sys::SteamAPI_ISteamApps_BIsSubscribedFromFreeWeekend(self.apps.as_ptr()) }
    }

    /// Returns whether the user has a VAC ban on their account.
    pub fn is_vac_banned(&self) -> bool {
        unsafe { sys::SteamAPI_ISteamApps_BIsVACBanned(self.apps.as_ptr()) }
    }

    /// Returns whether the license for the current app ID
    /// is for cyber cafes.
    pub fn is_cybercafe(&self) -> bool {
        unsafe { sys::SteamAPI_ISteamApps_BIsCybercafe(self.apps.as_ptr()) }
    }

    /// Returns whether the license for the current app ID
    /// provides low violence depots.
    pub fn is_low_violence(&self) -> bool {
        unsafe { sys::SteamAPI_ISteamApps_BIsLowViolence(self.apps.as_ptr()) }
    }

    /// Returns whether the user is subscribed to the current app ID
    pub fn is_subscribed(&self) -> bool {
        unsafe { sys::SteamAPI_ISteamApps_BIsSubscribed(self.apps.as_ptr()) }
    }

    /// Returns the build id of this app.
    pub fn app_build_id(&self) -> i32 {
        unsafe { sys::SteamAPI_ISteamApps_GetAppBuildId(self.apps.as_ptr()) as i32 }
    }

    /// Returns the installation folder of the app with the given ID.
    ///
    /// This works even if the app isn't installed, returning where it
    /// would be installed in the default location.
    pub fn app_install_dir(&self, app_id: AppId) -> String {
        let mut buffer = [0; 2048];
        unsafe {
            sys::SteamAPI_ISteamApps_GetAppInstallDir(
                self.apps.as_ptr(),
                app_id.0,
                buffer.as_mut_ptr(),
                buffer.len() as u32,
            );
            lossy_string_from_cstr(buffer.as_ptr())
        }
    }

    /// Returns the steam id of the original owner of the app.
    ///
    /// Differs from the current user if the app is borrowed.
    pub fn app_owner(&self) -> SteamId {
        unsafe { SteamId(sys::SteamAPI_ISteamApps_GetAppOwner(self.apps.as_ptr())) }
    }

    /// Returns a list of languages that the current app supports.
    pub fn available_game_languages(&self) -> Vec<String> {
        unsafe {
            let langs = sys::SteamAPI_ISteamApps_GetAvailableGameLanguages(self.apps.as_ptr());
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
            let lang = sys::SteamAPI_ISteamApps_GetCurrentGameLanguage(self.apps.as_ptr());
            lossy_string_from_cstr(lang)
        }
    }

    /// Returns the current beta name if any.
    ///
    /// If the user isn't playing on a beta branch then this
    /// returns `None`
    pub fn current_beta_name(&self) -> Option<String> {
        let mut buffer = [0; 256];
        unsafe {
            sys::SteamAPI_ISteamApps_GetCurrentBetaName(
                self.apps.as_ptr(),
                buffer.as_mut_ptr(),
                buffer.len() as i32,
            )
            .then(|| lossy_string_from_cstr(buffer.as_ptr()))
        }
    }

    /// Returns the command line if the game was launched via Steam URL
    ///
    /// If the game was not launched through Steam URL, this returns an empty string.
    ///
    /// See [Steam API](https://partner.steamgames.com/doc/api/ISteamApps#GetLaunchCommandLine)
    pub fn launch_command_line(&self) -> String {
        let mut buffer = [0; 256];
        unsafe {
            sys::SteamAPI_ISteamApps_GetLaunchCommandLine(
                self.apps.as_ptr(),
                buffer.as_mut_ptr(),
                buffer.len() as _,
            );
            lossy_string_from_cstr(buffer.as_ptr())
        }
    }
}
