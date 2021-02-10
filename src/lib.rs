
use libc;
extern crate steamworks_sys as sys;
#[macro_use]
extern crate thiserror;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;

mod error;
pub use crate::error::*;

mod callback;
pub use crate::callback::*;
mod server;
pub use crate::server::*;
mod utils;
pub use crate::utils::*;
mod app;
pub use crate::app::*;
mod friends;
pub use crate::friends::*;
mod matchmaking;
pub use crate::matchmaking::*;
mod networking;
pub use crate::networking::*;
mod user;
pub use crate::user::*;
mod user_stats;
pub use crate::user_stats::*;
mod remote_storage;
pub use crate::remote_storage::*;
mod ugc;
pub use crate::ugc::*;

use std::sync::{Arc, Mutex};
use std::ffi::{CString, CStr};
use std::fmt::{
    Debug, Formatter, self
};
use std::marker::PhantomData;
use std::collections::HashMap;

#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

pub type SResult<T> = Result<T, SteamError>;

// A note about thread-safety:
// The steam api is assumed to be thread safe unless
// the documentation for a method states otherwise,
// however this is never stated anywhere in the docs
// that I could see.

/// The main entry point into the steam client.
///
/// This provides access to all of the steamworks api that
/// clients can use.
pub struct Client<Manager = ClientManager> {
    inner: Arc<Inner<Manager>>,
}

impl <Manager> Clone for Client<Manager> {
    fn clone(&self) -> Self {
        Client {
            inner: self.inner.clone(),
        }
    }
}

/// Allows access parts of the steam api that can only be called
/// on a single thread at any given time.
pub struct SingleClient<Manager = ClientManager> {
    inner: Arc<Inner<Manager>>,
    _not_sync: PhantomData<*mut ()>,
}

struct Inner<Manager> {
    _manager: Manager,
    callbacks: Mutex<Callbacks>,
}

struct Callbacks {
    callbacks: HashMap<i32, Box<dyn FnMut(*mut libc::c_void) + Send + 'static>>,
    call_results: HashMap<sys::SteamAPICall_t, Box<dyn FnOnce(*mut libc::c_void, bool) + Send + 'static>>,
}

unsafe impl <Manager: Send + Sync> Send for Inner<Manager> {}
unsafe impl <Manager: Send + Sync> Sync for Inner<Manager> {}
unsafe impl <Manager: Send + Sync> Send for Client<Manager> {}
unsafe impl <Manager: Send + Sync> Sync for Client<Manager> {}
unsafe impl <Manager: Send + Sync> Send for SingleClient<Manager> {}

/// Returns true if the app wasn't launched through steam and
/// begins relaunching it, the app should exit as soon as possible.
///
/// Returns false if the app was either launched through steam
/// or has a `steam_appid.txt`
pub fn restart_app_if_necessary(app_id: AppId) -> bool {
    unsafe {
        sys::SteamAPI_RestartAppIfNecessary(app_id.0)
    }
}

fn static_assert_send<T: Send>() {}
fn static_assert_sync<T>() where T: Sync {}

impl Client<ClientManager> {
    /// Attempts to initialize the steamworks api and returns
    /// a client to access the rest of the api.
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
    ///   working directory
    /// * The game isn't running on the same user/level as the steam client
    /// * The user doesn't own a license for the game.
    /// * The app ID isn't completely set up.
    pub fn init() -> SResult<(Client<ClientManager>, SingleClient<ClientManager>)> {
        static_assert_send::<Client<ClientManager>>();
        static_assert_sync::<Client<ClientManager>>();
        static_assert_send::<SingleClient<ClientManager>>();
        unsafe {
            if !sys::SteamAPI_Init() {
                return Err(SteamError::InitFailed);
            }
            sys::SteamAPI_ManualDispatch_Init();
            let client = Arc::new(Inner {
                _manager: ClientManager { _priv: () },
                callbacks: Mutex::new(Callbacks {
                    callbacks: HashMap::new(),
                    call_results: HashMap::new(),
                }),
            });
            Ok((Client {
                inner: client.clone(),
            }, SingleClient {
                inner: client,
                _not_sync: PhantomData,
            }))
        }
    }
}
impl <M> SingleClient<M> where M: Manager {
    /// Runs any currently pending callbacks
    ///
    /// This runs all currently pending callbacks on the current
    /// thread.
    ///
    /// This should be called frequently (e.g. once per a frame)
    /// in order to reduce the latency between recieving events.
    pub fn run_callbacks(&self) {
        unsafe {
            let pipe = M::get_pipe();
            sys::SteamAPI_ManualDispatch_RunFrame(pipe);
            let mut callback = std::mem::zeroed();
            while sys::SteamAPI_ManualDispatch_GetNextCallback(pipe, &mut callback) {
                let mut callbacks = self.inner.callbacks.lock().unwrap();
                if callback.m_iCallback == sys::SteamAPICallCompleted_t_k_iCallback as i32 {
                    let apicall = &mut *(callback.m_pubParam as *mut _ as *mut sys::SteamAPICallCompleted_t);
                    let mut apicall_result = vec![0; apicall.m_cubParam as usize];
                    let mut failed = false;
                    if sys::SteamAPI_ManualDispatch_GetAPICallResult(
                        pipe, apicall.m_hAsyncCall,
                        apicall_result.as_mut_ptr() as *mut _, apicall.m_cubParam as _,
                        apicall.m_iCallback, &mut failed
                    ) {
                        if let Some(cb) = callbacks.call_results.remove(&apicall.m_hAsyncCall) {
                            cb(apicall_result.as_mut_ptr() as *mut _, failed);
                        }
                    }
                } else {
                    if let Some(cb) = callbacks.callbacks.get_mut(&callback.m_iCallback) {
                        cb(callback.m_pubParam as *mut _);
                    }
                }
                sys::SteamAPI_ManualDispatch_FreeLastCallback(pipe);
            }
        }
    }
}

