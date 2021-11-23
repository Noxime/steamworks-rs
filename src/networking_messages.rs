//! The non-connection-oriented interface to send and receive messages
//! (whether they be "clients" or "servers").
//!
//! ISteamNetworkingSockets is connection-oriented (like TCP), meaning you
//! need to listen and connect, and then you send messages using a connection
//! handle.  ISteamNetworkingMessages is more like UDP, in that you can just send
//! messages to arbitrary peers at any time.  The underlying connections are
//! established implicitly.
//!
//! Under the hood ISteamNetworkingMessages works on top of the ISteamNetworkingSockets
//! code, so you get the same routing and messaging efficiency.  The difference is
//! mainly in your responsibility to explicitly establish a connection and
//! the type of feedback you get about the state of the connection.  Both
//! interfaces can do "P2P" communications, and both support both unreliable
//! and reliable messages, fragmentation and reassembly.
//!
//! The primary purpose of this interface is to be "like UDP", so that UDP-based code
//! can be ported easily to take advantage of relayed connections.  If you find
//! yourself needing more low level information or control, or to be able to better
//! handle failure, then you probably need to use ISteamNetworkingSockets directly.
//! Also, note that if your main goal is to obtain a connection between two peers
//! without concerning yourself with assigning roles of "client" and "server",
//! you may find the symmetric connection mode of ISteamNetworkingSockets useful.
//! (See k_ESteamNetworkingConfig_SymmetricConnect.)
// TODO: examples here
use crate::networking_types::{
    NetConnectionInfo, NetworkingIdentity, NetworkingMessage, SendFlags,
};
use crate::{register_callback, Callback, Inner, SteamError};
use std::ffi::c_void;
use std::sync::{Arc, Weak};

use steamworks_sys as sys;

/// Access to the steam networking messages interface
pub struct NetworkingMessages<Manager> {
    pub(crate) net: *mut sys::ISteamNetworkingMessages,
    pub(crate) inner: Arc<Inner<Manager>>,
}

unsafe impl<Manager> Sync for NetworkingMessages<Manager> {}
unsafe impl<Manager> Send for NetworkingMessages<Manager> {}

