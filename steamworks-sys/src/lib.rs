#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

extern crate libc;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

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
pub struct ISteamUserStats(c_void);
#[repr(C)]
pub struct ISteamGameServer(c_void);
#[repr(C)]
pub struct ISteamNetworking(c_void);

pub type HSteamPipe = i32;
pub type AppId = u32;
pub type SteamAPICall = u64;

#[repr(C)]
pub struct CallbackData {
    pub param_size: c_int,
    pub userdata: *mut c_void,
    pub run: unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void),
    pub run_extra: unsafe extern "C" fn(*mut c_void, *mut c_void, *mut c_void, u8, SteamAPICall),
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
    pub fn steam_rust_get_user_stats() -> *mut ISteamUserStats;
    pub fn steam_rust_get_server() -> *mut ISteamGameServer;
    pub fn steam_rust_get_server_apps() -> *mut ISteamApps;
    pub fn steam_rust_get_networking() -> *mut ISteamNetworking;

    pub fn steam_rust_game_server_init(ip: u32, steam_port: u16, game_port: u16, query_port: u16, server_mode: EServerMode, version: *const c_char) -> c_int;
    pub fn steam_rust_is_steam_id_valid(id: u64) -> c_int;
    pub fn steam_rust_is_game_id_valid(id: u64) -> c_int;
    pub fn steam_rust_get_game_id_mod(id: u64) -> u32;
    pub fn steam_rust_get_game_id_app(id: u64) -> u32;

    //

    pub fn SteamAPI_Init() -> u8;
    pub fn SteamAPI_Shutdown();
    pub fn SteamAPI_RunCallbacks();
    pub fn SteamAPI_RegisterCallback(pCallback: *mut c_void, id: c_int);
    pub fn SteamAPI_UnregisterCallback(pCallback: *mut c_void);
    pub fn SteamAPI_RegisterCallResult(pCallback: *mut c_void, api_call: SteamAPICall);
    pub fn SteamAPI_UnregisterCallResult(pCallback: *mut c_void, api_call: SteamAPICall);
    pub fn SteamAPI_RestartAppIfNecessary(app_id: u32) -> u8;

    pub fn SteamGameServer_Shutdown();
    pub fn SteamGameServer_RunCallbacks();

    pub fn SteamAPI_ISteamClient_CreateSteamPipe(instance: *mut ISteamClient) -> HSteamPipe;
    pub fn SteamAPI_ISteamClient_BReleaseSteamPipe(instance: *mut ISteamClient, pipe: HSteamPipe) -> u8;
    pub fn SteamAPI_ISteamClient_ConnectToGlobalUser(instance: *mut ISteamClient, pipe: HSteamPipe) -> HSteamUser;

    pub fn SteamAPI_ISteamUtils_GetAppID(instance: *mut ISteamUtils) -> u32;
    pub fn SteamAPI_ISteamUtils_GetSteamUILanguage(instance: *mut ISteamUtils) -> *const c_char;
    pub fn SteamAPI_ISteamUtils_IsAPICallCompleted(instance: *mut ISteamUtils, api_call: SteamAPICall, failed: *mut bool) -> u8;
    pub fn SteamAPI_ISteamUtils_SetOverlayNotificationPosition(instance: *mut ISteamUtils, position: ENotificationPosition);
    pub fn SteamAPI_ISteamUtils_GetImageSize(instance: *mut ISteamUtils, image: c_int, width: *mut u32, height: *mut u32) -> u8;
    pub fn SteamAPI_ISteamUtils_GetImageRGBA(instance: *mut ISteamUtils, image: c_int, dest: *mut u8, dest_size: c_int) -> u8;

    pub fn SteamAPI_ISteamApps_BIsAppInstalled(instance: *mut ISteamApps, app_id: AppId) -> u8;
    pub fn SteamAPI_ISteamApps_BIsDlcInstalled(instance: *mut ISteamApps, app_id: AppId) -> u8;
    pub fn SteamAPI_ISteamApps_BIsSubscribedApp(instance: *mut ISteamApps, app_id: AppId) -> u8;
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
    pub fn SteamAPI_ISteamFriends_GetFriendPersonaState(instance: *mut ISteamFriends, friend: u64) -> EPersonaState;
    pub fn SteamAPI_ISteamFriends_RequestUserInformation(instance: *mut ISteamFriends, user_id: u64, name_only: u8) -> u8;
    pub fn SteamAPI_ISteamFriends_ActivateGameOverlayToWebPage(instance: *mut ISteamFriends, url: *const c_char);
    pub fn SteamAPI_ISteamFriends_GetPersonaName(instance: *mut ISteamFriends) -> *const c_char;
    pub fn SteamAPI_ISteamFriends_ActivateGameOverlayInviteDialog(instance: *mut ISteamFriends, lobby: u64);
    pub fn SteamAPI_ISteamFriends_GetSmallFriendAvatar(instance: *mut ISteamFriends, friend: u64) -> c_int;
    pub fn SteamAPI_ISteamFriends_GetMediumFriendAvatar(instance: *mut ISteamFriends, friend: u64) -> c_int;
    pub fn SteamAPI_ISteamFriends_GetFriendGamePlayed(instance: *mut ISteamFriends, friend: u64, game_info: *mut FriendGameInfo_t) -> u8;

    pub fn SteamAPI_ISteamMatchmaking_CreateLobby(instance: *mut ISteamMatchmaking, lobby_ty: ELobbyType, max_members: c_int) -> SteamAPICall;
    pub fn SteamAPI_ISteamMatchmaking_RequestLobbyList(instance: *mut ISteamMatchmaking) -> SteamAPICall;
    pub fn SteamAPI_ISteamMatchmaking_GetLobbyByIndex(instance: *mut ISteamMatchmaking, lobby: c_int) -> u64;
    pub fn SteamAPI_ISteamMatchmaking_LeaveLobby(instance: *mut ISteamMatchmaking, lobby: u64);
    pub fn SteamAPI_ISteamMatchmaking_JoinLobby(instance: *mut ISteamMatchmaking, lobby: u64) -> SteamAPICall;
    pub fn SteamAPI_ISteamMatchmaking_GetLobbyOwner(instance: *mut ISteamMatchmaking, lobby: u64) -> u64;
    pub fn SteamAPI_ISteamMatchmaking_GetNumLobbyMembers(instance: *mut ISteamMatchmaking, lobby: u64) -> c_int;
    pub fn SteamAPI_ISteamMatchmaking_GetLobbyMemberByIndex(instance: *mut ISteamMatchmaking, lobby: u64, member: c_int) -> u64;

    pub fn SteamAPI_ISteamNetworking_AcceptP2PSessionWithUser(instance: *mut ISteamNetworking, remote: u64) -> u8;
    pub fn SteamAPI_ISteamNetworking_CloseP2PSessionWithUser(instance: *mut ISteamNetworking, remote: u64) -> u8;
    pub fn SteamAPI_ISteamNetworking_SendP2PPacket(instance: *mut ISteamNetworking, remote: u64, data: *const c_void, data_len: u32, send_type: EP2PSend, channel: c_int) -> u8;
    pub fn SteamAPI_ISteamNetworking_IsP2PPacketAvailable(instance: *mut ISteamNetworking, msg_size: *mut u32, channel: c_int) -> u8;
    pub fn SteamAPI_ISteamNetworking_ReadP2PPacket(instance: *mut ISteamNetworking, data: *mut c_void, data_len: u32, msg_size: *mut u32, remote: *mut u64, channel: c_int) -> u8;

    pub fn SteamAPI_ISteamUser_GetSteamID(instance: *mut ISteamUser) -> u64;
    pub fn SteamAPI_ISteamUser_GetAuthSessionTicket(instance: *mut ISteamUser, ticket: *mut c_void, max_ticket: c_int, ticket_size: *mut u32) -> HAuthTicket;
    pub fn SteamAPI_ISteamUser_BeginAuthSession(instance: *mut ISteamUser, ticket: *const c_void, ticket_size: *mut u32, steam_id: u64) -> EBeginAuthSessionResult;
    pub fn SteamAPI_ISteamUser_EndAuthSession(instance: *mut ISteamUser, steam_id: u64);
    pub fn SteamAPI_ISteamUser_CancelAuthTicket(instance: *mut ISteamUser, auth_ticket: HAuthTicket);

    pub fn SteamAPI_ISteamUserStats_FindLeaderboard(instance: *mut ISteamUserStats, name: *const c_char) -> SteamAPICall;
    pub fn SteamAPI_ISteamUserStats_FindOrCreateLeaderboard(instance: *mut ISteamUserStats, name: *const c_char, sort_method: ELeaderboardSortMethod, display_type: ELeaderboardDisplayType) -> SteamAPICall;
    pub fn SteamAPI_ISteamUserStats_UploadLeaderboardScore(instance: *mut ISteamUserStats, leaderboard: u64, method: ELeaderboardUploadScoreMethod, score: i32, details: *const i32, details_count: c_int) -> SteamAPICall;
    pub fn SteamAPI_ISteamUserStats_DownloadLeaderboardEntries(instance: *mut ISteamUserStats, leaderboard: u64, data_request: ELeaderboardDataRequest, start: c_int, end: c_int) -> SteamAPICall;
    pub fn SteamAPI_ISteamUserStats_GetDownloadedLeaderboardEntry(instance: *mut ISteamUserStats, entries: SteamLeaderboardEntries_t, index: c_int, entry: *mut LeaderboardEntry_t, details: *mut i32, details_max: c_int) -> u8;

    pub fn SteamAPI_ISteamGameServer_LogOnAnonymous(instance: *mut ISteamGameServer);
    pub fn SteamAPI_ISteamGameServer_SetProduct(instance: *mut ISteamGameServer, product: *const c_char);
    pub fn SteamAPI_ISteamGameServer_SetGameDescription(instance: *mut ISteamGameServer, description: *const c_char);
    pub fn SteamAPI_ISteamGameServer_SetDedicatedServer(instance: *mut ISteamGameServer, dedicated: u8);
    pub fn SteamAPI_ISteamGameServer_GetSteamID(instance: *mut ISteamGameServer) -> u64;
    pub fn SteamAPI_ISteamGameServer_GetAuthSessionTicket(instance: *mut ISteamGameServer, ticket: *mut c_void, max_ticket: c_int, ticket_size: *mut u32) -> HAuthTicket;
    pub fn SteamAPI_ISteamGameServer_BeginAuthSession(instance: *mut ISteamGameServer, ticket: *const c_void, ticket_size: *mut u32, steam_id: u64) -> EBeginAuthSessionResult;
    pub fn SteamAPI_ISteamGameServer_EndAuthSession(instance: *mut ISteamGameServer, steam_id: u64);
    pub fn SteamAPI_ISteamGameServer_CancelAuthTicket(instance: *mut ISteamGameServer, auth_ticket: HAuthTicket);
}
