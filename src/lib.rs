#![doc = include_str!("../README.md")]

#[macro_use]
extern crate thiserror;
#[macro_use]
extern crate bitflags;

use screenshots::Screenshots;
#[cfg(feature = "raw-bindings")]
pub use steamworks_sys as sys;
#[cfg(not(feature = "raw-bindings"))]
use steamworks_sys as sys;
use sys::{EServerMode, ESteamAPIInitResult, SteamErrMsg};

use core::ffi::c_void;
use std::collections::HashMap;
use std::ffi::{c_char, CStr, CString};
use std::fmt::{self, Debug, Formatter};
use std::sync::atomic::AtomicU64;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, Weak};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub use crate::app::*;
pub use crate::callback::*;
pub use crate::error::*;
pub use crate::friends::*;
pub use crate::input::*;
pub use crate::matchmaking::*;
pub use crate::matchmaking_servers::*;
pub use crate::networking::*;
pub use crate::remote_play::*;
pub use crate::remote_storage::*;
pub use crate::server::*;
pub use crate::timeline::*;
pub use crate::ugc::*;
pub use crate::user::*;
pub use crate::user_stats::*;
pub use crate::utils::*;

#[macro_use]
mod callback;
mod app;
mod error;
mod friends;
mod input;
mod matchmaking;
mod matchmaking_servers;
mod networking;
pub mod networking_messages;
pub mod networking_sockets;
mod networking_sockets_callback;
pub mod networking_types;
pub mod networking_utils;
mod remote_play;
mod remote_storage;
pub mod screenshots;
mod server;
pub mod timeline;
mod ugc;
mod user;
mod user_stats;
mod utils;

/// Alias for [`Result<T, SteamError>`](SteamError)
pub type SteamResult<T = ()> = Result<T, SteamError>;

/// Convert a raw result to a Rust [`Result`] type.
pub(crate) fn to_steam_result(result: sys::EResult) -> SteamResult {
    match result {
        // sys::EResult::k_EResultNone => Ok(()), TODO: I'm not sure if this is considered a success
        sys::EResult::k_EResultOK => Ok(()),
        error => Err(error.try_into().unwrap()),
    }
}

// A note about thread-safety:
// The steam api is assumed to be thread safe unless
// the documentation for a method states otherwise,
// however this is never stated anywhere in the docs
// that I could see.

/// The main entry point into the steam client.
///
/// This provides access to all of the steamworks api that
/// clients can use.
pub struct Client {
    inner: Arc<Inner>,
}

impl Clone for Client {
    fn clone(&self) -> Self {
        Client {
            inner: self.inner.clone(),
        }
    }
}

struct Inner {
    manager: Manager,
    callbacks: Callbacks,
    networking_sockets_data: Mutex<NetworkingSocketsData>,
}

struct Callbacks {
    /// Registered callbacks keyed by callback type ID. Multiple callbacks of
    /// the same type coexist and are dispatched in registration order; each
    /// entry carries a unique sequence number so a `CallbackHandle` removes
    /// exactly its own registration.
    callbacks: Mutex<HashMap<i32, Vec<(u64, Box<dyn FnMut(*mut c_void) + Send + 'static>)>>>,
    call_results:
        Mutex<HashMap<sys::SteamAPICall_t, Box<dyn FnOnce(*mut c_void, bool) + Send + 'static>>>,
    /// Registrations made through `register_replacing_callback`, keyed by
    /// callback type ID. Re-registering through that helper drops the stored
    /// handle, which preserves the documented replace semantics of those
    /// wrapper APIs without touching registrations made by other callers.
    replacing: Mutex<HashMap<i32, CallbackHandle>>,
    /// Source of the unique sequence numbers stored in `callbacks` entries.
    next_seq: AtomicU64,
}

impl Callbacks {
    /// Calls every callback registered for `id`, in registration order.
    ///
    /// The table lock is held while the callbacks run: registering or
    /// removing callbacks from inside a callback deadlocks (pre-existing
    /// behavior, tracked as defect #2 residual).
    fn dispatch(&self, id: i32, data: *mut c_void) {
        let mut callbacks = self.callbacks.lock().unwrap();
        if let Some(entries) = callbacks.get_mut(&id) {
            for (_seq, cb) in entries.iter_mut() {
                cb(data);
            }
        }
    }
}

impl Inner {
    /// Runs any currently pending callbacks
    ///
    /// This runs all currently pending callbacks on the current
    /// thread.
    ///
    /// This should be called frequently (e.g. once per a frame)
    /// in order to reduce the latency between receiving events.
    pub fn run_callbacks(&self) {
        self.run_callbacks_raw(|cb_discrim, data| {
            self.callbacks.dispatch(cb_discrim, data);
        });
    }