impl<Manager: 'static> NetworkingMessages<Manager> {
    /// Sends a message to the specified host.
    ///
    /// If we don't already have a session with that user, a session is implicitly created.
    /// There might be some handshaking that needs to happen before we can actually begin sending message data.
    /// If this handshaking fails and we can't get through, an error will be posted via the callback
    /// SteamNetworkingMessagesSessionFailed_t.
    /// There is no notification when the operation succeeds.  (You should have the peer send a reply
    /// for this purpose.)
    ///
    /// Sending a message to a host will also implicitly accept any incoming connection from that host.
    ///
    /// `channel` is a routing number you can use to help route message to different systems.
    /// You'll have to call ReceiveMessagesOnChannel() with the same channel number in order to retrieve
    /// the data on the other end.
    ///
    /// Using different channels to talk to the same user will still use the same underlying
    /// connection, saving on resources.  If you don't need this feature, use 0.
    /// Otherwise, small integers are the most efficient.
    ///
    /// It is guaranteed that reliable messages to the same host on the same channel
    /// will be be received by the remote host (if they are received at all) exactly once,
    /// and in the same order that they were sent.
    ///
    /// NO other order guarantees exist!  In particular, unreliable messages may be dropped,
    /// received out of order with respect to each other and with respect to reliable data,
    /// or may be received multiple times.  Messages on different channels are *not* guaranteed
    /// to be received in the order they were sent.
    ///
    /// A note for those familiar with TCP/IP ports, or converting an existing codebase that
    /// opened multiple sockets:  You might notice that there is only one channel, and with
    /// TCP/IP each endpoint has a port number.  You can think of the channel number as the
    /// *destination* port.  If you need each message to also include a "source port" (so the
    /// recipient can route the reply), then just put that in your message.  That is essentially
    /// how UDP works!
    ///
    /// Returns:
    /// - k_EREsultOK on success.
    /// - k_EResultNoConnection will be returned if the session has failed or was closed by the peer,
    ///   and k_nSteamNetworkingSend_AutoRestartBrokenSession is not used.  (You can use
    ///   GetSessionConnectionInfo to get the details.)  In order to acknowledge the broken session
    ///   and start a new one, you must call CloseSessionWithUser
    /// - See ISteamNetworkingSockets::SendMessageToConnection for more possible return values
    pub fn send_message_to_user(
        &self,
        user: NetworkingIdentity,
        send_type: SendFlags,
        data: &[u8],
        channel: u32,
    ) -> Result<(), SteamError> {
        let result = unsafe {
            sys::SteamAPI_ISteamNetworkingMessages_SendMessageToUser(
                self.net,
                user.as_ptr(),
                data.as_ptr() as *const c_void,
                data.len() as u32,
                send_type.bits(),
                channel as i32,
            )
        };

        if result == sys::EResult::k_EResultOK {
            return Ok(());
        }

        Err(result.into())
    }

    /// Reads the next message that has been sent from another user on the given channel.
    ///
    /// `batch_size` is the maximum number of messages that can be received at once.
    ///
    /// # Example
    /// ```
    /// # use steamworks::Client;
    /// # use std::time::Duration;
    /// let (client, single) = Client::init().unwrap();
    ///
    /// // run_callbacks must be called regularly, or no incoming connections can be received
    /// let callback_loop = std::thread::spawn(move || loop {
    ///     single.run_callbacks();
    ///     std::thread::sleep(Duration::from_millis(10));
    /// });
    /// let networking_messages = client.networking_messages();
    ///
    /// // Accept all new connections
    /// networking_messages.session_request_callback(|request| request.accept());
    ///
    /// let _received = networking_messages.receive_messages_on_channel(0, 10);
    /// ```
    pub fn receive_messages_on_channel(
        &self,
        channel: u32,
        batch_size: usize,
    ) -> Vec<NetworkingMessage<Manager>> {
        let mut buffer = Vec::with_capacity(batch_size);
        unsafe {
            let message_count = sys::SteamAPI_ISteamNetworkingMessages_ReceiveMessagesOnChannel(
                self.net,
                channel as i32,
                buffer.as_mut_ptr(),
                batch_size as _,
            );
            buffer.set_len(message_count as usize);
        }

        buffer
            .into_iter()
            .map(|x| NetworkingMessage {
                message: x,
                _inner: self.inner.clone(),
            })
            .collect()
    }

    /// Register a callback that will be called whenever a peer requests a connection.
    ///
    /// Use the [`SessionRequest`](../networking_messages/struct.SessionRequest.html) to accept or reject the connection.
    ///
    /// Requires regularly calling [`SingleClient.run_callbacks()`](../struct.SingleClient.html#method.run_callbacks).
    /// Calling this function more than once will replace the previous callback.
    ///
    /// # Example
    /// ```
    /// # use steamworks::Client;
    /// # use std::time::Duration;
    /// let (client, single) = Client::init().unwrap();
    ///
    /// // run_callbacks must be called regularly, or no incoming connections can be received
    /// let callback_loop = std::thread::spawn(move || loop {
    ///     single.run_callbacks();
    ///     std::thread::sleep(Duration::from_millis(10));
    /// });
    /// let messages = client.networking_messages();
    ///
    /// // Accept all incoming connections
    /// messages.session_request_callback(|request| {
    ///     request.accept();
    /// });
    /// ```
    pub fn session_request_callback(
        &self,
        mut callback: impl FnMut(SessionRequest<Manager>) + Send + 'static,
    ) {
        let builder = SessionRequestBuilder {
            message: self.net,
            inner: Arc::downgrade(&self.inner),
        };
        unsafe {
            register_callback(
                &self.inner,
                move |request: NetworkingMessagesSessionRequest| {
                    if let Some(request) = builder.build_request(request.remote) {
                        callback(request);
                    }
                },
            );
        }
    }

    /// Register a callback that will be called whenever a connection fails to be established.
    ///
    /// Requires regularly calling [`SingleClient.run_callbacks()`](../struct.SingleClient.html#method.run_callbacks).
    /// Calling this function more than once will replace the previous callback.
    pub fn session_failed_callback(
        &self,
        mut callback: impl FnMut(NetConnectionInfo) + Send + 'static,
    ) {
        unsafe {
            register_callback(
                &self.inner,
                move |failed: NetworkingMessagesSessionFailed| {
                    callback(failed.info);
                },
            );
        }
    }
}

