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

pub type SResult<T> = Result<T, SteamError>;

pub type SIResult<T> = Result<T, SteamAPIInitError>;

pub(crate) fn to_steam_result(result: sys::EResult) -> SResult<()> {
    if result == sys::EResult::k_EResultOK {
        Ok(())
    } else {
        Err(result.into())
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
    callbacks: Mutex<HashMap<i32, Box<dyn FnMut(*mut c_void) + Send + 'static>>>,
    call_results:
        Mutex<HashMap<sys::SteamAPICall_t, Box<dyn FnOnce(*mut c_void, bool) + Send + 'static>>>,
}

impl Inner {
    /// Runs any currently pending callbacks
    ///
    /// This runs all currently pending callbacks on the current
    /// thread.
    ///
    /// This should be called frequently (e.g. once per a frame)
    /// in order to reduce the latency between recieving events.
    pub fn run_callbacks(&self) {
        self.run_callbacks_raw(|cb_discrim, data| {
            let mut callbacks = self.callbacks.callbacks.lock().unwrap();
            if let Some(cb) = callbacks.get_mut(&cb_discrim) {
                cb(data);
            }
        });
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
        self.run_callbacks_raw(|cb_discrim, data| {
            {
                let mut callbacks = self.callbacks.callbacks.lock().unwrap();
                if let Some(cb) = callbacks.get_mut(&cb_discrim) {
                    cb(data);
                }
            }
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
    pub fn init() -> SIResult<Client> {
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
    pub fn init_app<ID: Into<AppId>>(app_id: ID) -> SIResult<Client> {
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
            let apps = sys::SteamAPI_SteamApps_v008();
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
            let rp = sys::SteamAPI_SteamRemotePlay_v003();
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

    /// Returns the formatted SteamID32 string for this steam id.
    pub fn steamid32(&self) -> String {
        let account_id = self.account_id().raw();
        let last_bit = account_id & 1;
        format!("STEAM_0:{}:{}", last_bit, (account_id >> 1))
    }
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
pub struct GameId(pub(crate) u64);

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
