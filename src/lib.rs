
extern crate libc;
extern crate steamworks_sys as sys;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate bitflags;

pub mod error;
pub use error::Result as SResult;
use error::ErrorKind;

mod utils;
pub use utils::*;
mod app;
pub use app::*;
mod friends;
pub use friends::*;
mod matchmaking;
pub use matchmaking::*;

use std::sync::{Arc, Mutex, Weak};
use std::ffi::{CString, CStr};
use std::borrow::Cow;
use std::fmt::{
    Debug, Formatter, self
};
use std::collections::HashMap;

// A note about thread-safety:
// The steam api is assumed to be thread safe unless
// the documentation for a method states otherwise,
// however this is never stated anywhere in the docs
// that I could see.

/// The main entry point into the steam client.
///
/// This provides access to all of the steamworks api.
#[derive(Clone)]
pub struct Client {
    inner: Arc<ClientInner>,
}

struct ClientInner {
    _client: *mut sys::ISteamClient,
    callbacks: Mutex<ClientCallbacks>,
}

struct ClientCallbacks {
    callbacks: Vec<*mut libc::c_void>,
    call_results: HashMap<sys::SteamAPICall, *mut libc::c_void>,
}

unsafe impl Send for ClientInner {}
unsafe impl Sync for ClientInner {}

impl Client {
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
    pub fn init() -> SResult<Client> {
        unsafe {
            if sys::SteamAPI_Init() == 0 {
                bail!(ErrorKind::InitFailed);
            }
            let client = sys::steam_rust_get_client();
            let client = Arc::new(ClientInner {
                _client: client,
                callbacks: Mutex::new(ClientCallbacks {
                    callbacks: Vec::new(),
                    call_results: HashMap::new(),
                }),
            });
            Ok(Client {
                inner: client,
            })
        }
    }

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
            let userdata = Box::into_raw(Box::new(f));

            extern "C" fn run_func<C, F>(userdata: *mut libc::c_void, param: *mut libc::c_void)
                where C: Callback,
                      F: FnMut(C) + 'static + Send + Sync
            {
                unsafe {
                    let func: &mut F = &mut *(userdata as *mut F);
                    let param = C::from_raw(param);
                    func(param);
                }
            }
            extern "C" fn dealloc<C, F>(userdata: *mut libc::c_void)
                where C: Callback,
                      F: FnMut(C) + 'static + Send + Sync
            {
                let func: Box<F> = unsafe { Box::from_raw(userdata as _) };
                drop(func);
            }

            let ptr = sys::register_rust_steam_callback(
                C::size() as _,
                userdata as _,
                run_func::<C, F>,
                dealloc::<C, F>,
                C::id() as _
            );
            let mut cbs = self.inner.callbacks.lock().unwrap();
            cbs.callbacks.push(ptr);
        }
    }

    pub(crate) unsafe fn register_call_result<C, F>(inner: &Arc<ClientInner>, api_call: sys::SteamAPICall, callback_id: i32, f: F)
        where F: for <'a> FnMut(&'a C, bool) + 'static + Send + Sync
    {
        use std::mem;

        struct Info<F> {
            func: F,
            api_call: sys::SteamAPICall,
            client: Weak<ClientInner>,
        }

        let userdata = Box::into_raw(Box::new(Info {
            func: f,
            api_call,
            client: Arc::downgrade(&inner),
        }));

        extern "C" fn run_func<C, F>(userdata: *mut libc::c_void, param: *mut libc::c_void, io_error: bool)
            where F: for <'a> FnMut(&'a C, bool) + 'static + Send + Sync
        {
            unsafe {
                let func: &mut Info<F> = &mut *(userdata as *mut Info<F>);
                (func.func)(&*(param as *const _), io_error);
            }
        }
        extern "C" fn dealloc<C, F>(userdata: *mut libc::c_void)
            where F: for <'a> FnMut(&'a C, bool) + 'static + Send + Sync
        {
            let func: Box<Info<F>> = unsafe { Box::from_raw(userdata as _) };
            if let Some(inner) = func.client.upgrade() {
                let mut cbs = inner.callbacks.lock().unwrap();
                cbs.call_results.remove(&func.api_call);
            }
            drop(func);
        }

        let ptr = sys::register_rust_steam_call_result(
            mem::size_of::<C>() as _,
            userdata as _,
            run_func::<C, F>,
            dealloc::<C, F>,
            api_call,
            callback_id as _,
        );
        let mut cbs = inner.callbacks.lock().unwrap();
        cbs.call_results.insert(api_call, ptr);
    }

    /// Returns an accessor to the steam utils interface
    pub fn utils(&self) -> Utils {
        unsafe {
            let utils = sys::steam_rust_get_utils();
            debug_assert!(!utils.is_null());
            Utils {
                utils: utils,
                _client: self.inner.clone(),
            }
        }
    }

    /// Returns an accessor to the steam matchmaking interface
    pub fn matchmaking(&self) -> Matchmaking {
        unsafe {
            let mm = sys::steam_rust_get_matchmaking();
            debug_assert!(!mm.is_null());
            Matchmaking {
                mm: mm,
                client: self.inner.clone(),
            }
        }
    }

    /// Returns an accessor to the steam apps interface
    pub fn apps(&self) -> Apps {
        unsafe {
            let apps = sys::steam_rust_get_apps();
            debug_assert!(!apps.is_null());
            Apps {
                apps: apps,
                _client: self.inner.clone(),
            }
        }
    }

    /// Returns an accessor to the steam friends interface
    pub fn friends(&self) -> Friends {
        unsafe {
            let friends = sys::steam_rust_get_friends();
            debug_assert!(!friends.is_null());
            Friends {
                friends: friends,
                _client: self.inner.clone(),
            }
        }

    }
}

impl Drop for ClientInner {
    fn drop(&mut self) {
        unsafe {
            {
                let callbacks = self.callbacks.lock().unwrap();
                for cb in &callbacks.callbacks {
                    sys::unregister_rust_steam_callback(*cb);
                }
                for cb in callbacks.call_results.values() {
                    sys::unregister_rust_steam_call_result(*cb);
                }
            }
            sys::SteamAPI_Shutdown();
        }
    }
}

/// A user's steam id
#[derive(Clone, Copy, Debug)]
pub struct SteamId(pub(crate) u64);

pub unsafe trait Callback {
    fn id() -> i32;
    fn size() -> i32;
    unsafe fn from_raw(raw: *mut libc::c_void) -> Self;
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