    /// Runs any currently pending callbacks.
    ///
    /// This is identical to `run_callbacks` in every way, except that
    /// `callback_handler` is called for every callback invoked.
    ///
    /// This option provides an alternative for handling callbacks that
    /// don't require the handler to be `Send`, and `'static`.
    ///
    /// This should be called frequently (e.g. once per a frame)
    /// in order to reduce the latency between receiving events.
    pub fn process_callbacks(&self, mut callback_handler: impl FnMut(CallbackResult)) {
        self.run_callbacks_raw(|cb_discrim, data| {
            self.callbacks.dispatch(cb_discrim, data);
            let cb_result = unsafe { CallbackResult::from_raw(cb_discrim, data) };
            if let Some(cb_result) = cb_result {
                callback_handler(cb_result);
            }
        });
    }

    fn run_callbacks_raw(&self, mut callback_handler: impl FnMut(i32, *mut c_void)) {
        unsafe {
            let pipe = self.manager.get_pipe();
            sys::SteamAPI_ManualDispatch_RunFrame(pipe);
            let mut callback = std::mem::zeroed();
            let mut apicall_result = Vec::new();
            while sys::SteamAPI_ManualDispatch_GetNextCallback(pipe, &mut callback) {
                if callback.m_iCallback == sys::SteamAPICallCompleted_t_k_iCallback as i32 {
                    let apicall = callback
                        .m_pubParam
                        .cast::<sys::SteamAPICallCompleted_t>()
                        .read_unaligned();
                    apicall_result.resize(apicall.m_cubParam as usize, 0u8);
                    let mut failed = false;
                    if sys::SteamAPI_ManualDispatch_GetAPICallResult(
                        pipe,
                        apicall.m_hAsyncCall,
                        apicall_result.as_mut_ptr().cast(),
                        apicall.m_cubParam as _,
                        apicall.m_iCallback,
                        &mut failed,
                    ) {
                        let mut call_results = self.callbacks.call_results.lock().unwrap();
                        // The &{val} pattern here is to avoid taking a reference to a packed field
                        // Since the value here is Copy, we can just copy it and borrow the copy
                        if let Some(cb) = call_results.remove(&{ apicall.m_hAsyncCall }) {
                            cb(apicall_result.as_mut_ptr().cast(), failed);
                        }
                    }
                } else {
                    callback_handler(callback.m_iCallback, callback.m_pubParam.cast());
                }
                sys::SteamAPI_ManualDispatch_FreeLastCallback(pipe);
            }
        }
    }
}

struct NetworkingSocketsData {
    sockets: HashMap<
        sys::HSteamListenSocket,
        (
            Weak<networking_sockets::InnerSocket>,
            Sender<networking_types::ListenSocketEvent>,
        ),
    >,
    /// Connections to a remote listening port
    independent_connections:
        HashMap<sys::HSteamNetConnection, Sender<networking_types::NetConnectionEvent>>,
    connection_callback: Weak<CallbackHandle>,
}

/// Returns true if the app wasn't launched through steam and
/// begins relaunching it, the app should exit as soon as possible.
///
/// Returns false if the app was either launched through steam
/// or has a `steam_appid.txt`
pub fn restart_app_if_necessary(app_id: AppId) -> bool {
    unsafe { sys::SteamAPI_RestartAppIfNecessary(app_id.0) }
}

fn static_assert_send<T: Send>() {}
fn static_assert_sync<T>()
where
    T: Sync,
{
}

impl Client {
    /// Call to the native SteamAPI_Init function.
    /// should not be used directly, but through either
    /// init_flat() or init_flat_app()
    unsafe fn steam_api_init_flat(p_out_err_msg: *mut SteamErrMsg) -> ESteamAPIInitResult {
        unsafe { sys::SteamAPI_InitFlat(p_out_err_msg) }
    }