impl <Manager> Client<Manager> {
    /// Registers the passed function as a callback for the
    /// given type.
    ///
    /// The callback will be run on the thread that `run_callbacks`
    /// is called when the event arrives.
    pub fn register_callback<C, F>(&self, f: F) -> CallbackHandle<Manager>
        where C: Callback,
              F: FnMut(C) + 'static + Send
    {
        unsafe {
            register_callback(&self.inner, f, false)
        }
    }

    /// Returns an accessor to the steam utils interface
    pub fn utils(&self) -> Utils<Manager> {
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
    pub fn matchmaking(&self) -> Matchmaking<Manager> {
        unsafe {
            let mm = sys::SteamAPI_SteamMatchmaking_v009();
            debug_assert!(!mm.is_null());
            Matchmaking {
                mm: mm,
                inner: self.inner.clone(),
            }
        }
    }

    /// Returns an accessor to the steam networking interface
    pub fn networking(&self) -> Networking<Manager> {
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
    pub fn apps(&self) -> Apps<Manager> {
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
    pub fn friends(&self) -> Friends<Manager> {
        unsafe {
            let friends = sys::SteamAPI_SteamFriends_v017();
            debug_assert!(!friends.is_null());
            Friends {
                friends: friends,
                inner: self.inner.clone(),
            }
        }

    }

    /// Returns an accessor to the steam user interface
    pub fn user(&self) -> User<Manager> {
        unsafe {
            let user = sys::SteamAPI_SteamUser_v021();
            debug_assert!(!user.is_null());
            User {
                user,
                _inner: self.inner.clone(),
            }
        }
    }

    /// Returns an accessor to the steam user stats interface
    pub fn user_stats(&self) -> UserStats<Manager> {
        unsafe {
            let us = sys::SteamAPI_SteamUserStats_v012();
            debug_assert!(!us.is_null());
            UserStats {
                user_stats: us,
                inner: self.inner.clone(),
            }
        }
    }

    /// Returns an accessor to the steam remote storage interface
    pub fn remote_storage(&self) -> RemoteStorage<Manager> {
        unsafe {
            let rs = sys::SteamAPI_SteamRemoteStorage_v014();
            debug_assert!(!rs.is_null());
            let util = sys::SteamAPI_SteamUtils_v010();
            debug_assert!(!util.is_null());
            RemoteStorage {
                rs,
                util,
                inner: self.inner.clone()
            }
        }
    }

    /// Returns an accessor to the steam UGC interface (steam workshop)
    pub fn ugc(&self) -> UGC<Manager> {
        unsafe {
            let ugc = sys::SteamAPI_SteamUGC_v015();
            debug_assert!(!ugc.is_null());
            UGC {
                ugc,
                inner: self.inner.clone(),
            }
        }
    }
}

/// Used to separate client and game server modes
pub unsafe trait Manager {
    unsafe fn get_pipe() -> sys::HSteamPipe;
}

/// Manages keeping the steam api active for clients
pub struct ClientManager {
    _priv: (),
}

unsafe impl Manager for ClientManager {
    unsafe fn get_pipe() -> sys::HSteamPipe {
        sys::SteamAPI_GetHSteamPipe()
    }
}

impl Drop for ClientManager {
    fn drop(&mut self) {
        unsafe {
            sys::SteamAPI_Shutdown();
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

    /// Returns the account id for this steam id
    pub fn account_id(&self) -> AccountId {
        unsafe {
            let bits = sys::CSteamID_SteamID_t {
                m_unAll64Bits: self.0,
            };
            AccountId(bits.m_comp.m_unAccountID())
        }
    }
}

/// A user's account id
#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AccountId(pub(crate) u32);

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
    use super::*;
    use serial_test_derive::serial;

    #[test]
    #[serial]
    fn basic_test() {
        let (client, single) = Client::init().unwrap();

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

        for _ in 0 .. 50 {
            single.run_callbacks();
            ::std::thread::sleep(::std::time::Duration::from_millis(100));
        }
    }
}
