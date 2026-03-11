//! An older networking solution that is now deprecated.
//!
//! In the future you should use [`networking_sockets`][../networking_sockets], but for now the wrapper for the new API
//! is still unfinished.

use super::*;

/// Access to the steam networking interface
pub struct Networking {
    pub(crate) net: *mut sys::ISteamNetworking,
    pub(crate) _inner: Arc<Inner>,
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

/// P2P session error codes
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum P2PSessionError {
    /// No error
    None = 0,
    #[deprecated(
        note = "For privacy reasons, there is no error if the remote user is playing another game."
    )]
    NotRunningApp = 1,
    /// Local user doesn't own the app that is running
    NoRightsToApp = 2,
    #[deprecated(note = "For privacy reasons, there is no error if the remote user is offline")]
    NotLoggedIn = 3,
    /// Target isn't responding, perhaps not calling AcceptP2PSessionWithUser()
    /// This may also occur behind corporate firewalls (P2P sessions require UDP ports 3478, 4379, and 4380 to be open for outgoing traffic)
    Timeout = 4,

    /// Unknown error code
    Unknown(u8),
}

impl From<u8> for P2PSessionError {
    fn from(value: u8) -> Self {
        #[allow(deprecated)]
        match value {
            0 => P2PSessionError::None,
            1 => P2PSessionError::NotRunningApp,
            2 => P2PSessionError::NoRightsToApp,
            3 => P2PSessionError::NotLoggedIn,
            4 => P2PSessionError::Timeout,
            other => P2PSessionError::Unknown(other),
        }
    }
}

impl From<P2PSessionError> for u8 {
    fn from(error: P2PSessionError) -> Self {
        #[allow(deprecated)]
        match error {
            P2PSessionError::None => 0,
            P2PSessionError::NotRunningApp => 1,
            P2PSessionError::NoRightsToApp => 2,
            P2PSessionError::NotLoggedIn => 3,
            P2PSessionError::Timeout => 4,
            P2PSessionError::Unknown(code) => code,
        }
    }
}

/// Information about a P2P session with a remote user
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct P2PSessionState {
    /// Whether we've got an active open connection
    pub connection_active: bool,
    /// Whether we're currently trying to establish a connection
    pub connecting: bool,
    /// Last error recorded
    pub error: P2PSessionError,
    /// Whether it's going through a Steam relay server
    pub using_relay: bool,
    /// Number of bytes queued for sending
    pub bytes_queued_for_send: i32,
    /// Number of packets queued for sending
    pub packets_queued_for_send: i32,
    /// Potential IP address of remote host (could be Steam relay server)
    pub remote_ip: u32,
    /// Remote port number
    pub remote_port: u16,
}

impl Networking {
    /// Accepts incoming packets from the given user
    ///
    /// Should only be called in response to a `P2PSessionRequest`.
    pub fn accept_p2p_session(&self, user: SteamId) -> bool {
        unsafe { sys::SteamAPI_ISteamNetworking_AcceptP2PSessionWithUser(self.net, user.0) }
    }

    /// Closes the p2p connection between the given user
    pub fn close_p2p_session(&self, user: SteamId) -> bool {
        unsafe { sys::SteamAPI_ISteamNetworking_CloseP2PSessionWithUser(self.net, user.0) }
    }

    /// Gets the connection state to the specified user
    ///
    /// Returns the P2P session state if a connection exists with the user,
    /// or None if no connection exists.
    pub fn get_p2p_session_state(&self, user: SteamId) -> Option<P2PSessionState> {
        unsafe {
            let mut state: sys::P2PSessionState_t = std::mem::zeroed();
            if sys::SteamAPI_ISteamNetworking_GetP2PSessionState(self.net, user.0, &mut state) {
                Some(P2PSessionState {
                    connection_active: state.m_bConnectionActive != 0,
                    connecting: state.m_bConnecting != 0,
                    error: P2PSessionError::from(state.m_eP2PSessionError),
                    using_relay: state.m_bUsingRelay != 0,
                    bytes_queued_for_send: state.m_nBytesQueuedForSend,
                    packets_queued_for_send: state.m_nPacketsQueuedForSend,
                    remote_ip: state.m_nRemoteIP,
                    remote_port: state.m_nRemotePort,
                })
            } else {
                None
            }
        }
    }

    /// Sends a packet to the user, starting the
    /// connection if it isn't started already
    pub fn send_p2p_packet(&self, remote: SteamId, send_type: SendType, data: &[u8]) -> bool {
        self.send_p2p_packet_on_channel(remote, send_type, data, 0)
    }

    /// Sends a packet to the user on a specific channel
    pub fn send_p2p_packet_on_channel(
        &self,
        remote: SteamId,
        send_type: SendType,
        data: &[u8],
        channel: i32,
    ) -> bool {
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
                data.as_ptr().cast(),
                data.len() as u32,
                send_type,
                channel,
            )
        }
    }

    /// Returns whether there is a packet queued that can be read.
    ///
    /// Returns the size of the queued packet if any.
    pub fn is_p2p_packet_available(&self) -> Option<usize> {
        self.is_p2p_packet_available_on_channel(0)
    }

    /// Returns whether there is a packet available on a specific channel
    pub fn is_p2p_packet_available_on_channel(&self, channel: i32) -> Option<usize> {
        unsafe {
            let mut size = 0;
            if sys::SteamAPI_ISteamNetworking_IsP2PPacketAvailable(self.net, &mut size, channel) {
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
        self.read_p2p_packet_from_channel(buf, 0)
    }

    /// Attempts to read a queued packet into the buffer
    /// from a specific channel
    pub fn read_p2p_packet_from_channel(
        &self,
        buf: &mut [u8],
        channel: i32,
    ) -> Option<(SteamId, usize)> {
        unsafe {
            let mut size = 0;
            let mut remote = 0;
            if sys::SteamAPI_ISteamNetworking_ReadP2PPacket(
                self.net,
                buf.as_mut_ptr().cast(),
                buf.len() as _,
                &mut size,
                &mut remote as *mut _ as *mut _,
                channel,
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

impl_callback!(cb: P2PSessionRequest_t => P2PSessionRequest {
    Self {
        remote: SteamId(cb.m_steamIDRemote.m_steamid.m_unAll64Bits),
    }
});

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct P2PSessionConnectFail {
    pub remote: SteamId,
    pub error: u8,
}

impl_callback!(cb: P2PSessionConnectFail_t => P2PSessionConnectFail {
    Self {
        remote: SteamId(cb.m_steamIDRemote.m_steamid.m_unAll64Bits),
        error: cb.m_eP2PSessionError,
    }
});