/// A helper for creating SessionRequests.
///
/// It's Send and Sync, so it can be moved into the callback.
struct SessionRequestBuilder<Manager> {
    message: *mut sys::ISteamNetworkingMessages,
    // Once the builder is in the callback, it creates a cyclic reference, so this has to be Weak
    inner: Weak<Inner<Manager>>,
}

unsafe impl<Manager> Sync for SessionRequestBuilder<Manager> {}
unsafe impl<Manager> Send for SessionRequestBuilder<Manager> {}

impl<Manager> SessionRequestBuilder<Manager> {
    pub fn build_request(&self, remote: NetworkingIdentity) -> Option<SessionRequest<Manager>> {
        self.inner.upgrade().map(|inner| SessionRequest {
            remote,
            messages: self.message,
            _inner: inner,
        })
    }
}

struct NetworkingMessagesSessionRequest {
    remote: NetworkingIdentity,
}

unsafe impl Callback for NetworkingMessagesSessionRequest {
    const ID: i32 = sys::SteamNetworkingMessagesSessionRequest_t_k_iCallback as _;
    const SIZE: i32 = std::mem::size_of::<sys::SteamNetworkingMessagesSessionRequest_t>() as _;

    unsafe fn from_raw(raw: *mut c_void) -> Self {
        let remote = *(raw as *mut sys::SteamNetworkingMessagesSessionRequest_t);
        let remote = remote.m_identityRemote.into();
        Self { remote }
    }
}

struct NetworkingMessagesSessionFailed {
    pub info: NetConnectionInfo,
}

unsafe impl Callback for NetworkingMessagesSessionFailed {
    const ID: i32 = sys::SteamNetworkingMessagesSessionFailed_t_k_iCallback as _;
    const SIZE: i32 = std::mem::size_of::<sys::SteamNetworkingMessagesSessionFailed_t>() as _;

    unsafe fn from_raw(raw: *mut c_void) -> Self {
        let remote = *(raw as *mut sys::SteamNetworkingMessagesSessionFailed_t);
        let remote = remote.m_info.into();
        Self { info: remote }
    }
}

/// A request for a new connection.
///
/// Use this to accept or reject the connection.
/// Letting this struct go out of scope will reject the connection.
pub struct SessionRequest<Manager> {
    remote: NetworkingIdentity,
    messages: *mut sys::ISteamNetworkingMessages,
    _inner: Arc<Inner<Manager>>,
}

unsafe impl<Manager> Sync for SessionRequest<Manager> {}
unsafe impl<Manager> Send for SessionRequest<Manager> {}

impl<Manager> SessionRequest<Manager> {
    /// The remote peer requesting the connection.
    pub fn remote(&self) -> &NetworkingIdentity {
        &self.remote
    }

    /// Accept the connection.
    pub fn accept(self) {
        unsafe {
            sys::SteamAPI_ISteamNetworkingMessages_AcceptSessionWithUser(
                self.messages,
                self.remote.as_ptr(),
            );
        }
    }

    /// Reject the connection.
    pub fn reject(mut self) {
        self.reject_inner();
    }

    /// Reject the connection without consuming self, useful for implementing [`Drop`]
    fn reject_inner(&mut self) {
        unsafe {
            sys::SteamAPI_ISteamNetworkingMessages_CloseSessionWithUser(
                self.messages,
                self.remote.as_ptr(),
            );
        }
    }
}

impl<Manager> Drop for SessionRequest<Manager> {
    fn drop(&mut self) {
        self.reject_inner();
    }
}
