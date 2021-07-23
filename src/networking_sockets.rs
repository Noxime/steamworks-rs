use super::*;
use std::convert::TryInto;
use std::net::SocketAddr;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use sys::ISteamNetworkingSockets;

/// Access to the steam networking sockets interface
pub struct NetworkingSockets<Manager> {
    pub(crate) sockets: *mut sys::ISteamNetworkingSockets,
    pub(crate) inner: Arc<Inner<Manager>>,
}

unsafe impl<T> Send for NetworkingSockets<T> {}
unsafe impl<T> Sync for NetworkingSockets<T> {}

impl<Manager> NetworkingSockets<Manager> {
    /// Creates a "server" socket that listens for clients to connect to by calling ConnectByIPAddress, over ordinary UDP (IPv4 or IPv6)
    ///
    /// You must select a specific local port to listen on and set it as the port field of the local address.
    ///
    /// Usually you will set the IP portion of the address to zero, (SteamNetworkingIPAddr::Clear()).
    /// This means that you will not bind to any particular local interface (i.e. the same as INADDR_ANY in plain socket code).
    /// Furthermore, if possible the socket will be bound in "dual stack" mode, which means that it can accept both IPv4 and IPv6 client connections.
    /// If you really do wish to bind a particular interface, then set the local address to the appropriate IPv4 or IPv6 IP.
    ///
    /// If you need to set any initial config options, pass them here.
    /// See SteamNetworkingConfigValue_t for more about why this is preferable to setting the options "immediately" after creation.
    ///
    /// When a client attempts to connect, a SteamNetConnectionStatusChangedCallback_t will be posted.
    /// The connection will be in the k_ESteamNetworkingConnectionState_Connecting state.
    pub fn create_listen_socket_ip(
        &self,
        local_address: SocketAddr,
        options: impl IntoIterator<Item = NetworkingConfigEntry>,
    ) -> Result<ListenSocket<Manager>, InvalidHandle> {
        let local_address = SteamIpAddr::from(local_address);
        let options: Vec<_> = options.into_iter().map(|x| x.into()).collect();
        let handle = unsafe {
            sys::SteamAPI_ISteamNetworkingSockets_CreateListenSocketIP(
                self.sockets,
                local_address.as_ptr(),
                options.len() as _,
                options.as_ptr(),
            )
        };
        if handle == sys::k_HSteamListenSocket_Invalid {
            Err(InvalidHandle)
        } else {
            Ok(ListenSocket::new(handle, self.sockets, self.inner.clone()))
        }
    }
    /// Creates a connection and begins talking to a "server" over UDP at the
    /// given IPv4 or IPv6 address.  The remote host must be listening with a
    /// matching call to CreateListenSocketIP on the specified port.
    ///
    /// A SteamNetConnectionStatusChangedCallback_t callback will be triggered when we start
    /// connecting, and then another one on either timeout or successful connection.
    ///
    /// If the server does not have any identity configured, then their network address
    /// will be the only identity in use.  Or, the network host may provide a platform-specific
    /// identity with or without a valid certificate to authenticate that identity.  (These
    /// details will be contained in the SteamNetConnectionStatusChangedCallback_t.)  It's
    /// up to your application to decide whether to allow the connection.
    ///
    /// By default, all connections will get basic encryption sufficient to prevent
    /// casual eavesdropping.  But note that without certificates (or a shared secret
    /// distributed through some other out-of-band mechanism), you don't have any
    /// way of knowing who is actually on the other end, and thus are vulnerable to
    /// man-in-the-middle attacks.
    ///
    /// If you need to set any initial config options, pass them here.  See
    /// SteamNetworkingConfigValue_t for more about why this is preferable to
    /// setting the options "immediately" after creation.
    pub fn connect_by_ip_address(
        &self,
        address: SocketAddr,
        options: impl IntoIterator<Item = NetworkingConfigEntry>,
    ) -> Result<NetConnection<Manager>, InvalidHandle> {
        let handle = unsafe {
            let address = SteamIpAddr::from(address);
            let options: Vec<_> = options.into_iter().map(|x| x.into()).collect();
            sys::SteamAPI_ISteamNetworkingSockets_ConnectByIPAddress(
                self.sockets,
                address.as_ptr(),
                options.len() as _,
                options.as_ptr(),
            )
        };
        if handle == sys::k_HSteamNetConnection_Invalid {
            Err(InvalidHandle)
        } else {
            let callback_handle = get_or_create_connection_callback(self.inner.clone());
            Ok(NetConnection::new_independent(
                handle,
                self.sockets,
                self.inner.clone(),
            ))
        }
    }

