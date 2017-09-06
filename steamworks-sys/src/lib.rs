
extern crate libc;

use libc::{
    c_char,
    c_void,
    c_int,
};

#[repr(C)]
pub struct ISteamClient(c_void);
#[repr(C)]
pub struct ISteamUtils(c_void);
#[repr(C)]
pub struct ISteamApps(c_void);
#[repr(C)]
pub struct ISteamFriends(c_void);

pub type HSteamPipe = i32;
pub type HSteamUser = i32;
pub type AppId = u32;

pub const STEAMCLIENT_INTERFACE_VERSION: &'static [u8] = b"SteamClient017\0";
pub const STEAMUTILS_INTERFACE_VERSION: &'static [u8] = b"SteamUtils009\0";
pub const STEAMAPPS_INTERFACE_VERSION: &'static [u8] = b"STEAMAPPS_INTERFACE_VERSION008\0";
pub const STEAMFRIENDS_INTERFACE_VERSION: &'static [u8] = b"SteamFriends015\0";

#[repr(C)]
pub enum PersonaState {
    Offline = 0,
    Online = 1,
    Busy = 2,
    Away = 3,
    Snooze = 4,
    LookingToTrade = 5,
    LookingToPlay = 6,
    Max,
}

#[repr(C)]
pub struct PersonaStateChange_t {
    pub steam_id: u64,
    pub flags: c_int,
}

extern "C" {
    // Helpers from lib.cpp

    pub fn register_rust_steam_callback(
        parameter_size: c_int,
        userdata: *mut c_void,
        run_func: extern "C" fn (*mut c_void, *mut c_void),
        dealloc: extern "C" fn (*mut c_void),
        callback_id: c_int
    ) -> *mut c_void;
    pub fn unregister_rust_steam_callback(
        ty: *mut c_void,
    );
    //

    pub fn SteamAPI_Init() -> u8;
    pub fn SteamAPI_Shutdown();
    pub fn SteamAPI_RunCallbacks();

    pub fn SteamInternal_CreateInterface(version: *const c_char) -> *mut ISteamClient;

    pub fn SteamAPI_ISteamClient_CreateSteamPipe(instance: *mut ISteamClient) -> HSteamPipe;
    pub fn SteamAPI_ISteamClient_BReleaseSteamPipe(instance: *mut ISteamClient, pipe: HSteamPipe) -> u8;
    pub fn SteamAPI_ISteamClient_ConnectToGlobalUser(instance: *mut ISteamClient, pipe: HSteamPipe) -> HSteamUser;
    pub fn SteamAPI_ISteamClient_GetISteamUtils(instance: *mut ISteamClient, pipe: HSteamPipe, version: *const c_char) -> *mut ISteamUtils;
    pub fn SteamAPI_ISteamClient_GetISteamApps(instance: *mut ISteamClient, user: HSteamUser, pipe: HSteamPipe, version: *const c_char) -> *mut ISteamApps;
    pub fn SteamAPI_ISteamClient_GetISteamFriends(instance: *mut ISteamClient, user: HSteamUser, pipe: HSteamPipe, version: *const c_char) -> *mut ISteamFriends;

    pub fn SteamAPI_ISteamUtils_GetAppID(instance: *mut ISteamUtils) -> u32;
    pub fn SteamAPI_ISteamUtils_GetSteamUILanguage(instance: *mut ISteamUtils) -> *const c_char;

    pub fn SteamAPI_ISteamApps_BIsAppInstalled(instance: *mut ISteamApps, app_id: AppId) -> u8;
    pub fn SteamAPI_ISteamApps_BIsDlcInstalled(instance: *mut ISteamApps, app_id: AppId) -> u8;
    pub fn SteamAPI_ISteamApps_BIsSubscribedApp(instace: *mut ISteamApps, app_id: AppId) -> u8;
    pub fn SteamAPI_ISteamApps_BIsSubscribedFromFreeWeekend(instance: *mut ISteamApps) -> u8;
    pub fn SteamAPI_ISteamApps_BIsVACBanned(instance: *mut ISteamApps) -> u8;
    pub fn SteamAPI_ISteamApps_BIsCybercafe(instance: *mut ISteamApps) -> u8;
    pub fn SteamAPI_ISteamApps_BIsLowViolence(instance: *mut ISteamApps) -> u8;
    pub fn SteamAPI_ISteamApps_BIsSubscribed(instance: *mut ISteamApps) -> u8;
    pub fn SteamAPI_ISteamApps_GetAppBuildId(instance: *mut ISteamApps) -> c_int;
    pub fn SteamAPI_ISteamApps_GetAppInstallDir(instance: *mut ISteamApps, app_id: AppId, folder: *const c_char, buffer_size: u32) -> u32;
    pub fn SteamAPI_ISteamApps_GetAppOwner(instance: *mut ISteamApps) -> u64;
    pub fn SteamAPI_ISteamApps_GetAvailableGameLanguages(instance: *mut ISteamApps) -> *const c_char;
    pub fn SteamAPI_ISteamApps_GetCurrentBetaName(instance: *mut ISteamApps, name: *const c_char, buffer_size: c_int) -> u8;
    pub fn SteamAPI_ISteamApps_GetCurrentGameLanguage(instance: *mut ISteamApps) -> *const c_char;

    pub fn SteamAPI_ISteamFriends_GetFriendCount(instance: *mut ISteamFriends, flags: c_int) -> c_int;
    pub fn SteamAPI_ISteamFriends_GetFriendByIndex(instance: *mut ISteamFriends, friend: c_int, flags: c_int) -> u64;
    pub fn SteamAPI_ISteamFriends_GetFriendPersonaName(instance: *mut ISteamFriends, friend: u64) -> *const c_char;
    pub fn SteamAPI_ISteamFriends_GetFriendPersonaState(instance: *mut ISteamFriends, friend: u64) -> PersonaState;
    pub fn SteamAPI_ISteamFriends_RequestUserInformation(instance: *mut ISteamFriends, user_id: u64, name_only: u8) -> u8;
}
