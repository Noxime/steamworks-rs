use super::*;
use sys::{EResult, SteamNetworkingIdentity};

/// Access to the steam networking messages interface
pub struct NetworkingMessages<Manager> {
    pub(crate) net: *mut sys::ISteamNetworkingMessages,
    pub(crate) _inner: Arc<Inner<Manager>>,
}

impl<Manager> NetworkingMessages<Manager> {
    pub fn accept_p2p_session(&self, user: SteamNetworkingIdentity) -> bool {
        unsafe {
            sys::SteamAPI_ISteamNetworkingMessages_AcceptSessionWithUser(
                self.net,
                &user as *const _,
            )
        }
    }

    pub fn close_p2p_session(&self, user: SteamNetworkingIdentity) -> bool {
        unsafe {
            sys::SteamAPI_ISteamNetworkingMessages_CloseSessionWithUser(self.net, &user as *const _)
        }
    }

    pub fn send_message_to_user(
        &self,
        user: SteamNetworkingIdentity,
        send_type: SendFlags,
        data: &[u8],
        channel: u32,
    ) -> Result<(), SteamError> {
        let result = unsafe {
            sys::SteamAPI_ISteamNetworkingMessages_SendMessageToUser(
                self.net,
                &user as *const _,
                data.as_ptr() as _,
                data.len() as u32,
                send_type.bits(),
                channel as i32,
            )
        };

        if result == EResult::k_EResultOK {
            return Ok(());
        }

        return Err(result.into());
    }
}
