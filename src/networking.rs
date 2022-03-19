//! An older networking solution that is now deprecated.
//!
//! In the future you should use [`networking_sockets`][../networking_sockets], but for now the wrapper for the new API
//! is still unfinished.

use super::*;

/// Access to the steam networking interface
pub struct Networking<Manager> {
    pub(crate) net: *mut sys::ISteamNetworking,
    pub(crate) _inner: Arc<Inner<Manager>>,
}

/// The method used to send a packet
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SendType {
    /// Send the packet directly over udp.
    ///
    /// Can't be larger than 1200 bytes
    Unreliable,
    /// Like `Unreliable` but doesn't buffer packets
    /// sent before the connection has started.
    UnreliableNoDelay,
    /// Reliable packet sending.
    ///
    /// Can't be larger than 1 megabyte.
    Reliable,
    /// Like `Reliable` but applies the nagle
    /// algorithm to packets being sent
    ReliableWithBuffering,
}

impl<Manager> Networking<Manager> {
    /// Accepts incoming packets from the given user
    ///
    /// Should only be called in response to a `P2PSessionRequest`.
    pub fn accept_p2p_session(&self, user: SteamId) {
        unsafe {
            sys::SteamAPI_ISteamNetworking_AcceptP2PSessionWithUser(self.net, user.0);
        }
    }

    /// Closes the p2p connection between the given user
    pub fn close_p2p_session(&self, user: SteamId) {
        unsafe {
            sys::SteamAPI_ISteamNetworking_CloseP2PSessionWithUser(self.net, user.0);
        }
    }

    /// Sends a packet to the start user starting the
    /// connection if it isn't started already
    pub fn send_p2p_packet(&self, remote: SteamId, send_type: SendType, data: &[u8]) -> bool {
        unsafe {
            let send_type = match send_type {
                SendType::Unreliable => sys::EP2PSend::k_EP2PSendUnreliable,
                SendType::UnreliableNoDelay => sys::EP2PSend::k_EP2PSendUnreliableNoDelay,
                SendType::Reliable => sys::EP2PSend::k_EP2PSendReliable,
                SendType::ReliableWithBuffering => sys::EP2PSend::k_EP2PSendReliableWithBuffering,
            };
            sys::SteamAPI_ISteamNetworking_SendP2PPacket(
                self.net,
                remote.0,
                data.as_ptr() as *const _,
                data.len() as u32,
                send_type,
                0,
            )
        }
    }

    /// Returns whether there is a packet queued that can be read.
    ///
    /// Returns the size of the queued packet if any.
    pub fn is_p2p_packet_available(&self) -> Option<usize> {
        unsafe {
            let mut size = 0;
            if sys::SteamAPI_ISteamNetworking_IsP2PPacketAvailable(self.net, &mut size, 0) {
                Some(size as usize)
            } else {
                None
            }
        }
    }

    /// Attempts to read a queued packet into the buffer
    /// if there are any.
    ///
    /// Returns the steam id of the sender and the size of the
    /// packet.
    pub fn read_p2p_packet(&self, buf: &mut [u8]) -> Option<(SteamId, usize)> {
        unsafe {
            let mut size = 0;
            let mut remote = 0;
            if sys::SteamAPI_ISteamNetworking_ReadP2PPacket(
                self.net,
                buf.as_mut_ptr() as *mut _,
                buf.len() as _,
                &mut size,
                &mut remote as *mut _ as *mut _,
                0,
            ) {
                Some((SteamId(remote), size as usize))
            } else {
                None
            }
        }
    }
}

/// Called when a user wants to communicate via p2p
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct P2PSessionRequest {
    /// The steam ID of the user requesting a p2p
    /// session
    pub remote: SteamId,
}

unsafe impl Callback for P2PSessionRequest {
    const ID: i32 = 1202;
    const SIZE: i32 = ::std::mem::size_of::<sys::P2PSessionRequest_t>() as i32;

    unsafe fn from_raw(raw: *mut c_void) -> Self {
        let val = &mut *(raw as *mut sys::P2PSessionRequest_t);
        P2PSessionRequest {
            remote: SteamId(val.m_steamIDRemote.m_steamid.m_unAll64Bits),
        }
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct P2PSessionConnectFail {
    pub remote: SteamId,
    pub error: u8,
}

unsafe impl Callback for P2PSessionConnectFail {
    const ID: i32 = 1203;
    const SIZE: i32 = ::std::mem::size_of::<sys::P2PSessionConnectFail_t>() as i32;

    unsafe fn from_raw(raw: *mut c_void) -> Self {
        let val = &mut *(raw as *mut sys::P2PSessionConnectFail_t);
        P2PSessionConnectFail {
            remote: SteamId(val.m_steamIDRemote.m_steamid.m_unAll64Bits),
            error: val.m_eP2PSessionError,
        }
    }
}
