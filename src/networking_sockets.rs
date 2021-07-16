use super::*;
use std::net::SocketAddr;
use std::sync::Arc;

/// Access to the steam networking sockets interface
pub struct NetworkingSockets<Manager> {
    pub(crate) sockets: *mut sys::ISteamNetworkingSockets,
    pub(crate) _inner: Arc<Inner<Manager>>,
}

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
    ) -> Result<ListenSocket, InvalidHandle> {
        let local_address = SteamIpAddr::from(local_address);
        let options: Vec<_> = options.into_iter().map(|x| x.into()).collect();
        let socket = unsafe {
            sys::SteamAPI_ISteamNetworkingSockets_CreateListenSocketIP(
                self.sockets,
                local_address.as_ptr(),
                options.len() as _,
                options.as_ptr(),
            )
        };
        ListenSocket::from_raw(socket)
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
    ) -> Result<NetConnection, InvalidHandle> {
        let connection = unsafe {
            let address = SteamIpAddr::from(address);
            let options: Vec<_> = options.into_iter().map(|x| x.into()).collect();
            sys::SteamAPI_ISteamNetworkingSockets_ConnectByIPAddress(
                self.sockets,
                address.as_ptr(),
                options.len() as _,
                options.as_ptr(),
            )
        };
        NetConnection::from_raw(connection)
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
    ) -> Result<ListenSocket, InvalidHandle> {
        let options: Vec<_> = options.into_iter().map(|x| x.into()).collect();
        let socket = unsafe {
            sys::SteamAPI_ISteamNetworkingSockets_CreateListenSocketP2P(
                self.sockets,
                local_virtual_port as _,
                options.len() as _,
                options.as_ptr(),
            )
        };
        ListenSocket::from_raw(socket)
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
    ) -> Result<NetConnection, InvalidHandle> {
        let connection = unsafe {
            let options: Vec<_> = options.into_iter().map(|x| x.into()).collect();
            sys::SteamAPI_ISteamNetworkingSockets_ConnectP2P(
                self.sockets,
                identity_remote.as_ptr(),
                remote_virtual_port as _,
                options.len() as _,
                options.as_ptr(),
            )
        };
        NetConnection::from_raw(connection)
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
    pub fn accept_connection(&self, connection: &NetConnection) -> SResult<()> {
        let result = unsafe {
            sys::SteamAPI_ISteamNetworkingSockets_AcceptConnection(self.sockets, connection.0)
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
        &self,
        connection: NetConnection,
        reason: i32,
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
                connection.into(),
                reason,
                debug_string_ptr,
                enable_linger,
            )
        }
    }

    /// Destroy a listen socket.  All the connections that were accepting on the listen
    /// socket are closed ungracefully.
    pub fn close_listen_socket(&self, socket: ListenSocket) -> Result<(), InvalidHandle> {
        // There's no documentation for this return value, but looking at all other functions in this interface,
        // it's safe to assume it returns false when the handle is invalid
        let was_successful = unsafe {
            sys::SteamAPI_ISteamNetworkingSockets_CloseListenSocket(self.sockets, socket.into())
        };
        if was_successful {
            Ok(())
        } else {
            Err(InvalidHandle)
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
    ) -> Result<ListenSocket, InvalidHandle> {
        let options: Vec<_> = options.into_iter().map(|x| x.into()).collect();
        let socket = unsafe {
            sys::SteamAPI_ISteamNetworkingSockets_CreateHostedDedicatedServerListenSocket(
                self.sockets,
                local_virtual_port as _,
                options.len() as _,
                options.as_ptr(),
            )
        };
        ListenSocket::from_raw(socket)
    }

    /// Set connection user data.  the data is returned in the following places
    /// - You can query it using GetConnectionUserData.
    /// - The SteamNetworkingmessage_t structure.
    /// - The SteamNetConnectionInfo_t structure.  (Which is a member of SteamNetConnectionStatusChangedCallback_t.)
    ///
    /// Returns false if the handle is invalid.
    pub fn set_connection_user_data(
        &self,
        peer: &NetConnection,
        user_data: i64,
    ) -> Result<(), InvalidHandle> {
        let was_successful = unsafe {
            sys::SteamAPI_ISteamNetworkingSockets_SetConnectionUserData(
                self.sockets,
                peer.0,
                user_data,
            )
        };
        if was_successful {
            Ok(())
        } else {
            Err(InvalidHandle)
        }
    }

    /// Fetch connection user data.  Returns -1 if handle is invalid
    /// or if you haven't set any userdata on the connection.
    pub fn get_connection_user_data(&self, peer: &NetConnection) -> Result<i64, InvalidHandle> {
        let user_data = unsafe {
            sys::SteamAPI_ISteamNetworkingSockets_GetConnectionUserData(self.sockets, peer.0)
        };
        if user_data == -1 {
            Err(InvalidHandle)
        } else {
            Ok(user_data)
        }
    }

    /// Set a name for the connection, used mostly for debugging
    pub fn set_connection_name(&self, peer: &NetConnection, name: &str) {
        let name = CString::new(name).unwrap();
        unsafe {
            sys::SteamAPI_ISteamNetworkingSockets_SetConnectionName(
                self.sockets,
                peer.0,
                name.as_ptr(),
            )
        }
    }

    /// Fetch connection name.  Returns false if handle is invalid
    pub fn get_connection_name(&self, _peer: &NetConnection) -> Result<(), InvalidHandle> {
        unimplemented!()
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
    pub fn send_message_to_connection(
        &self,
        connection: &NetConnection,
        data: &[u8],
        send_flags: SendFlags,
    ) -> SResult<OutMessageNumber> {
        unsafe {
            let mut out_message_number = 0i64;
            let result = sys::SteamAPI_ISteamNetworkingSockets_SendMessageToConnection(
                self.sockets,
                connection.0,
                data.as_ptr() as _,
                data.len() as _,
                send_flags.bits(),
                &mut out_message_number,
            );
            match result {
                sys::EResult::k_EResultOK => Ok(OutMessageNumber(out_message_number)),
                error => Err(error.into()),
            }
        }
    }

    /// Create a new poll group.
    ///
    /// You should destroy the poll group when you are done using DestroyPollGroup
    pub fn create_poll_group(&self) -> NetPollGroup {
        let poll_group =
            unsafe { sys::SteamAPI_ISteamNetworkingSockets_CreatePollGroup(self.sockets) };
        NetPollGroup(poll_group)
    }

    /// Destroy a poll group created with CreatePollGroup().
    ///
    /// If there are any connections in the poll group, they are removed from the group,
    /// and left in a state where they are not part of any poll group.
    /// Returns false if passed an invalid poll group handle.
    pub fn destroy_poll_group(&self, poll_group: NetPollGroup) -> Result<(), InvalidHandle> {
        let was_successful = unsafe {
            sys::SteamAPI_ISteamNetworkingSockets_DestroyPollGroup(self.sockets, poll_group.0)
        };
        if was_successful {
            Ok(())
        } else {
            Err(InvalidHandle)
        }
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
    pub fn set_connection_poll_group(
        &self,
        connection: &NetConnection,
        poll_group: &NetPollGroup,
    ) -> Result<(), InvalidHandle> {
        let was_successful = unsafe {
            sys::SteamAPI_ISteamNetworkingSockets_SetConnectionPollGroup(
                self.sockets,
                connection.0,
                poll_group.0,
            )
        };
        if was_successful {
            Ok(())
        } else {
            Err(InvalidHandle)
        }
    }

    /// Clear the poll group for a connection.
    ///
    /// Returns `Err(InvalidHandle)` when `connection` is invalid.
    pub fn clear_connection_poll_group(
        &self,
        connection: &NetConnection,
    ) -> Result<(), InvalidHandle> {
        let was_successful = unsafe {
            sys::SteamAPI_ISteamNetworkingSockets_SetConnectionPollGroup(
                self.sockets,
                connection.0,
                sys::k_HSteamNetPollGroup_Invalid,
            )
        };
        if was_successful {
            Ok(())
        } else {
            Err(InvalidHandle)
        }
    }
}

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
    fn test_connect_to_self() {
        let (client, _single) = Client::init().unwrap();
        let sockets = client.networking_sockets();
        let socket = sockets
            .create_listen_socket_ip(
                SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), 12345),
                vec![],
            )
            .expect("Listen socket creation failed");
        let connection = sockets
            .connect_by_ip_address(
                SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), 12345),
                vec![],
            )
            .expect("Starting connection failed");
    }
}
