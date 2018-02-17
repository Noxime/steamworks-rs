
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

use std::sync::{Arc, Mutex};
use std::ffi::{CString, CStr};
use std::borrow::Cow;
use std::fmt::{
    Debug, Formatter, self
};

// A note about thread-safety:
// The steam api is assumed to be thread safe unless
// the documentation for a method states otherwise,
// however this is never stated anywhere in the docs
// that I could see.

pub struct Client {
    inner: Arc<ClientInner>,
}

struct ClientInner {
    client: *mut sys::ISteamClient,
    pipe: sys::HSteamPipe,

    callbacks: Mutex<Vec<*mut libc::c_void>>,
}

impl Client {
    pub fn init() -> SResult<Client> {
        unsafe {
            if sys::SteamAPI_Init() == 0 {
                bail!(ErrorKind::InitFailed);
            }
            let client = sys::SteamInternal_CreateInterface(sys::STEAMCLIENT_INTERFACE_VERSION.as_ptr() as *const _);
            let client = Arc::new(ClientInner {
                client: client,
                pipe: sys::SteamAPI_ISteamClient_CreateSteamPipe(client),
                callbacks: Mutex::new(Vec::new()),
            });
            Ok(Client {
                inner: client,
            })
        }
    }

    pub fn run_callbacks(&self) {
        unsafe {
            sys::SteamAPI_RunCallbacks();
        }
    }

    pub fn register_callback<C, F>(&self, f: F)
        where C: Callback,
              F: FnMut(C) + 'static + Send + Sync
    {
        unsafe {
            let userdata = Box::into_raw(Box::new(f));

            extern "C" fn run_func<C, F>(userdata: *mut libc::c_void, param: *mut libc::c_void)
                where C: Callback,
                      F: FnMut(C) + 'static
            {
                unsafe {
                    let func: &mut F = &mut *(userdata as *mut F);
                    let param = C::from_raw(param);
                    func(param);
                }
            }
            extern "C" fn dealloc<C, F>(userdata: *mut libc::c_void)
                where C: Callback,
                      F: FnMut(C) + 'static
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
            cbs.push(ptr);
        }
    }

    pub fn utils(&self) -> Utils {
        unsafe {
            let utils = sys::SteamAPI_ISteamClient_GetISteamUtils(
                self.inner.client, self.inner.pipe,
                sys::STEAMUTILS_INTERFACE_VERSION.as_ptr() as *const _
            );
            assert!(!utils.is_null());
            Utils {
                utils: utils,
                _client: self.inner.clone(),
            }
        }
    }

    pub fn apps(&self) -> Apps {
        unsafe {
            let user = sys::SteamAPI_ISteamClient_ConnectToGlobalUser(self.inner.client, self.inner.pipe);
            let apps = sys::SteamAPI_ISteamClient_GetISteamApps(
                self.inner.client, user, self.inner.pipe,
                sys::STEAMAPPS_INTERFACE_VERSION.as_ptr() as *const _
            );
            assert!(!apps.is_null());
            Apps {
                apps: apps,
                _client: self.inner.clone(),
            }
        }
    }

    pub fn friends(&self) -> Friends {
        unsafe {
            let user = sys::SteamAPI_ISteamClient_ConnectToGlobalUser(self.inner.client, self.inner.pipe);
            let friends = sys::SteamAPI_ISteamClient_GetISteamFriends(
                self.inner.client, user, self.inner.pipe,
                sys::STEAMFRIENDS_INTERFACE_VERSION.as_ptr() as *const _
            );
            assert!(!friends.is_null());
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
            for cb in &**self.callbacks.lock().unwrap() {
                sys::unregister_rust_steam_callback(*cb);
            }
            debug_assert!(sys::SteamAPI_ISteamClient_BReleaseSteamPipe(self.client, self.pipe) != 0);
            sys::SteamAPI_Shutdown();
        }
    }
}

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