    /// Like CreateListenSocketIP, but clients will connect using ConnectP2P.
    ///
    /// nLocalVirtualPort specifies how clients can connect to this socket using
    /// ConnectP2P.  It's very common for applications to only have one listening socket;
    /// in that case, use zero.  If you need to open multiple listen sockets and have clients
    /// be able to connect to one or the other, then nLocalVirtualPort should be a small
    /// integer (<1000) unique to each listen socket you create.
    ///
    /// If you use this, you probably want to call ISteamNetworkingUtils::InitRelayNetworkAccess()
    /// when your app initializes.
    ///
    /// If you are listening on a dedicated servers in known data center,
    /// then you can listen using this function instead of CreateHostedDedicatedServerListenSocket,
    /// to allow clients to connect without a ticket.  Any user that owns
    /// the app and is signed into Steam will be able to attempt to connect to
    /// your server.  Also, a connection attempt may require the client to
    /// be connected to Steam, which is one more moving part that may fail.  When
    /// tickets are used, then once a ticket is obtained, a client can connect to
    /// your server even if they got disconnected from Steam or Steam is offline.
    ///
    /// If you need to set any initial config options, pass them here.  See
    /// SteamNetworkingConfigValue_t for more about why this is preferable to
    /// setting the options "immediately" after creation.
    pub fn create_listen_socket_p2p(
        &self,
        local_virtual_port: i32,
        options: impl IntoIterator<Item = NetworkingConfigEntry>,
    ) -> Result<ListenSocket<Manager>, InvalidHandle> {
        let options: Vec<_> = options.into_iter().map(|x| x.into()).collect();
        let handle = unsafe {
            sys::SteamAPI_ISteamNetworkingSockets_CreateListenSocketP2P(
                self.sockets,
                local_virtual_port as _,
                options.len() as _,
                options.as_ptr(),
            )
        };
        if handle == sys::k_HSteamListenSocket_Invalid {
            Err(InvalidHandle)
        } else {
            Ok(ListenSocket::new(handle, self.sockets, self.inner.clone()))
        }
    }

    /// Begin connecting to a peer that is identified using a platform-specific identifier.
    /// This uses the default rendezvous service, which depends on the platform and library
    /// configuration.  (E.g. on Steam, it goes through the steam backend.)
    ///
    /// If you need to set any initial config options, pass them here.  See
    /// SteamNetworkingConfigValue_t for more about why this is preferable to
    /// setting the options "immediately" after creation.
    ///
    /// To use your own signaling service, see:
    /// - ConnectP2PCustomSignaling
    /// - k_ESteamNetworkingConfig_Callback_CreateConnectionSignaling
    pub fn connect_p2p(
        &self,
        identity_remote: NetworkingIdentity,
        remote_virtual_port: i32,
        options: impl IntoIterator<Item = NetworkingConfigEntry>,
    ) -> Result<NetConnection<Manager>, InvalidHandle> {
        let handle = unsafe {
            let options: Vec<_> = options.into_iter().map(|x| x.into()).collect();
            sys::SteamAPI_ISteamNetworkingSockets_ConnectP2P(
                self.sockets,
                identity_remote.as_ptr(),
                remote_virtual_port as _,
                options.len() as _,
                options.as_ptr(),
            )
        };
        if handle == sys::k_HSteamNetConnection_Invalid {
            Err(InvalidHandle)
        } else {
            Ok(NetConnection::new_independent(
                handle,
                self.sockets,
                self.inner.clone(),
            ))
        }
    }

    /// Create a listen socket on the specified virtual port.  The physical UDP port to use
    /// will be determined by the SDR_LISTEN_PORT environment variable.  If a UDP port is not
    /// configured, this call will fail.
    ///
    /// This call MUST be made through the SteamGameServerNetworkingSockets() interface.
    ///
    /// This function should be used when you are using the ticket generator library
    /// to issue your own tickets.  Clients connecting to the server on this virtual
    /// port will need a ticket, and they must connect using ConnectToHostedDedicatedServer.
    ///
    /// If you need to set any initial config options, pass them here.  See
    /// SteamNetworkingConfigValue_t for more about why this is preferable to
    /// setting the options "immediately" after creation.
    pub fn create_hosted_dedicated_server_listen_socket(
        &self,
        local_virtual_port: u32,
        options: impl IntoIterator<Item = NetworkingConfigEntry>,
    ) -> Result<ListenSocket<Manager>, InvalidHandle> {
        let options: Vec<_> = options.into_iter().map(|x| x.into()).collect();
        let handle = unsafe {
            sys::SteamAPI_ISteamNetworkingSockets_CreateHostedDedicatedServerListenSocket(
                self.sockets,
                local_virtual_port as _,
                options.len() as _,
                options.as_ptr(),
            )
        };
        if handle == sys::k_HSteamListenSocket_Invalid {
            Err(InvalidHandle)
        } else {
            Ok(ListenSocket::new(handle, self.sockets, self.inner.clone()))
        }
    }