    /// Attempts to initialize the steamworks api without full API integration
    /// through SteamAPI_InitFlat added in SDK 1.59
    /// and returns a client to access the rest of the api.
    ///
    /// This should only ever have one instance per a program.
    ///
    /// # Errors
    ///
    /// This can fail if:
    /// * The steam client isn't running
    /// * The app ID of the game couldn't be determined.
    ///
    ///   If the game isn't being run through steam this can be provided by
    ///   placing a `steam_appid.txt` with the ID inside in the current
    ///   working directory. Alternatively, you can use `Client::init_app(<app_id>)`
    ///   to force a specific app ID.
    /// * The game isn't running on the same user/level as the steam client
    /// * The user doesn't own a license for the game.
    /// * The app ID isn't completely set up.
    pub fn init() -> Result<Client, SteamAPIInitError> {
        static_assert_send::<Client>();
        static_assert_sync::<Client>();
        unsafe {
            let mut err_msg: sys::SteamErrMsg = [0; 1024];
            let result = Self::steam_api_init_flat(&mut err_msg);

            if result != sys::ESteamAPIInitResult::k_ESteamAPIInitResult_OK {
                return Err(SteamAPIInitError::from_result_and_message(result, err_msg));
            }

            sys::SteamAPI_ManualDispatch_Init();
            let client = Arc::new(Inner {
                manager: Manager::Client,
                callbacks: Callbacks {
                    callbacks: Mutex::new(HashMap::new()),
                    call_results: Mutex::new(HashMap::new()),
                    replacing: Mutex::new(HashMap::new()),
                    next_seq: AtomicU64::new(1),
                },
                networking_sockets_data: Mutex::new(NetworkingSocketsData {
                    sockets: Default::default(),
                    independent_connections: Default::default(),
                    connection_callback: Default::default(),
                }),
            });
            Ok(Client { inner: client })
        }
    }

    /// Attempts to initialize the steamworks api with the APP_ID
    /// without full API integration through SteamAPI_InitFlat
    /// and returns a client to access the rest of the api.
    ///
    /// This should only ever have one instance per a program.
    ///
    /// # Errors
    ///
    /// This can fail if:
    /// * The steam client isn't running
    /// * The game isn't running on the same user/level as the steam client
    /// * The user doesn't own a license for the game.
    /// * The app ID isn't completely set up.
    pub fn init_app<ID: Into<AppId>>(app_id: ID) -> Result<Client, SteamAPIInitError> {
        let app_id = app_id.into().0.to_string();
        std::env::set_var("SteamAppId", &app_id);
        std::env::set_var("SteamGameId", app_id);
        Client::init()
    }
}

impl Client {
    /// Runs any currently pending callbacks
    ///
    /// This runs all currently pending callbacks on the current
    /// thread.
    ///
    /// This should be called frequently (e.g. once per a frame)
    /// in order to reduce the latency between recieving events.
    pub fn run_callbacks(&self) {
        self.inner.run_callbacks()
    }

    /// Runs any currently pending callbacks.
    ///
    /// This is identical to `run_callbacks` in every way, except that
    /// `callback_handler` is called for every callback invoked.
    ///
    /// This option provides an alternative for handling callbacks that
    /// can doesn't require the handler to be `Send`, and `'static`.
    ///
    /// This should be called frequently (e.g. once per a frame)
    /// in order to reduce the latency between recieving events.
    pub fn process_callbacks(&self, mut callback_handler: impl FnMut(CallbackResult)) {
        self.inner.process_callbacks(&mut callback_handler)
    }

    /// Registers the passed function as a callback for the
    /// given type.
    ///
    /// Registering multiple callbacks of the same type is supported: they are
    /// called in registration order, and each registration stays active until
    /// its returned handle is dropped. Dropping a handle removes only the
    /// registration it was created for; other registrations of the same
    /// callback type are unaffected.
    ///
    /// The callback will be run on the thread that [`run_callbacks`]
    /// is called when the event arrives.
    ///
    /// If the callback handler cannot be made `Send` or `'static`
    /// the call to [`run_callbacks`] should be replaced with a call to
    /// [`process_callbacks`] instead.
    ///
    /// [`run_callbacks`]: Self::run_callbacks
    /// [`process_callbacks`]: Self::process_callbacks
    pub fn register_callback<C, F>(&self, f: F) -> CallbackHandle
    where
        C: Callback,
        F: FnMut(C) + 'static + Send,
    {
        unsafe { register_callback(&self.inner, f) }
    }

