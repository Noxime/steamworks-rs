#![allow(non_camel_case_types)]

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
#[repr(C)]
pub struct ISteamMatchmaking(c_void);
#[repr(C)]
pub struct ISteamUser(c_void);
#[repr(C)]
pub struct ISteamGameServer(c_void);

pub type HSteamPipe = i32;
pub type HSteamUser = i32;
pub type HAuthTicket = u32;
pub type AppId = u32;
pub type SteamAPICall = u64;

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
pub enum LobbyType {
    Private = 0,
    FriendsOnly = 1,
    Public = 2,
    Invisible = 3,
}

#[repr(C)]
pub struct PersonaStateChange_t {
    pub steam_id: u64,
    pub flags: c_int,
}

#[repr(C)]
pub struct GetAuthSessionTicketResponse_t {
    pub auth_ticket: HAuthTicket,
    pub result: SResult,
}

#[repr(C)]
pub struct ValidateAuthTicketResponse_t {
    pub steam_id: u64,
    pub response: AuthSessionResponse,
    pub owner_steam_id: u64,
}

#[repr(C)]
pub struct LobbyCreated {
    pub result: SResult,
    pub lobby_steam_id: u64,
}
#[repr(C)]
pub struct LobbyMatchList {
    pub lobbies_matching: u32,
}

#[repr(C)]
pub enum NotificationPosition {
    TopLeft = 0,
    TopRight = 1,
    BottomLeft = 2,
    BottomRight = 3,
}

#[repr(C)]
pub enum BeginAuthSessionResult {
    Ok = 0,
    InvalidTicket = 1,
    DuplicateRequest = 2,
    InvalidVersion = 3,
    GameMismatch = 4,
    ExpiredTicket = 5,
}

#[repr(C)]
pub enum AuthSessionResponse {
    Ok = 0,
    UserNotConnectedToSteam = 1,
    NoLicenseOrExpired = 2,
    VACBanned = 3,
    LoggedInElseWhere = 4,
    VACCheckTimedOut = 5,
    AuthTicketCancelled = 6,
    AuthTicketInvalidAlreadyUsed = 7,
    AuthTicketInvalid = 8,
    PublisherIssuedBan = 9,
}