    /// Indicate our desire to be ready participate in authenticated communications.
    /// If we are currently not ready, then steps will be taken to obtain the necessary
    /// certificates.   (This includes a certificate for us, as well as any CA certificates
    /// needed to authenticate peers.)
    ///
    /// You can call this at program init time if you know that you are going to
    /// be making authenticated connections, so that we will be ready immediately when
    /// those connections are attempted.  (Note that essentially all connections require
    /// authentication, with the exception of ordinary UDP connections with authentication
    /// disabled using k_ESteamNetworkingConfig_IP_AllowWithoutAuth.)  If you don't call
    /// this function, we will wait until a feature is utilized that that necessitates
    /// these resources.
    ///
    /// You can also call this function to force a retry, if failure has occurred.
    /// Once we make an attempt and fail, we will not automatically retry.
    /// In this respect, the behavior of the system after trying and failing is the same
    /// as before the first attempt: attempting authenticated communication or calling
    /// this function will call the system to attempt to acquire the necessary resources.
    ///
    /// You can use GetAuthenticationStatus or listen for SteamNetAuthenticationStatus_t
    /// to monitor the status.
    ///
    /// Returns the current value that would be returned from GetAuthenticationStatus.
    pub fn init_authentication(
        &self,
    ) -> Result<NetworkingAvailability, NetworkingAvailabilityError> {
        unsafe { sys::SteamAPI_ISteamNetworkingSockets_InitAuthentication(self.sockets).try_into() }
    }

    /// Create a new poll group.
    ///
    /// You should destroy the poll group when you are done using DestroyPollGroup
    pub fn create_poll_group(&self) -> NetPollGroup<Manager> {
        let poll_group =
            unsafe { sys::SteamAPI_ISteamNetworkingSockets_CreatePollGroup(self.sockets) };
        NetPollGroup {
            handle: poll_group,
            sockets: self.sockets,
            inner: self.inner.clone(),
        }
    }
}

/// A socket that will continually listen for client connections.
/// Call `events()` to receive incoming connection.
/// You should regularly check for events and answer `ConnectionRequests` requests immediately or the socket will
/// appear as unresponsive to the client.
///
/// If a Listen Socket goes out of scope while there are still connections, but new requests will be rejected immediately.
pub struct ListenSocket<Manager> {
    inner: Arc<InnerSocket<Manager>>,
    callback_handle: Arc<CallbackHandle<Manager>>,
    receiver: Receiver<ListenSocketEvent<Manager>>,
}

unsafe impl<Manager: Send + Sync> Send for ListenSocket<Manager> {}
unsafe impl<Manager: Send + Sync> Sync for ListenSocket<Manager> {}

impl<Manager> ListenSocket<Manager> {
    pub fn new(
        handle: sys::HSteamListenSocket,
        sockets: *mut sys::ISteamNetworkingSockets,
        inner: Arc<Inner<Manager>>,
    ) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        let inner_socket = Arc::new( InnerSocket {
            sockets,
            handle,
            inner,
        });
        inner
            .networking_sockets_data
            .lock()
            .unwrap()
            .sockets
            .insert(handle, (Arc::downgrade(&inner_socket), sender));
        let callback_handle = get_or_create_connection_callback(inner.clone());
        ListenSocket {
            inner: inner_socket,
            callback_handle,
            receiver,
        }
    }

    /// Send one or more messages without copying the message payload.
    /// This is the most efficient way to send messages. To use this
    /// function, you must first allocate a message object using
    /// ISteamNetworkingUtils::AllocateMessage.  (Do not declare one
    /// on the stack or allocate your own.)
    ///
    /// You should fill in the message payload.  You can either let
    /// it allocate the buffer for you and then fill in the payload,
    /// or if you already have a buffer allocated, you can just point
    /// m_pData at your buffer and set the callback to the appropriate function
    /// to free it.  Note that if you use your own buffer, it MUST remain valid
    /// until the callback is executed.  And also note that your callback can be
    /// invoked at ant time from any thread (perhaps even before SendMessages
    /// returns!), so it MUST be fast and threadsafe.
    ///
    /// You MUST also fill in:
    /// - m_conn - the handle of the connection to send the message to
    /// - m_nFlags - bitmask of k_nSteamNetworkingSend_xxx flags.
    ///
    /// All other fields are currently reserved and should not be modified.
    ///
    /// The library will take ownership of the message structures.  They may
    /// be modified or become invalid at any time, so you must not read them
    /// after passing them to this function.
    ///
    /// Returns the message number or Steam error for each sent message.
    pub fn send_messages(
        &self,
        messages: impl IntoIterator<Item = NetworkingMessage<Manager>>,
    ) -> Vec<SResult<MessageNumber>> {
        let messages: Vec<_> = messages.into_iter().map(|x| x.message).collect();
        let mut results = Vec::with_capacity(messages.len());
        unsafe {
            sys::SteamAPI_ISteamNetworkingSockets_SendMessages(
                self.inner.sockets,
                messages.len() as _,
                messages.as_ptr(),
                results.as_mut_ptr(),
            );
            // Error codes are returned as negative numbers, while positive numbers are message numbers
            results
                .into_iter()
                .map(|x| {
                    if x >= 0 {
                        Ok(MessageNumber(x))
                    } else {
                        Err((-x).try_into().expect("invalid error code"))
                    }
                })
                .collect()
        }
    }
}