    /// Returns an accessor to the steam utils interface
    pub fn utils(&self) -> Utils {
        unsafe {
            let utils = sys::SteamAPI_SteamUtils_v010();
            debug_assert!(!utils.is_null());
            Utils {
                utils: utils,
                _inner: self.inner.clone(),
            }
        }
    }

    /// Returns an accessor to the steam matchmaking interface
    pub fn matchmaking(&self) -> Matchmaking {
        unsafe {
            let mm = sys::SteamAPI_SteamMatchmaking_v009();
            debug_assert!(!mm.is_null());
            Matchmaking {
                mm: mm,
                inner: self.inner.clone(),
            }
        }
    }

    /// Returns an accessor to the steam matchmaking_servers interface
    pub fn matchmaking_servers(&self) -> MatchmakingServers {
        unsafe {
            let mm = sys::SteamAPI_SteamMatchmakingServers_v002();
            debug_assert!(!mm.is_null());
            MatchmakingServers {
                mms: mm,
                _inner: self.inner.clone(),
            }
        }
    }

    /// Returns an accessor to the steam networking interface
    pub fn networking(&self) -> Networking {
        unsafe {
            let net = sys::SteamAPI_SteamNetworking_v006();
            debug_assert!(!net.is_null());
            Networking {
                net: net,
                _inner: self.inner.clone(),
            }
        }
    }

    /// Returns an accessor to the steam apps interface
    pub fn apps(&self) -> Apps {
        unsafe {
            let apps = sys::SteamAPI_SteamApps_v009();
            debug_assert!(!apps.is_null());
            Apps {
                apps: apps,
                _inner: self.inner.clone(),
            }
        }
    }

    /// Returns an accessor to the steam friends interface
    pub fn friends(&self) -> Friends {
        unsafe {
            let friends = sys::SteamAPI_SteamFriends_v018();
            debug_assert!(!friends.is_null());
            Friends {
                friends: friends,
                inner: self.inner.clone(),
            }
        }
    }

    /// Returns an accessor to the steam input interface
    pub fn input(&self) -> Input {
        unsafe {
            let input = sys::SteamAPI_SteamInput_v006();
            debug_assert!(!input.is_null());
            Input {
                input,
                _inner: self.inner.clone(),
            }
        }
    }

    /// Returns an accessor to the steam user interface
    pub fn user(&self) -> User {
        unsafe {
            let user = sys::SteamAPI_SteamUser_v023();
            debug_assert!(!user.is_null());
            User {
                user,
                _inner: self.inner.clone(),
            }
        }
    }

    /// Returns an accessor to the steam user stats interface
    pub fn user_stats(&self) -> UserStats {
        unsafe {
            let us = sys::SteamAPI_SteamUserStats_v013();
            debug_assert!(!us.is_null());
            UserStats {
                user_stats: us,
                inner: self.inner.clone(),
            }
        }
    }

    /// Returns an accessor to the steam remote play interface
    pub fn remote_play(&self) -> RemotePlay {
        unsafe {
            let rp = sys::SteamAPI_SteamRemotePlay_v004();
            debug_assert!(!rp.is_null());
            RemotePlay {
                rp,
                inner: self.inner.clone(),
            }
        }
    }

    /// Returns an accessor to the steam remote storage interface
    pub fn remote_storage(&self) -> RemoteStorage {
        unsafe {
            let rs = sys::SteamAPI_SteamRemoteStorage_v016();
            debug_assert!(!rs.is_null());
            let util = sys::SteamAPI_SteamUtils_v010();
            debug_assert!(!util.is_null());
            RemoteStorage {
                rs,
                util,
                inner: self.inner.clone(),
            }
        }
    }

