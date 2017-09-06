
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

use std::rc::Rc;
use std::ffi::CStr;
use std::borrow::Cow;
use std::fmt::{
    Debug, Formatter, self
};

pub struct Client {
    inner: Rc<ClientInner>,
}

struct ClientInner {
    client: *mut sys::ISteamClient,
    pipe: sys::HSteamPipe,
}

impl Client {
    pub fn init() -> SResult<Client> {
        unsafe {
            if sys::SteamAPI_Init() == 0 {
                bail!(ErrorKind::InitFailed);
            }
            let client = sys::SteamInternal_CreateInterface(sys::STEAMCLIENT_INTERFACE_VERSION.as_ptr() as *const _);
            let client = Rc::new(ClientInner {
                client: client,
                pipe: sys::SteamAPI_ISteamClient_CreateSteamPipe(client),
            });
            Ok(Client {
                inner: client,
            })
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
            debug_assert!(sys::SteamAPI_ISteamClient_BReleaseSteamPipe(self.client, self.pipe) != 0);
            sys::SteamAPI_Shutdown();
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SteamId(pub(crate) u64);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic_test() {
        let client = Client::init().unwrap();

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
        let list = friends.get_friends(FRIEND_FLAG_IMMEDIATE);
        println!("{:?}", list);
        for f in &list {
            println!("Friend: {:?} - {}({:?})", f.id(), f.name(), f.state());
        }
    }
}