/// Inner struct that keeps sockets alive as long as there is still a connection alive
pub(crate) struct InnerSocket<Manager> {
    pub(crate) sockets: *mut sys::ISteamNetworkingSockets,
    pub(crate) handle: sys::HSteamListenSocket,
    pub(crate) inner: Arc<Inner<Manager>>,
}

impl<Manager> Drop for InnerSocket<Manager> {
    fn drop(&mut self) {
        // There's no documentation for this return value, so it's most likely false when hSocket is invalid
        // The handle should always be valid in our case.
        let _was_successful = unsafe {
            sys::SteamAPI_ISteamNetworkingSockets_CloseListenSocket(self.sockets, self.handle)
        };

        self.inner
            .networking_sockets_data
            .lock()
            .unwrap()
            .sockets
            .remove(&self.handle)
            .expect("internal socket was removed before being dropped");
    }
}

pub struct NetConnection<Manager> {
    pub(crate) handle: sys::HSteamNetConnection,
    sockets: *mut sys::ISteamNetworkingSockets,
    inner: Arc<Inner<Manager>>,
    socket: Option<Arc<InnerSocket<Manager>>>,
    callback_handle: Option<Arc<CallbackHandle<Manager>>>,
}

unsafe impl<Manager: Send + Sync> Send for NetConnection<Manager> {}
unsafe impl<Manager: Send + Sync> Sync for NetConnection<Manager> {}

impl<Manager> NetConnection<Manager> {
    pub(crate) fn new(
        handle: sys::HSteamNetConnection,
        sockets: *mut sys::ISteamNetworkingSockets,
        inner: Arc<Inner<Manager>>,
        socket: Arc<InnerSocket<Manager>>,
    ) -> Self {
        NetConnection {
            handle,
            sockets,
            inner,
            socket: Some(socket),
            callback_handle: None,
        }
    }