    /// Returns an accessor to the steam screenshots interface
    pub fn screenshots(&self) -> Screenshots {
        unsafe {
            let screenshots = sys::SteamAPI_SteamScreenshots_v003();
            debug_assert!(!screenshots.is_null());
            Screenshots {
                screenshots,
                _inner: self.inner.clone(),
            }
        }
    }

    /// Returns an accessor to the steam UGC interface (steam workshop)
    pub fn ugc(&self) -> UGC {
        unsafe {
            let ugc = sys::SteamAPI_SteamUGC_v021();
            debug_assert!(!ugc.is_null());
            UGC {
                ugc,
                inner: self.inner.clone(),
            }
        }
    }

    /// Returns an accessor to the steam timeline interface
    pub fn timeline(&self) -> Timeline {
        unsafe {
            let timeline = sys::SteamAPI_SteamTimeline_v004();

            Timeline {
                timeline,
                disabled: timeline.is_null(),
                _inner: self.inner.clone(),
            }
        }
    }

    pub fn networking_messages(&self) -> networking_messages::NetworkingMessages {
        unsafe {
            let net = sys::SteamAPI_SteamNetworkingMessages_SteamAPI_v002();
            debug_assert!(!net.is_null());
            networking_messages::NetworkingMessages {
                net,
                inner: self.inner.clone(),
            }
        }
    }

    pub fn networking_sockets(&self) -> networking_sockets::NetworkingSockets {
        unsafe {
            let sockets = sys::SteamAPI_SteamNetworkingSockets_SteamAPI_v012();
            debug_assert!(!sockets.is_null());
            networking_sockets::NetworkingSockets {
                sockets,
                inner: self.inner.clone(),
            }
        }
    }

    pub fn networking_utils(&self) -> networking_utils::NetworkingUtils {
        unsafe {
            let utils = sys::SteamAPI_SteamNetworkingUtils_SteamAPI_v004();
            debug_assert!(!utils.is_null());
            networking_utils::NetworkingUtils {
                utils,
                inner: self.inner.clone(),
            }
        }
    }
}

/// Used to separate client and game server modes
enum Manager {
    Client,
    Server,
}

impl Manager {
    /// Returns the pipe handle for the steam api
    fn get_pipe(&self) -> sys::HSteamPipe {
        match self {
            Manager::Client => unsafe { sys::SteamAPI_GetHSteamPipe() },
            Manager::Server => unsafe { sys::SteamGameServer_GetHSteamPipe() },
        }
    }
}

impl Drop for Manager {
    fn drop(&mut self) {
        // SAFETY: This is considered unsafe only because of FFI, the function is otherwise
        // always safe to call from any thread.
        match self {
            Manager::Client => unsafe { sys::SteamAPI_Shutdown() },
            Manager::Server => unsafe { sys::SteamGameServer_Shutdown() },
        }
    }
}

/// A user's steam id
#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SteamId(pub(crate) u64);

impl SteamId {
    /// Creates a `SteamId` from a raw 64 bit value.
    ///
    /// May be useful for deserializing steam ids from
    /// a network or save format.
    pub fn from_raw(id: u64) -> SteamId {
        SteamId(id)
    }

    /// Returns the raw 64 bit value of the steam id
    ///
    /// May be useful for serializing steam ids over a
    /// network or to a save format.
    pub fn raw(&self) -> u64 {
        self.0
    }

    /// Returns whether or not this Steam ID is invalid, which is when `account_type` is `k_EAccountTypeInvalid`.
    pub fn is_invalid(&self) -> bool {
        unsafe {
            let bits = sys::CSteamID_SteamID_t {
                m_unAll64Bits: self.0,
            };
            bits.m_comp.m_EAccountType()
                == sys::EAccountType::k_EAccountTypeInvalid as std::os::raw::c_uint
        }
    }

    /// Returns the account id for this steam id
    pub fn account_id(&self) -> AccountId {
        unsafe {
            let bits = sys::CSteamID_SteamID_t {
                m_unAll64Bits: self.0,
            };
            AccountId(bits.m_comp.m_unAccountID())
        }
    }

