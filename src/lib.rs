
extern crate libc;
extern crate steamworks_sys as sys;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate bitflags;

mod error;
pub use error::*;

mod callback;
pub use callback::*;
mod server;
pub use server::*;
mod utils;
pub use utils::*;
mod app;
pub use app::*;
mod friends;
pub use friends::*;
mod matchmaking;
pub use matchmaking::*;
mod user;
pub use user::*;

use std::sync::{ Arc, Mutex };
use std::ffi::{CString, CStr};
use std::fmt::{
    Debug, Formatter, self
};
use std::collections::HashMap;

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
    client: *mut sys::ISteamClient,
}

impl <Manager> Clone for Client<Manager> {
    fn clone(&self) -> Self {
        Client {
            inner: self.inner.clone(),
            client: self.client,
        }
    }
}

struct Inner<Manager> {
    _manager: Manager,
    callbacks: Mutex<Callbacks>,
}

struct Callbacks {
    callbacks: Vec<*mut libc::c_void>,
    call_results: HashMap<sys::SteamAPICall, *mut libc::c_void>,
}

unsafe impl <Manager: Send + Sync> Send for Inner<Manager> {}
unsafe impl <Manager: Send + Sync> Sync for Inner<Manager> {}
unsafe impl <Manager: Send + Sync> Send for Client<Manager> {}
unsafe impl <Manager: Send + Sync> Sync for Client<Manager> {}

/// Returns true if the app wasn't launched through steam and
/// begins relaunching it, the app should exit as soon as possible.
///
/// Returns false if the app was either launched through steam
/// or has a `steam_appid.txt`
pub fn restart_app_if_necessary(app_id: AppId) -> bool {
    unsafe {
        sys::SteamAPI_RestartAppIfNecessary(app_id.0) != 0
    }
}

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
    pub fn init() -> SResult<Client<ClientManager>> {
        unsafe {
            if sys::SteamAPI_Init() == 0 {
                return Err(SteamError::InitFailed);
            }
            let raw_client = sys::steam_rust_get_client();
            let client = Arc::new(Inner {
                _manager: ClientManager { _priv: () },
                callbacks: Mutex::new(Callbacks {
                    callbacks: Vec::new(),
                    call_results: HashMap::new(),
                }),
            });
            Ok(Client {
                inner: client,
                client: raw_client,
            })
        }
    }
}

impl <Manager> Client<Manager> {
    /// Runs any currently pending callbacks
    ///
    /// This runs all currently pending callbacks on the current
    /// thread.
    ///
    /// This should be called frequently (e.g. once per a frame)
    /// in order to reduce the latency between recieving events.
    pub fn run_callbacks(&self) {
        unsafe {
            sys::SteamAPI_RunCallbacks();
        }
    }

    /// Registers the passed function as a callback for the
    /// given type.
    ///
    /// The callback will be run on the thread that `run_callbacks`
    /// is called when the event arrives.
    pub fn register_callback<C, F>(&self, f: F)
        where C: Callback,
              F: FnMut(C) + 'static + Send + Sync
    {
        unsafe {
            register_callback(&self.inner, f, false);
        }
    }

    /// Returns an accessor to the steam utils interface
    pub fn utils(&self) -> Utils<Manager> {
        unsafe {
            let utils = sys::steam_rust_get_utils();
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
            let mm = sys::steam_rust_get_matchmaking();
            debug_assert!(!mm.is_null());
            Matchmaking {
                mm: mm,
                inner: self.inner.clone(),
            }
        }
    }

    /// Returns an accessor to the steam apps interface
    pub fn apps(&self) -> Apps<Manager> {
        unsafe {
            let apps = sys::steam_rust_get_apps();
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
            let friends = sys::steam_rust_get_friends();
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
            let user = sys::steam_rust_get_user();
            debug_assert!(!user.is_null());
            User {
                user,
                _inner: self.inner.clone(),
            }
        }

    }
}

impl <Manager> Drop for Inner<Manager> {
    fn drop(&mut self) {
        unsafe {
            {
                let callbacks = self.callbacks.lock().unwrap();
                for cb in &callbacks.callbacks {
                    sys::delete_rust_callback(*cb);
                }
                for cb in callbacks.call_results.values() {
                    sys::delete_rust_callback(*cb);
                }
            }
        }
    }
}

/// Manages keeping the steam api active for clients
pub struct ClientManager {
    _priv: (),
}

impl Drop for ClientManager {
    fn drop(&mut self) {
        unsafe {
            sys::SteamAPI_Shutdown();
        }
    }
}

/// A user's steam id
#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq)]
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
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic_test() {
        let client = Client::init().unwrap();

        client.register_callback(|p: PersonaStateChange| {
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
            client.run_callbacks();
            ::std::thread::sleep(::std::time::Duration::from_millis(100));
        }
    }
}