#[repr(C)]
pub enum ServerMode {
    Invalid = 0,
    NoAuthentication = 1,
    Authentication = 2,
    AuthenticationAndSecure = 3,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum SResult {
    Ok = 1,
    Fail = 2,
    NoConnection = 3,
    InvalidPassword = 5,
    LoggedInElsewhere = 6,
    InvalidProtocolVer = 7,
    InvalidParam = 8,
    FileNotFound = 9,
    Busy = 10,
    InvalidState = 11,
    InvalidName = 12,
    InvalidEmail = 13,
    DuplicateName = 14,
    AccessDenied = 15,
    Timeout = 16,
    Banned = 17,
    AccountNotFound = 18,
    InvalidSteamID = 19,
    ServiceUnavailable = 20,
    NotLoggedOn = 21,
    Pending = 22,
    EncryptionFailure = 23,
    InsufficientPrivilege = 24,
    LimitExceeded = 25,
    Revoked = 26,
    Expired = 27,
    AlreadyRedeemed = 28,
    DuplicateRequest = 29,
    AlreadyOwned = 30,
    IPNotFound = 31,
    PersistFailed = 32,
    LockingFailed = 33,
    LogonSessionReplaced = 34,
    ConnectFailed = 35,
    HandshakeFailed = 36,
    IOFailure = 37,
    RemoteDisconnect = 38,
    ShoppingCartNotFound = 39,
    Blocked = 40,
    Ignored = 41,
    NoMatch = 42,
    AccountDisabled = 43,
    ServiceReadOnly = 44,
    AccountNotFeatured = 45,
    AdministratorOK = 46,
    ContentVersion = 47,
    TryAnotherCM = 48,
    PasswordRequiredToKickSession = 49,
    AlreadyLoggedInElsewhere = 50,
    Suspended = 51,
    Cancelled = 52,
    DataCorruption = 53,
    DiskFull = 54,
    RemoteCallFailed = 55,
    PasswordUnset = 56,
    ExternalAccountUnlinked = 57,
    PSNTicketInvalid = 58,
    ExternalAccountAlreadyLinked = 59,
    RemoteFileConflict = 60,
    IllegalPassword = 61,
    SameAsPreviousValue = 62,
    AccountLogonDenied = 63,
    CannotUseOldPassword = 64,
    InvalidLoginAuthCode = 65,
    AccountLogonDeniedNoMail = 66,
    HardwareNotCapableOfIPT = 67,
    IPTInitError = 68,
    ParentalControlRestricted = 69,
    FacebookQueryError = 70,
    ExpiredLoginAuthCode = 71,
    IPLoginRestrictionFailed = 72,
    AccountLockedDown = 73,
    AccountLogonDeniedVerifiedEmailRequired = 74,
    NoMatchingURL = 75,
    BadResponse = 76,
    RequirePasswordReEntry = 77,
    ValueOutOfRange = 78,
    UnexpectedError = 79,
    Disabled = 80,
    InvalidCEGSubmission = 81,
    RestrictedDevice = 82,
    RegionLocked = 83,
    RateLimitExceeded = 84,
    AccountLoginDeniedNeedTwoFactor = 85,
    ItemDeleted = 86,
    AccountLoginDeniedThrottle = 87,
    TwoFactorCodeMismatch = 88,
    TwoFactorActivationCodeMismatch = 89,
    AccountAssociatedToMultiplePartners = 90,
    NotModified = 91,
    NoMobileDevice = 92,
    TimeNotSynced = 93,
    SmsCodeFailed = 94,
    AccountLimitExceeded = 95,
    AccountActivityLimitExceeded = 96,
    PhoneActivityLimitExceeded = 97,
    RefundToWallet = 98,
    EmailSendFailure = 99,
    NotSettled = 100,
    NeedCaptcha = 101,
    GSLTDenied = 102,
    GSOwnerDenied = 103,
    InvalidItemType = 104,
    IPBanned = 105,
    GSLTExpired = 106,
    InsufficientFunds = 107,
    TooManyPending = 108,
    NoSiteLicensesFound = 109,
    WGNetworkSendExceeded = 110,
    AccountNotFriends = 111,
    LimitedUserAccount = 112,
}

#[repr(C)]
pub struct CallbackData {
    pub param_size: c_int,
    pub userdata: *mut c_void,
    pub run: unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void),
    pub run_extra: unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void, bool, SteamAPICall),
    pub dealloc: unsafe extern "C" fn(*mut c_void, *mut c_void),
} 