    /// Returns the Steam universe this Steam ID is part of.
    pub fn universe(&self) -> Universe {
        let bits = sys::CSteamID_SteamID_t {
            m_unAll64Bits: self.0,
        };
        match unsafe { bits.m_comp }.m_EUniverse() {
            sys::EUniverse::k_EUniversePublic => Universe::Public,
            sys::EUniverse::k_EUniverseBeta => Universe::Beta,
            sys::EUniverse::k_EUniverseInternal => Universe::Internal,
            sys::EUniverse::k_EUniverseDev => Universe::Dev,
            _ => Universe::Invalid,
        }
    }

    pub fn account_type(&self) -> AccountType {
        let bits = sys::CSteamID_SteamID_t {
            m_unAll64Bits: self.0,
        };
        match unsafe { bits.m_comp }.m_EAccountType() {
            x if x == sys::EAccountType::k_EAccountTypeIndividual as u32 => AccountType::Individual,
            x if x == sys::EAccountType::k_EAccountTypeMultiseat as u32 => AccountType::Multiseat,
            x if x == sys::EAccountType::k_EAccountTypeGameServer as u32 => AccountType::GameServer,
            x if x == sys::EAccountType::k_EAccountTypeAnonGameServer as u32 => {
                AccountType::AnonGameServer
            }
            x if x == sys::EAccountType::k_EAccountTypePending as u32 => AccountType::Pending,
            x if x == sys::EAccountType::k_EAccountTypeContentServer as u32 => {
                AccountType::ContentServer
            }
            x if x == sys::EAccountType::k_EAccountTypeClan as u32 => AccountType::Clan,
            x if x == sys::EAccountType::k_EAccountTypeChat as u32 => AccountType::Chat,
            x if x == sys::EAccountType::k_EAccountTypeConsoleUser as u32 => {
                AccountType::ConsoleUser
            }
            x if x == sys::EAccountType::k_EAccountTypeAnonUser as u32 => AccountType::AnonUser,
            _ => AccountType::Invalid,
        }
    }

    /// Returns the formatted SteamID32 string for this steam id.
    pub fn steamid32(&self) -> String {
        let account_id = self.account_id().raw();
        let last_bit = account_id & 1;
        format!("STEAM_0:{}:{}", last_bit, (account_id >> 1))
    }
}

/// Steam universes.
///
/// Each universe is a self-contained Steam instance.
#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(u32)]
pub enum Universe {
    Invalid = 0,
    Public = 1,
    Beta = 2,
    Internal = 3,
    Dev = 4,
}

/// Steam account types.
///
/// [`SteamId`]s are used to identify many different types of entities within Steam.
/// This type represents the type of account associated with a Steam ID.
#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(u32)]
pub enum AccountType {
    /// Invalid user account type.
    Invalid = 0,
    /// Individual user account.
    Individual = 1,
    /// Multiseat (e.g. cybercafe) account.
    Multiseat = 2,
    /// Game server account with a fixed Steam ID.
    GameServer = 3,
    /// Anonymous game server account.
    AnonGameServer = 4,
    /// Pending account.
    Pending = 5,
    /// Identifies a Steam content server.
    ContentServer = 6,
    /// Identifies a Steam clan.
    Clan = 7,
    /// Identifies a Steam chat.
    Chat = 8,
    /// Fake SteamID for local PSN account on PS3 or Live account on 360, etc.
    ConsoleUser = 9,
    /// Anoynmous user account.
    AnonUser = 10,
}

/// A user's account id
#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AccountId(pub(crate) u32);

impl AccountId {
    /// Creates an `AccountId` from a raw 32 bit value.
    ///
    /// May be useful for deserializing account ids from
    /// a network or save format.
    pub fn from_raw(id: u32) -> AccountId {
        AccountId(id)
    }

    /// Returns the raw 32 bit value of the steam id
    ///
    /// May be useful for serializing steam ids over a
    /// network or to a save format.
    pub fn raw(&self) -> u32 {
        self.0
    }
}

/// A game id
///
/// Combines `AppId` and other information
#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GameId(#[cfg_attr(feature = "serde", serde(rename = "game_id"))] pub(crate) u64);

impl GameId {
    /// Creates a `GameId` from a raw 64 bit value.
    ///
    /// May be useful for deserializing game ids from
    /// a network or save format.
    pub fn from_raw(id: u64) -> GameId {
        GameId(id)
    }