    pub(crate) fn new_independent(
        handle: sys::HSteamNetConnection,
        sockets: *mut sys::ISteamNetworkingSockets,
        inner: Arc<Inner<Manager>>,
    ) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        inner
            .networking_sockets_data
            .lock()
            .unwrap()
            .independent_connections
            .insert(handle, sender);
        let callback = get_or_create_connection_callback(inner.clone());
        NetConnection {
            handle,
            sockets,
            inner,
            socket: None,
            callback_handle: Some(callback),
        }
    }

    /// Clear the poll group for a connection.
    ///
    /// Returns `Err(InvalidHandle)` when `connection` is invalid.
    pub fn clear_poll_group(&self) -> Result<(), InvalidHandle> {
        let was_successful = unsafe {
            sys::SteamAPI_ISteamNetworkingSockets_SetConnectionPollGroup(
                self.sockets,
                self.handle,
                sys::k_HSteamNetPollGroup_Invalid,
            )
        };

        if was_successful {
            Ok(())
        } else {
            Err(InvalidHandle)
        }
    }

    /// Accept an incoming connection that has been received on a listen socket.
    ///
    /// When a connection attempt is received (perhaps after a few basic handshake
    /// packets have been exchanged to prevent trivial spoofing), a connection interface
    /// object is created in the k_ESteamNetworkingConnectionState_Connecting state
    /// and a SteamNetConnectionStatusChangedCallback_t is posted.  At this point, your
    /// application MUST either accept or close the connection.  (It may not ignore it.)
    /// Accepting the connection will transition it either into the connected state,
    /// or the finding route state, depending on the connection type.
    ///
    /// You should take action within a second or two, because accepting the connection is
    /// what actually sends the reply notifying the client that they are connected.  If you
    /// delay taking action, from the client's perspective it is the same as the network
    /// being unresponsive, and the client may timeout the connection attempt.  In other
    /// words, the client cannot distinguish between a delay caused by network problems
    /// and a delay caused by the application.
    ///
    /// This means that if your application goes for more than a few seconds without
    /// processing callbacks (for example, while loading a map), then there is a chance
    /// that a client may attempt to connect in that interval and fail due to timeout.
    ///
    /// If the application does not respond to the connection attempt in a timely manner,
    /// and we stop receiving communication from the client, the connection attempt will
    /// be timed out locally, transitioning the connection to the
    /// k_ESteamNetworkingConnectionState_ProblemDetectedLocally state.  The client may also
    /// close the connection before it is accepted, and a transition to the
    /// k_ESteamNetworkingConnectionState_ClosedByPeer is also possible depending the exact
    /// sequence of events.
    ///
    /// Returns k_EResultInvalidParam if the handle is invalid.
    /// Returns k_EResultInvalidState if the connection is not in the appropriate state.
    /// (Remember that the connection state could change in between the time that the
    /// notification being posted to the queue and when it is received by the application.)
    ///
    /// A note about connection configuration options.  If you need to set any configuration
    /// options that are common to all connections accepted through a particular listen
    /// socket, consider setting the options on the listen socket, since such options are
    /// inherited automatically.  If you really do need to set options that are connection
    /// specific, it is safe to set them on the connection before accepting the connection.
    pub fn accept_connection(&self) -> SResult<()> {
        let result = unsafe {
            sys::SteamAPI_ISteamNetworkingSockets_AcceptConnection(self.sockets, self.handle)
        };
        match result {
            sys::EResult::k_EResultOK => Ok(()),
            error => Err(error.into()),
        }
    }

    /// Disconnects from the remote host and invalidates the connection handle.
    /// Any unread data on the connection is discarded.
    ///
    /// nReason is an application defined code that will be received on the other
    /// end and recorded (when possible) in backend analytics.  The value should
    /// come from a restricted range.  (See ESteamNetConnectionEnd.)  If you don't need
    /// to communicate any information to the remote host, and do not want analytics to
    /// be able to distinguish "normal" connection terminations from "exceptional" ones,
    /// You may pass zero, in which case the generic value of
    /// k_ESteamNetConnectionEnd_App_Generic will be used.
    ///
    /// pszDebug is an optional human-readable diagnostic string that will be received
    /// by the remote host and recorded (when possible) in backend analytics.
    ///
    /// If you wish to put the socket into a "linger" state, where an attempt is made to
    /// flush any remaining sent data, use bEnableLinger=true.  Otherwise reliable data
    /// is not flushed.
    ///
    /// If the connection has already ended and you are just freeing up the
    /// connection interface, the reason code, debug string, and linger flag are
    /// ignored.
    pub fn close_connection(
        self,
        reason: NetConnectionEnd,
        debug_string: Option<&str>,
        enable_linger: bool,
    ) -> bool {
        let debug_string = debug_string.map(|x| CString::new(x).unwrap());
        let debug_string_ptr = match debug_string {
            None => std::ptr::null(),
            Some(s) => s.as_ptr(),
        };
         unsafe {
            sys::SteamAPI_ISteamNetworkingSockets_CloseConnection(
                self.sockets,
                self.handle,
                reason.into(),
                debug_string_ptr,
                enable_linger,
            )
        }
    }

    /// Fetch connection user data.  Returns -1 if handle is invalid
    /// or if you haven't set any userdata on the connection.
    pub fn connection_user_data(&self) -> Result<i64, InvalidHandle> {
        let user_data = unsafe {
            sys::SteamAPI_ISteamNetworkingSockets_GetConnectionUserData(self.sockets, self.handle)
        };
        if user_data == -1 {
            // I'm not sure if a connection can become invalid on its own, so returning a result may be unnecessary
            Err(InvalidHandle)
        } else {
            Ok(user_data)
        }
    }

    /// Set connection user data.  the data is returned in the following places
    /// - You can query it using GetConnectionUserData.
    /// - The SteamNetworkingmessage_t structure.
    /// - The SteamNetConnectionInfo_t structure.  (Which is a member of SteamNetConnectionStatusChangedCallback_t.)
    ///
    /// Returns false if the handle is invalid.
    pub fn set_connection_user_data(&self, user_data: i64) -> Result<(), InvalidHandle> {
        let was_successful = unsafe {
            sys::SteamAPI_ISteamNetworkingSockets_SetConnectionUserData(
                self.sockets,
                self.handle,
                user_data,
            )
        };
        if was_successful {
            Ok(())
        } else {
            Err(InvalidHandle)
        }
    }

    /// Set a name for the connection, used mostly for debugging
    pub fn set_connection_name(&self, name: &str) {
        let name = CString::new(name).unwrap();
        unsafe {
            sys::SteamAPI_ISteamNetworkingSockets_SetConnectionName(
                self.sockets,
                self.handle,
                name.as_ptr(),
            )
        }
    }

    /// Send a message to the remote host on the specified connection.
    ///
    /// nSendFlags determines the delivery guarantees that will be provided,
    /// when data should be buffered, etc.  E.g. k_nSteamNetworkingSend_Unreliable
    ///
    /// Note that the semantics we use for messages are not precisely
    /// the same as the semantics of a standard "stream" socket.
    /// (SOCK_STREAM)  For an ordinary stream socket, the boundaries
    /// between chunks are not considered relevant, and the sizes of
    /// the chunks of data written will not necessarily match up to
    /// the sizes of the chunks that are returned by the reads on
    /// the other end.  The remote host might read a partial chunk,
    /// or chunks might be coalesced.  For the message semantics
    /// used here, however, the sizes WILL match.  Each send call
    /// will match a successful read call on the remote host
    /// one-for-one.  If you are porting existing stream-oriented
    /// code to the semantics of reliable messages, your code should
    /// work the same, since reliable message semantics are more
    /// strict than stream semantics.  The only caveat is related to
    /// performance: there is per-message overhead to retain the
    /// message sizes, and so if your code sends many small chunks
    /// of data, performance will suffer. Any code based on stream
    /// sockets that does not write excessively small chunks will
    /// work without any changes.
    ///
    /// The pOutMessageNumber is an optional pointer to receive the
    /// message number assigned to the message, if sending was successful.
    ///
    /// Returns:
    /// - k_EResultInvalidParam: invalid connection handle, or the individual message is too big.
    ///   (See k_cbMaxSteamNetworkingSocketsMessageSizeSend)
    /// - k_EResultInvalidState: connection is in an invalid state
    /// - k_EResultNoConnection: connection has ended
    /// - k_EResultIgnored: You used k_nSteamNetworkingSend_NoDelay, and the message was dropped because
    ///   we were not ready to send it.
    /// - k_EResultLimitExceeded: there was already too much data queued to be sent.
    ///   (See k_ESteamNetworkingConfig_SendBufferSize)
    pub fn send_message(&self, data: &[u8], send_flags: SendFlags) -> SResult<MessageNumber> {
        unsafe {
            let mut out_message_number = 0i64;
            let result = sys::SteamAPI_ISteamNetworkingSockets_SendMessageToConnection(
                self.sockets,
                self.handle,
                data.as_ptr() as _,
                data.len() as _,
                send_flags.bits(),
                &mut out_message_number,
            );
            match result {
                sys::EResult::k_EResultOK => Ok(MessageNumber(out_message_number)),
                error => Err(error.into()),
            }
        }
    }

    /// Fetch connection name.  Returns false if handle is invalid
    pub fn connection_name(&self) -> Result<(), InvalidHandle> {
        unimplemented!()
    }

    /// Flush any messages waiting on the Nagle timer and send them
    /// at the next transmission opportunity (often that means right now).
    ///
    /// If Nagle is enabled (it's on by default) then when calling
    /// SendMessageToConnection the message will be buffered, up to the Nagle time
    /// before being sent, to merge small messages into the same packet.
    /// (See k_ESteamNetworkingConfig_NagleTime)
    ///
    /// Returns:
    /// k_EResultInvalidParam: invalid connection handle
    /// k_EResultInvalidState: connection is in an invalid state
    /// k_EResultNoConnection: connection has ended
    /// k_EResultIgnored: We weren't (yet) connected, so this operation has no effect.
    pub fn flush_messages(&self) -> SResult<()> {
        unsafe {
            let result = sys::SteamAPI_ISteamNetworkingSockets_FlushMessagesOnConnection(
                self.sockets,
                self.handle,
            );
            if let sys::EResult::k_EResultOK = result {
                Ok(())
            } else {
                Err(result.into())
            }
        }
    }

    /// Fetch the next available message(s) from the connection, if any.
    /// Returns the number of messages returned into your array, up to nMaxMessages.
    /// If the connection handle is invalid, -1 is returned.
    ///
    /// The order of the messages returned in the array is relevant.
    /// Reliable messages will be received in the order they were sent (and with the
    /// same sizes --- see SendMessageToConnection for on this subtle difference from a stream socket).
    ///
    /// Unreliable messages may be dropped, or delivered out of order with respect to
    /// each other or with respect to reliable messages.  The same unreliable message
    /// may be received multiple times.
    ///
    /// If any messages are returned, you MUST call SteamNetworkingMessage_t::Release() on each
    /// of them free up resources after you are done.  It is safe to keep the object alive for
    /// a little while (put it into some queue, etc), and you may call Release() from any thread.
    pub fn receive_messages(&self, batch_size: usize) -> Vec<NetworkingMessage<Manager>> {
        // TODO: Optionally make it possible to reuse the same buffer to avoid allocation
        let mut buffer = Vec::with_capacity(batch_size);
        unsafe {
            let message_count = sys::SteamAPI_ISteamNetworkingSockets_ReceiveMessagesOnConnection(
                self.sockets,
                self.handle,
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

    /// Assign a connection to a poll group.  Note that a connection may only belong to a
    /// single poll group.  Adding a connection to a poll group implicitly removes it from
    /// any other poll group it is in.
    ///
    /// You can call `clear_connection_poll_group` to remove a connection from its current
    /// poll group without adding it to a new poll group.
    ///
    /// If there are received messages currently pending on the connection, an attempt
    /// is made to add them to the queue of messages for the poll group in approximately
    /// the order that would have applied if the connection was already part of the poll
    /// group at the time that the messages were received.
    ///
    /// Returns false if the connection handle is invalid, or if the poll group handle
    /// is invalid (and not k_HSteamNetPollGroup_Invalid).
    pub fn set_poll_group(&self, poll_group: &NetPollGroup<Manager>) -> Result<(), InvalidHandle> {
        let was_successful = unsafe {
            sys::SteamAPI_ISteamNetworkingSockets_SetConnectionPollGroup(
                self.sockets,
                self.handle,
                poll_group.handle,
            )
        };
        if was_successful {
            Ok(())
        } else {
            Err(InvalidHandle)
        }
    }
}

impl<Manager> Drop for NetConnection<Manager> {
    fn drop(&mut self) {
        let debug_string = CString::new("Handle was dropped").unwrap();
        let _was_successful = unsafe {
            sys::SteamAPI_ISteamNetworkingSockets_CloseConnection(
                self.sockets,
                self.handle,
                NetConnectionEnd::AppGeneric.into(),
                debug_string.as_ptr(),
                false,
            )
        };

        if self.socket.is_none() {
            self.inner
                .networking_sockets_data
                .lock()
                .unwrap()
                .independent_connections
                .remove(&self.handle)
                .expect("internal connection was removed before being dropped");
        }
    }
}
/// All independent connections (to a remote host) and listening sockets share the same Callback for
/// `NetConnectionStatusChangedCallback`. This function either returns the existing handle, or creates a new
/// handler.
fn get_or_create_connection_callback<Manager>(
    inner: Arc<Inner<Manager>>,
    sockets: *mut ISteamNetworkingSockets
) -> Arc<CallbackHandle<Manager>> {
    if let Some(callback) = inner
        .networking_sockets_data
        .lock()
        .unwrap()
        .connection_callback
        .upgrade()
    {
        callback
    } else {
        let callback = unsafe {
            register_callback(&inner, |event: NetConnectionStatusChanged| {
                // Using sockets here is safe, because the callback will be removed when all sockets and connections are
                // closed, so sockets is always valid while the socket exists.
                networking_connections_callback(inner, event, sockets)
            })
        };

        let callback = Arc::new(callback);
        inner
            .networking_sockets_data
            .lock()
            .unwrap()
            .connection_callback = Arc::downgrade(&callback);
        callback
    }
}

fn networking_connections_callback<Manager>(
    inner: Arc<Inner<Manager>>,
    event: NetConnectionStatusChanged,
    sockets: *mut ISteamNetworkingSockets,
) {
    let socket_handle;
    let sender = {
        let data = inner.networking_sockets_data.lock().unwrap();
        if let Some(socket) = event.connection_info.listen_socket() {
            socket_handle = Some(socket);
            data.sockets
                .get(&socket)
                .expect("received connection callback for unregistered socket")
        } else {
            socket_handle = None;
            data.independent_connections
                .get(&event.connection)
                .expect("received connection callback for unregistered connection")
        }
    };
    let connection = event.connection;
    let state = event.connection_info.state();
    let event = event.into_net_connection_event();
    if let Err(_err) = sender.send(event) {
        // If the main socket was dropped, but the inner socket still exists, reject all new connections,
        // as there's no way to accept them.
        if let (Some(_handle), Ok(NetworkingConnectionState::Connecting)) = (socket_handle, state) {
            // Always use the ::new() functions to create new connections, this isn't a valid connection.
            // But in this case it's enough to close the connection.
            NetConnection {
                handle: connection,
                sockets,
                inner: inner.clone(),
                socket: None,
                callback_handle: None,
            }
            .close_connection(
                NetConnectionEnd::AppGeneric,
                Some("no new connections will be accepted"),
                false,
            );
        }
    }
}

pub struct NetPollGroup<Manager> {
    handle: sys::HSteamNetPollGroup,
    sockets: *mut sys::ISteamNetworkingSockets,
    inner: Arc<Inner<Manager>>,
}

unsafe impl<Manager: Send + Sync> Send for NetPollGroup<Manager> {}
unsafe impl<Manager: Send + Sync> Sync for NetPollGroup<Manager> {}

impl<Manager> NetPollGroup<Manager> {
    pub fn receive_messages(&self, batch_size: usize) -> Vec<NetworkingMessage<Manager>> {
        let mut buffer = Vec::with_capacity(batch_size);
        unsafe {
            let count = sys::SteamAPI_ISteamNetworkingSockets_ReceiveMessagesOnPollGroup(
                self.sockets,
                self.handle,
                buffer.as_mut_ptr(),
                batch_size as _,
            ) as usize;
            buffer.set_len(count);
        }
        buffer
            .into_iter()
            .map(|x| NetworkingMessage {
                message: x,
                _inner: self.inner.clone(),
            })
            .collect()
    }
}

impl<Manager> Drop for NetPollGroup<Manager> {
    fn drop(&mut self) {
        let _was_successful = unsafe {
            sys::SteamAPI_ISteamNetworkingSockets_DestroyPollGroup(self.sockets, self.handle)
        };
    }
}

#[derive(Debug, Error)]
#[error("operation was unsuccessful an invalid handle was returned")]
pub struct InvalidHandle;

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_create_listen_socket_ip() {
        let (client, _single) = Client::init().unwrap();
        let sockets = client.networking_sockets();
        let socket_result = sockets.create_listen_socket_ip(
            SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), 12345),
            vec![],
        );
        assert!(socket_result.is_ok());
    }

    #[test]
    fn test_socket_connection() {
        // let (client, single) = Client::init().unwrap();
        // let sockets = Arc::new(client.networking_sockets());
        //
        // let poll_group = sockets.create_poll_group();
        //
        // // Create a callback that accepts the new connection and sends a message back immediately
        // let callback_sockets = sockets.clone();
        // let _connection_changed = client.register_callback(move |p: NetConnectionStatusChanged| {
        //     if let NetworkingConnectionState::Connecting = p.connection_info.state() {
        //         println!(
        //             "New connection to {:?}",
        //             p.connection_info.identity_remote()
        //         );
        //         if p.connection_info.identity_remote().is_valid() {
        //             let sockets = &callback_sockets;
        //
        //             sockets.accept_connection(&p.connection).unwrap();
        //             sockets
        //                 .set_connection_poll_group(&p.connection, &poll_group)
        //                 .unwrap();
        //             println!(
        //                 "Accepting connection to {:?}",
        //                 p.connection_info.identity_remote()
        //             );
        //
        //             // Send message back
        //             sockets
        //                 .send_message_to_connection(
        //                     &p.connection,
        //                     &[0, 0, 1, 2],
        //                     SendFlags::RELIABLE_NO_NAGLE,
        //                 )
        //                 .unwrap();
        //         } else {
        //             println!(
        //                 "Invalid connection, probably the event for creating the listening socket"
        //             )
        //         }
        //     } else {
        //         println!("Callback: {:?}", p.connection_info.state())
        //     }
        // });
        //
        // println!("Create listening socket");
        // let _socket = sockets
        //     .create_listen_socket_ip(
        //         SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), 12345),
        //         vec![],
        //     )
        //     .expect("Listen socket creation failed");
        //
        // println!("Create client connection");
        // let connection = sockets
        //     .connect_by_ip_address(
        //         SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), 12345),
        //         vec![],
        //     )
        //     .expect("Starting connection failed");
        //
        // println!("Run callbacks");
        // for _ in 0..10 {
        //     single.run_callbacks();
        //     ::std::thread::sleep(::std::time::Duration::from_millis(100));
        // }
        //
        // // Send from the client to the server
        // sockets
        //     .send_message_to_connection(&connection, &[1, 2, 3, 4], SendFlags::RELIABLE_NO_NAGLE)
        //     .unwrap();
        //
        // ::std::thread::sleep(::std::time::Duration::from_millis(100));
        //
        // // Receive message on the server
        // let messages = sockets.receive_messaged_on_poll_group(&poll_group, 1);
        // assert_eq!(messages.len(), 1);
        // assert_eq!(messages[0].data(), &[1, 2, 3, 4]);
        //
        // // Receive message on the client (the one we sent in the callback)
        // let messages = sockets.receive_messages_on_connection(connection, 1);
        // assert_eq!(messages.len(), 1);
        // assert_eq!(messages[0].data(), &[0, 0, 1, 2]);
    }
}