extern "C" {
    // Helpers from lib.cpp
    pub fn create_rust_callback(flags: u8, id: c_int, data: CallbackData) -> *mut c_void;
    pub fn delete_rust_callback(cb: *mut c_void);

    pub fn steam_rust_get_client() -> *mut ISteamClient;
    pub fn steam_rust_get_matchmaking() -> *mut ISteamMatchmaking;
    pub fn steam_rust_get_utils() -> *mut ISteamUtils;
    pub fn steam_rust_get_apps() -> *mut ISteamApps;
    pub fn steam_rust_get_friends() -> *mut ISteamFriends;
    pub fn steam_rust_get_user() -> *mut ISteamUser;
    pub fn steam_rust_get_server() -> *mut ISteamGameServer;
    pub fn steam_rust_get_server_apps() -> *mut ISteamApps;

    pub fn steam_rust_game_server_init(ip: u32, steam_port: u16, game_port: u16, query_port: u16, server_mode: ServerMode, version: *const c_char) -> c_int;
    
    //

    pub fn SteamAPI_Init() -> u8;
    pub fn SteamAPI_Shutdown();
    pub fn SteamAPI_RunCallbacks();
    pub fn SteamAPI_RegisterCallback(pCallback: *mut c_void, id: c_int);
    pub fn SteamAPI_UnregisterCallback(pCallback: *mut c_void);
    pub fn SteamAPI_RegisterCallResult(pCallback: *mut c_void, api_call: SteamAPICall);
    pub fn SteamAPI_UnregisterCallResult(pCallback: *mut c_void, api_call: SteamAPICall);

    pub fn SteamGameServer_Shutdown();
    pub fn SteamGameServer_RunCallbacks();

    pub fn SteamAPI_ISteamClient_CreateSteamPipe(instance: *mut ISteamClient) -> HSteamPipe;
    pub fn SteamAPI_ISteamClient_BReleaseSteamPipe(instance: *mut ISteamClient, pipe: HSteamPipe) -> u8;
    pub fn SteamAPI_ISteamClient_ConnectToGlobalUser(instance: *mut ISteamClient, pipe: HSteamPipe) -> HSteamUser;

    pub fn SteamAPI_ISteamUtils_GetAppID(instance: *mut ISteamUtils) -> u32;
    pub fn SteamAPI_ISteamUtils_GetSteamUILanguage(instance: *mut ISteamUtils) -> *const c_char;
    pub fn SteamAPI_ISteamUtils_IsAPICallCompleted(instance: *mut ISteamUtils, api_call: SteamAPICall, failed: *mut bool) -> bool;
    pub fn SteamAPI_ISteamUtils_SetOverlayNotificationPosition(instance: *mut ISteamUtils, position: NotificationPosition);

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
    pub fn SteamAPI_ISteamFriends_ActivateGameOverlayToWebPage(instance: *mut ISteamFriends, url: *const c_char);
    pub fn SteamAPI_ISteamFriends_GetPersonaName(instance: *mut ISteamFriends) -> *const c_char;

    pub fn SteamAPI_ISteamMatchmaking_CreateLobby(instance: *mut ISteamMatchmaking, lobby_ty: LobbyType, max_members: c_int) -> SteamAPICall;
    pub fn SteamAPI_ISteamMatchmaking_RequestLobbyList(instance: *mut ISteamMatchmaking) -> SteamAPICall;
    pub fn SteamAPI_ISteamMatchmaking_GetLobbyByIndex(instance: *mut ISteamMatchmaking, lobby: c_int) -> u64;

    pub fn SteamAPI_ISteamUser_GetSteamID(instance: *mut ISteamUser) -> u64;
    pub fn SteamAPI_ISteamUser_GetAuthSessionTicket(instance: *mut ISteamUser, ticket: *mut c_void, max_ticket: c_int, ticket_size: *mut u32) -> HAuthTicket;
    pub fn SteamAPI_ISteamUser_BeginAuthSession(instance: *mut ISteamUser, ticket: *const c_void, ticket_size: *mut u32, steam_id: u64) -> BeginAuthSessionResult;
    pub fn SteamAPI_ISteamUser_EndAuthSession(instance: *mut ISteamUser, steam_id: u64);
    pub fn SteamAPI_ISteamUser_CancelAuthTicket(instance: *mut ISteamUser, auth_ticket: HAuthTicket);

    pub fn SteamAPI_ISteamGameServer_LogOnAnonymous(instance: *mut ISteamGameServer);
    pub fn SteamAPI_ISteamGameServer_SetProduct(instance: *mut ISteamGameServer, product: *const c_char);
    pub fn SteamAPI_ISteamGameServer_SetGameDescription(instance: *mut ISteamGameServer, description: *const c_char);
    pub fn SteamAPI_ISteamGameServer_SetDedicatedServer(instance: *mut ISteamGameServer, dedicated: u8);
    pub fn SteamAPI_ISteamGameServer_GetSteamID(instance: *mut ISteamGameServer) -> u64;
    pub fn SteamAPI_ISteamGameServer_GetAuthSessionTicket(instance: *mut ISteamGameServer, ticket: *mut c_void, max_ticket: c_int, ticket_size: *mut u32) -> HAuthTicket;
    pub fn SteamAPI_ISteamGameServer_BeginAuthSession(instance: *mut ISteamGameServer, ticket: *const c_void, ticket_size: *mut u32, steam_id: u64) -> BeginAuthSessionResult;
    pub fn SteamAPI_ISteamGameServer_EndAuthSession(instance: *mut ISteamGameServer, steam_id: u64);
    pub fn SteamAPI_ISteamGameServer_CancelAuthTicket(instance: *mut ISteamGameServer, auth_ticket: HAuthTicket);
}