    /// Returns the raw 64 bit value of the game id
    ///
    /// May be useful for serializing game ids over a
    /// network or to a save format.
    pub fn raw(&self) -> u64 {
        self.0
    }

    /// Returns the app id of this game
    pub fn app_id(&self) -> AppId {
        // TODO: Relies on internal details
        AppId((self.0 & 0xFF_FF_FF) as u32)
    }
}

#[cfg(test)]
mod tests {
    use serial_test::serial;

    use super::*;

    #[test]
    #[serial]
    fn basic_test() {
        let client = Client::init().unwrap();

        let _cb = client.register_callback(|p: PersonaStateChange| {
            println!("Got callback: {:?}", p);
        });

        let utils = client.utils();
        println!("Utils:");
        println!("AppId: {:?}", utils.app_id());
        println!("UI Language: {}", utils.ui_language());

        let apps = client.apps();
        println!("Apps");
        println!("IsInstalled(480): {}", apps.is_app_installed(AppId(480)));
        println!("InstallDir(480): {}", apps.app_install_dir(AppId(480)));
        println!("BuildId: {}", apps.app_build_id());
        println!("AppOwner: {:?}", apps.app_owner());
        println!("Langs: {:?}", apps.available_game_languages());
        println!("Lang: {}", apps.current_game_language());
        println!("Beta: {:?}", apps.current_beta_name());

        let friends = client.friends();
        println!("Friends");
        let list = friends.get_friends(FriendFlags::IMMEDIATE);
        println!("{:?}", list);
        for f in &list {
            println!("Friend: {:?} - {}({:?})", f.id(), f.name(), f.state());
            friends.request_user_information(f.id(), true);
        }
        friends.request_user_information(SteamId(76561198174976054), true);

        for _ in 0..50 {
            client.run_callbacks();
            ::std::thread::sleep(::std::time::Duration::from_millis(100));
        }
    }

    #[test]
    fn steamid_test() {
        let steamid = SteamId(76561198040894045);
        assert_eq!("STEAM_0:1:40314158", steamid.steamid32());

        let steamid = SteamId(76561198174976054);
        assert_eq!("STEAM_0:0:107355163", steamid.steamid32());
    }
}

#[cfg(test)]
mod callback_multi_registration_tests {
    use super::*;
    use std::sync::mpsc;

    use crate::networking_sockets_callback::get_or_create_connection_callback;
    use crate::networking_types::NetConnectionStatusChanged;

    /// Stand-in callback type with an ID outside every real Steam callback ID,
    /// so table assertions never collide with registrations made elsewhere.
    struct TestCallback;
    unsafe impl Callback for TestCallback {
        const ID: i32 = 9_999_999;
        unsafe fn from_raw(_raw: *mut c_void) -> Self {
            TestCallback
        }
    }

    fn test_inner() -> Arc<Inner> {
        let inner = Arc::new(Inner {
            manager: Manager::Client,
            callbacks: Callbacks {
                callbacks: Mutex::new(HashMap::new()),
                call_results: Mutex::new(HashMap::new()),
                replacing: Mutex::new(HashMap::new()),
                next_seq: AtomicU64::new(1),
            },
            networking_sockets_data: Mutex::new(NetworkingSocketsData {
                sockets: HashMap::new(),
                independent_connections: HashMap::new(),
                connection_callback: Weak::new(),
            }),
        });
        // Leak one strong reference: dropping Inner fires SteamAPI_Shutdown
        // via Manager::drop, and these tests never initialize the Steam API.
        std::mem::forget(Arc::clone(&inner));
        inner
    }

    fn registered_count(inner: &Inner, id: i32) -> usize {
        inner
            .callbacks
            .callbacks
            .lock()
            .unwrap()
            .get(&id)
            .map(Vec::len)
            .unwrap_or(0)
    }

    /// Drives the same `Callbacks::dispatch` method the production dispatch
    /// paths use; `from_raw` is never called because TestCallback ignores it.
    fn dispatch_test_callback(inner: &Inner) {
        inner
            .callbacks
            .dispatch(TestCallback::ID, std::ptr::null_mut());
    }

    fn tag_receiver<C>(tx: mpsc::Sender<u32>, tag: u32) -> impl FnMut(C) + Send + 'static {
        move |_| {
            tx.send(tag).unwrap();
        }
    }

    /// Repro scenario A from defect-01: two callbacks of the same type must
    /// both fire, in registration order (was: second silently overwrote first).
    #[test]
    fn two_callbacks_same_type_both_fire_in_order() {
        let inner = test_inner();
        let (tx, rx) = mpsc::channel();
        let _h1 =
            unsafe { register_callback::<TestCallback, _>(&inner, tag_receiver(tx.clone(), 1)) };
        let _h2 = unsafe { register_callback::<TestCallback, _>(&inner, tag_receiver(tx, 2)) };
        assert_eq!(registered_count(&inner, TestCallback::ID), 2);
        dispatch_test_callback(&inner);
        assert_eq!(rx.try_recv().unwrap(), 1);
        assert_eq!(rx.try_recv().unwrap(), 2);
        assert!(rx.try_recv().is_err());
    }

    /// Repro scenario B from defect-01: dropping the first handle must remove
    /// only the first registration (was: removed by ID and killed the second).
    #[test]
    fn dropping_first_handle_keeps_second() {
        let inner = test_inner();
        let (tx, rx) = mpsc::channel();
        let h1 =
            unsafe { register_callback::<TestCallback, _>(&inner, tag_receiver(tx.clone(), 1)) };
        let _h2 = unsafe { register_callback::<TestCallback, _>(&inner, tag_receiver(tx, 2)) };
        drop(h1);
        assert_eq!(registered_count(&inner, TestCallback::ID), 1);
        dispatch_test_callback(&inner);
        assert_eq!(rx.try_recv().unwrap(), 2);
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn dropping_last_handle_removes_table_entry() {
        let inner = test_inner();
        let (tx, rx) = mpsc::channel();
        let h = unsafe { register_callback::<TestCallback, _>(&inner, tag_receiver(tx, 1)) };
        assert_eq!(registered_count(&inner, TestCallback::ID), 1);
        drop(h);
        assert_eq!(registered_count(&inner, TestCallback::ID), 0);
        dispatch_test_callback(&inner);
        assert!(rx.try_recv().is_err());
    }

    /// New finding 2 from defect-01: the sockets-internal registration and a
    /// user registration of `NetConnectionStatusChanged` must coexist (was:
    /// whichever came second silently killed the other), and dropping the
    /// internal handle must leave the user's registration intact.
    #[test]
    fn internal_and_user_registration_coexist() {
        let inner = test_inner();
        let (tx, rx) = mpsc::channel();
        let _user = unsafe {
            register_callback::<NetConnectionStatusChanged, _>(&inner, tag_receiver(tx, 7))
        };
        let internal = get_or_create_connection_callback(Arc::clone(&inner), std::ptr::null_mut());
        assert_eq!(
            registered_count(&inner, NetConnectionStatusChanged::ID),
            2,
            "internal registration must not overwrite the user's callback"
        );
        drop(internal);
        assert_eq!(
            registered_count(&inner, NetConnectionStatusChanged::ID),
            1,
            "dropping the internal handle must not remove the user's callback"
        );
        dispatch_test_callback(&inner);
        assert!(rx.try_recv().is_err());
    }

    /// The replace-on-re-register helper must drop exactly its own previous
    /// registration and leave user registrations of the same type untouched,
    /// preserving the documented semantics of the wrapper APIs.
    #[test]
    fn replacing_registration_replaces_only_its_own() {
        let inner = test_inner();
        let (tx, rx) = mpsc::channel();
        let _user =
            unsafe { register_callback::<TestCallback, _>(&inner, tag_receiver(tx.clone(), 0)) };
        unsafe {
            register_replacing_callback::<TestCallback, _>(&inner, tag_receiver(tx.clone(), 1))
        };
        unsafe { register_replacing_callback::<TestCallback, _>(&inner, tag_receiver(tx, 2)) };
        assert_eq!(registered_count(&inner, TestCallback::ID), 2);
        dispatch_test_callback(&inner);
        let mut got = Vec::new();
        while let Ok(v) = rx.try_recv() {
            got.push(v);
        }
        // user first, then the surviving (second) replacing registration; the
        // replaced first registration must not have fired.
        assert_eq!(got, vec![0, 2]);
    }
}
