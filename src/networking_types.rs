use crate::SteamId;
use std::borrow::Cow;
use std::ffi::CString;
use std::mem::MaybeUninit;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

pub struct NetConnection(pub(crate) u32);

impl NetConnection {
    pub fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    pub fn raw(&self) -> u32 {
        self.0
    }

    pub fn is_invalid(&self) -> bool {
        self.0 == sys::k_HSteamNetConnection_Invalid
    }
}

pub struct ListenSocket(pub(crate) u32);

impl ListenSocket {
    pub fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    pub fn raw(&self) -> u32 {
        self.0
    }

    pub fn is_invalid(&self) -> bool {
        self.0 == sys::k_HSteamListenSocket_Invalid
    }
}

pub struct NetPollGroup(pub(crate) u32);

impl NetPollGroup {
    pub fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    pub fn raw(&self) -> u32 {
        self.0
    }

    pub fn is_invalid(&self) -> bool {
        self.0 == sys::k_HSteamNetPollGroup_Invalid
    }
}

bitflags! {
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[repr(C)]
    pub struct SendFlags: i32 {
        const UNRELIABLE = sys::k_nSteamNetworkingSend_Unreliable;
        const NO_NAGLE = sys::k_nSteamNetworkingSend_NoNagle;
        const UNRELIABLE_NO_NAGLE = sys::k_nSteamNetworkingSend_UnreliableNoNagle;
        const NO_DELAY = sys::k_nSteamNetworkingSend_NoDelay;
        const UNRELIABLE_NO_DELAY = sys::k_nSteamNetworkingSend_UnreliableNoDelay;
        const RELIABLE = sys::k_nSteamNetworkingSend_Reliable;
        const RELIABLE_NO_NAGLE = sys::k_nSteamNetworkingSend_ReliableNoNagle;
        const USE_CURRENT_THREAD = sys::k_nSteamNetworkingSend_UseCurrentThread;
        const AUTO_RESTART_BROKEN_SESSION = sys::k_nSteamNetworkingSend_AutoRestartBrokenSession;
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum NetworkingConfigDataType {
    Int32,
    Int64,
    Float,
    String,
    Callback,
}

impl Into<sys::ESteamNetworkingConfigDataType> for NetworkingConfigDataType {
    fn into(self) -> sys::ESteamNetworkingConfigDataType {
        match self {
            NetworkingConfigDataType::Int32 => {
                sys::ESteamNetworkingConfigDataType::k_ESteamNetworkingConfig_Int32
            }
            NetworkingConfigDataType::Int64 => {
                sys::ESteamNetworkingConfigDataType::k_ESteamNetworkingConfig_Int64
            }
            NetworkingConfigDataType::Float => {
                sys::ESteamNetworkingConfigDataType::k_ESteamNetworkingConfig_Float
            }
            NetworkingConfigDataType::String => {
                sys::ESteamNetworkingConfigDataType::k_ESteamNetworkingConfig_String
            }
            NetworkingConfigDataType::Callback => {
                sys::ESteamNetworkingConfigDataType::k_ESteamNetworkingConfig_Ptr
            }
        }
    }
}

pub enum NetworkingConfigValue {
    /// [global float, 0--100] Randomly discard N pct of packets instead of sending/recv
    /// This is a global option only, since it is applied at a low level
    /// where we don't have much context
    FakePacketLossSend,
    FakePacketLossRecv,

    /// [global int32].  Delay all outbound/inbound packets by N ms
    FakePacketLagSend,
    FakePacketLagRecv,

    /// [global float] 0-100 Percentage of packets we will add additional delay
    /// to (causing them to be reordered)
    FakePacketReorderSend,
    FakePacketReorderRecv,

    /// [global int32] Extra delay, in ms, to apply to reordered packets.
    FakePacketReorderTime,

    /// [global float 0--100] Globally duplicate some percentage of packets we send
    FakePacketDupSend,
    FakePacketDupRecv,

    /// [global int32] Amount of delay, in ms, to delay duplicated packets.
    /// (We chose a random delay between 0 and this value)
    FakePacketDupTimeMax,

    /// [connection int32] Timeout value (in ms) to use when first connecting
    TimeoutInitial,

    /// [connection int32] Timeout value (in ms) to use after connection is established
    TimeoutConnected,

    /// [connection int32] Upper limit of buffered pending bytes to be sent,
    /// if this is reached SendMessage will return k_EResultLimitExceeded
    /// Default is 512k (524288 bytes)
    SendBufferSize,

    /// [connection int32] Minimum/maximum send rate clamp, 0 is no limit.
    /// This value will control the min/max allowed sending rate that
    /// bandwidth estimation is allowed to reach.  Default is 0 (no-limit)
    SendRateMin,
    SendRateMax,

    /// [connection int32] Nagle time, in microseconds.  When SendMessage is called, if
    /// the outgoing message is less than the size of the MTU, it will be
    /// queued for a delay equal to the Nagle timer value.  This is to ensure
    /// that if the application sends several small messages rapidly, they are
    /// coalesced into a single packet.
    /// See historical RFC 896.  Value is in microseconds.
    /// Default is 5000us (5ms).
    NagleTime,

    /// [connection int32] Don't automatically fail IP connections that don't have
    /// strong auth.  On clients, this means we will attempt the connection even if
    /// we don't know our identity or can't get a cert.  On the server, it means that
    /// we won't automatically reject a connection due to a failure to authenticate.
    /// (You can examine the incoming connection and decide whether to accept it.)
    ///
    /// This is a dev configuration value, and you should not let users modify it in
    /// production.
    IPAllowWithoutAuth,

    /// [connection int32] Do not send UDP packets with a payload of
    /// larger than N bytes.  If you set this, MTU_DataSize
    /// is automatically adjusted
    MTUPacketSize,

    /// [connection int32] (read only) Maximum message size you can send that
    /// will not fragment, based on MTU_PacketSize
    MTUDataSize,

    /// [connection int32] Allow unencrypted (and unauthenticated) communication.
    /// 0: Not allowed (the default)
    /// 1: Allowed, but prefer encrypted
    /// 2: Allowed, and preferred
    /// 3: Required.  (Fail the connection if the peer requires encryption.)
    ///
    /// This is a dev configuration value, since its purpose is to disable encryption.
    /// You should not let users modify it in production.  (But note that it requires
    /// the peer to also modify their value in order for encryption to be disabled.)
    Unencrypted,

    /// [global int32] 0 or 1.  Some variables are "dev" variables.  They are useful
    /// for debugging, but should not be adjusted in production.  When this flag is false (the default),
    /// such variables will not be enumerated by the ISteamnetworkingUtils::GetFirstConfigValue
    /// ISteamNetworkingUtils::GetConfigValueInfo functions.  The idea here is that you
    /// can use those functions to provide a generic mechanism to set any configuration
    /// value from a console or configuration file, looking up the variable by name.  Depending
    /// on your game, modifying other configuration values may also have negative effects, and
    /// you may wish to further lock down which variables are allowed to be modified by the user.
    /// (Maybe no variables!)  Or maybe you use a whitelist or blacklist approach.
    ///
    /// (This flag is itself a dev variable.)
    EnumerateDevVars,

    /// [connection int32] Set this to 1 on outbound connections and listen sockets,
    /// to enable "symmetric connect mode", which is useful in the following
    /// common peer-to-peer use case:
    ///
    /// - The two peers are "equal" to each other.  (Neither is clearly the "client"
    ///   or "server".)
    /// - Either peer may initiate the connection, and indeed they may do this
    ///   at the same time
    /// - The peers only desire a single connection to each other, and if both
    ///   peers initiate connections simultaneously, a protocol is needed for them
    ///   to resolve the conflict, so that we end up with a single connection.
    ///
    /// This use case is both common, and involves subtle race conditions and tricky
    /// pitfalls, which is why the API has support for dealing with it.
    ///
    /// If an incoming connection arrives on a listen socket or via custom signaling,
    /// and the application has not attempted to make a matching outbound connection
    /// in symmetric mode, then the incoming connection can be accepted as usual.
    /// A "matching" connection means that the relevant endpoint information matches.
    /// (At the time this comment is being written, this is only supported for P2P
    /// connections, which means that the peer identities must match, and the virtual
    /// port must match.  At a later time, symmetric mode may be supported for other
    /// connection types.)
    ///
    /// If connections are initiated by both peers simultaneously, race conditions
    /// can arise, but fortunately, most of them are handled internally and do not
    /// require any special awareness from the application.  However, there
    /// is one important case that application code must be aware of:
    /// If application code attempts an outbound connection using a ConnectXxx
    /// function in symmetric mode, and a matching incoming connection is already
    /// waiting on a listen socket, then instead of forming a new connection,
    /// the ConnectXxx call will accept the existing incoming connection, and return
    /// a connection handle to this accepted connection.
    /// IMPORTANT: in this case, a SteamNetConnectionStatusChangedCallback_t
    /// has probably *already* been posted to the queue for the incoming connection!
    /// (Once callbacks are posted to the queue, they are not modified.)  It doesn't
    /// matter if the callback has not been consumed by the app.  Thus, application
    /// code that makes use of symmetric connections must be aware that, when processing a
    /// SteamNetConnectionStatusChangedCallback_t for an incoming connection, the
    /// m_hConn may refer to a new connection that the app has has not
    /// seen before (the usual case), but it may also refer to a connection that
    /// has already been accepted implicitly through a call to Connect()!  In this
    /// case, AcceptConnection() will return k_EResultDuplicateRequest.
    ///
    /// Only one symmetric connection to a given peer (on a given virtual port)
    /// may exist at any given time.  If client code attempts to create a connection,
    /// and a (live) connection already exists on the local host, then either the
    /// existing connection will be accepted as described above, or the attempt
    /// to create a new connection will fail.  Furthermore, linger mode functionality
    /// is not supported on symmetric connections.
    ///
    /// A more complicated race condition can arise if both peers initiate a connection
    /// at roughly the same time.  In this situation, each peer will receive an incoming
    /// connection from the other peer, when the application code has already initiated
    /// an outgoing connection to that peer.  The peers must resolve this conflict and
    /// decide who is going to act as the "server" and who will act as the "client".
    /// Typically the application does not need to be aware of this case as it is handled
    /// internally.  On both sides, the will observe their outbound connection being
    /// "accepted", although one of them one have been converted internally to act
    /// as the "server".
    ///
    /// In general, symmetric mode should be all-or-nothing: do not mix symmetric
    /// connections with a non-symmetric connection that it might possible "match"
    /// with.  If you use symmetric mode on any connections, then both peers should
    /// use it on all connections, and the corresponding listen socket, if any.  The
    /// behaviour when symmetric and ordinary connections are mixed is not defined by
    /// this API, and you should not rely on it.  (This advice only applies when connections
    /// might possibly "match".  For example, it's OK to use all symmetric mode
    /// connections on one virtual port, and all ordinary, non-symmetric connections
    /// on a different virtual port, as there is no potential for ambiguity.)
    ///
    /// When using the feature, you should set it in the following situations on
    /// applicable objects:
    ///
    /// - When creating an outbound connection using ConnectXxx function
    /// - When creating a listen socket.  (Note that this will automatically cause
    ///   any accepted connections to inherit the flag.)
    /// - When using custom signaling, before accepting an incoming connection.
    ///
    /// Setting the flag on listen socket and accepted connections will enable the
    /// API to automatically deal with duplicate incoming connections, even if the
    /// local host has not made any outbound requests.  (In general, such duplicate
    /// requests from a peer are ignored internally and will not be visible to the
    /// application code.  The previous connection must be closed or resolved first.)
    SymmetricConnect,

    /// [connection int32] For connection types that use "virtual ports", this can be used
    /// to assign a local virtual port.  For incoming connections, this will always be the
    /// virtual port of the listen socket (or the port requested by the remote host if custom
    /// signaling is used and the connection is accepted), and cannot be changed.  For
    /// connections initiated locally, the local virtual port will default to the same as the
    /// requested remote virtual port, if you do not specify a different option when creating
    /// the connection.  The local port is only relevant for symmetric connections, when
    /// determining if two connections "match."  In this case, if you need the local and remote
    /// port to differ, you can set this value.
    ///
    /// You can also read back this value on listen sockets.
    ///
    /// This value should not be read or written in any other context.
    LocalVirtualPort,

    //
    // Callbacks
    //

    // On Steam, you may use the default Steam callback dispatch mechanism.  If you prefer
    // to not use this dispatch mechanism (or you are not running with Steam), or you want
    // to associate specific functions with specific listen sockets or connections, you can
    // register them as configuration values.
    //
    // Note also that ISteamNetworkingUtils has some helpers to set these globally.
    /// [connection FnSteamNetConnectionStatusChanged] Callback that will be invoked
    /// when the state of a connection changes.
    ///
    /// IMPORTANT: callbacks are dispatched to the handler that is in effect at the time
    /// the event occurs, which might be in another thread.  For example, immediately after
    /// creating a listen socket, you may receive an incoming connection.  And then immediately
    /// after this, the remote host may close the connection.  All of this could happen
    /// before the function to create the listen socket has returned.  For this reason,
    /// callbacks usually must be in effect at the time of object creation.  This means
    /// you should set them when you are creating the listen socket or connection, or have
    /// them in effect so they will be inherited at the time of object creation.
    ///
    /// For example:
    ///
    /// exterm void MyStatusChangedFunc( SteamNetConnectionStatusChangedCallback_t *info );
    /// SteamNetworkingConfigValue_t opt; opt.SetPtr( Callback_ConnectionStatusChanged, MyStatusChangedFunc );
    /// SteamNetworkingIPAddr localAddress; localAddress.Clear();
    /// HSteamListenSocket hListenSock = SteamNetworkingSockets()->CreateListenSocketIP( localAddress, 1, &opt );
    ///
    /// When accepting an incoming connection, there is no atomic way to switch the
    /// callback.  However, if the connection is DOA, AcceptConnection() will fail, and
    /// you can fetch the state of the connection at that time.
    ///
    /// If all connections and listen sockets can use the same callback, the simplest
    /// method is to set it globally before you create any listen sockets or connections.
    CallbackConnectionStatusChanged,

    /// [global FnSteamNetAuthenticationStatusChanged] Callback that will be invoked
    /// when our auth state changes.  If you use this, install the callback before creating
    /// any connections or listen sockets, and don't change it.
    /// See: ISteamNetworkingUtils::SetGlobalCallback_SteamNetAuthenticationStatusChanged
    CallbackAuthStatusChanged,

    /// [global FnSteamRelayNetworkStatusChanged] Callback that will be invoked
    /// when our auth state changes.  If you use this, install the callback before creating
    /// any connections or listen sockets, and don't change it.
    /// See: ISteamNetworkingUtils::SetGlobalCallback_SteamRelayNetworkStatusChanged
    CallbackRelayNetworkStatusChanged,

    /// [global FnSteamNetworkingMessagesSessionRequest] Callback that will be invoked
    /// when a peer wants to initiate a SteamNetworkingMessagesSessionRequest.
    /// See: ISteamNetworkingUtils::SetGlobalCallback_MessagesSessionRequest
    CallbackMessagesSessionRequest,

    /// [global FnSteamNetworkingMessagesSessionFailed] Callback that will be invoked
    /// when a session you have initiated, or accepted either fails to connect, or loses
    /// connection in some unexpected way.
    /// See: ISteamNetworkingUtils::SetGlobalCallback_MessagesSessionFailed
    CallbackMessagesSessionFailed,

    /// [global FnSteamNetworkingSocketsCreateConnectionSignaling] Callback that will
    /// be invoked when we need to create a signaling object for a connection
    /// initiated locally.  See: ISteamNetworkingSockets::ConnectP2P,
    /// ISteamNetworkingMessages.
    CallbackCreateConnectionSignaling,

    //
    // P2P settings
    //

    //	/// [listen socket int32] When you create a P2P listen socket, we will automatically
    //	/// open up a UDP port to listen for LAN connections.  LAN connections can be made
    //	/// without any signaling: both sides can be disconnected from the Internet.
    //	///
    //	/// This value can be set to zero to disable the feature.
    //	P2P_Discovery_Server_LocalPort = 101,
    //
    //	/// [connection int32] P2P connections can perform broadcasts looking for the peer
    //	/// on the LAN.
    //	P2P_Discovery_Client_RemotePort = 102,
    /// [connection string] Comma-separated list of STUN servers that can be used
    /// for NAT piercing.  If you set this to an empty string, NAT piercing will
    /// not be attempted.  Also if "public" candidates are not allowed for
    /// P2P_Transport_ICE_Enable, then this is ignored.
    P2PSTUNServerList,

    /// [connection int32] What types of ICE candidates to share with the peer.
    /// See k_nSteamNetworkingConfig_P2P_Transport_ICE_Enable_xxx values
    P2PTransportICEEnable,

    /// [connection int32] When selecting P2P transport, add various
    /// penalties to the scores for selected transports.  (Route selection
    /// scores are on a scale of milliseconds.  The score begins with the
    /// route ping time and is then adjusted.)
    P2PTransportICEPenalty,
    P2PTransportSDRPenalty = 106,
    //P2P_Transport_LANBeacon_Penalty,

    //
    // Settings for SDR relayed connections
    //
    /// [int32 global] If the first N pings to a port all fail, mark that port as unavailable for
    /// a while, and try a different one.  Some ISPs and routers may drop the first
    /// packet, so setting this to 1 may greatly disrupt communications.
    SDRClientConsecutitivePingTimeoutsFailInitial,

    /// [int32 global] If N consecutive pings to a port fail, after having received successful
    /// communication, mark that port as unavailable for a while, and try a
    /// different one.
    SDRClientConsecutitivePingTimeoutsFail,

    /// [int32 global] Minimum number of lifetime pings we need to send, before we think our estimate
    /// is solid.  The first ping to each cluster is very often delayed because of NAT,
    /// routers not having the best route, etc.  Until we've sent a sufficient number
    /// of pings, our estimate is often inaccurate.  Keep pinging until we get this
    /// many pings.
    SDRClientMinPingsBeforePingAccurate,

    /// [int32 global] Set all steam datagram traffic to originate from the same
    /// local port. By default, we open up a new UDP socket (on a different local
    /// port) for each relay.  This is slightly less optimal, but it works around
    /// some routers that don't implement NAT properly.  If you have intermittent
    /// problems talking to relays that might be NAT related, try toggling
    /// this flag
    SDRClientSingleSocket,

    /// [global string] Code of relay cluster to force use.  If not empty, we will
    /// only use relays in that cluster.  E.g. 'iad'
    SDRClientForceRelayCluster,

    /// [connection string] For debugging, generate our own (unsigned) ticket, using
    /// the specified  gameserver address.  Router must be configured to accept unsigned
    /// tickets.
    SDRClientDebugTicketAddress,

    /// [global string] For debugging.  Override list of relays from the config with
    /// this set (maybe just one).  Comma-separated list.
    SDRClientForceProxyAddr,

    /// [global string] For debugging.  Force ping times to clusters to be the specified
    /// values.  A comma separated list of <cluster>=<ms> values.  E.g. "sto=32,iad=100"
    ///
    /// This is a dev configuration value, you probably should not let users modify it
    /// in production.
    SDRClientFakeClusterPing,

    //
    // Log levels for debugging information of various subsystems.
    // Higher numeric values will cause more stuff to be printed.
    // See ISteamNetworkingUtils::SetDebugOutputFunction for more
    // information
    //
    // The default for all values is k_ESteamNetworkingSocketsDebugOutputType_Warning.
    //
    LogLevelAckRTT, // [connection int32] RTT calculations for inline pings and replies
    LogLevelPacketDecode, // [connection int32] log SNP packets send/recv
    LogLevelMessage, // [connection int32] log each message send/recv
    LogLevelPacketGaps, // [connection int32] dropped packets
    LogLevelP2PRendezvous, // [connection int32] P2P rendezvous messages
    LogLevelSDRRelayPings, // [global int32] Ping relays
}

impl NetworkingConfigValue {
    pub fn data_type(&self) -> NetworkingConfigDataType {
        match self {
            NetworkingConfigValue::FakePacketLossSend => NetworkingConfigDataType::Float,
            NetworkingConfigValue::FakePacketLossRecv => NetworkingConfigDataType::Float,
            NetworkingConfigValue::FakePacketLagSend => NetworkingConfigDataType::Int32,
            NetworkingConfigValue::FakePacketLagRecv => NetworkingConfigDataType::Int32,
            NetworkingConfigValue::FakePacketReorderSend => NetworkingConfigDataType::Float,
            NetworkingConfigValue::FakePacketReorderRecv => NetworkingConfigDataType::Float,
            NetworkingConfigValue::FakePacketReorderTime => NetworkingConfigDataType::Int32,
            NetworkingConfigValue::FakePacketDupSend => NetworkingConfigDataType::Float,
            NetworkingConfigValue::FakePacketDupRecv => NetworkingConfigDataType::Float,
            NetworkingConfigValue::FakePacketDupTimeMax => NetworkingConfigDataType::Int32,
            NetworkingConfigValue::TimeoutInitial => NetworkingConfigDataType::Int32,
            NetworkingConfigValue::TimeoutConnected => NetworkingConfigDataType::Int32,
            NetworkingConfigValue::SendBufferSize => NetworkingConfigDataType::Int32,
            NetworkingConfigValue::SendRateMin => NetworkingConfigDataType::Int32,
            NetworkingConfigValue::SendRateMax => NetworkingConfigDataType::Int32,
            NetworkingConfigValue::NagleTime => NetworkingConfigDataType::Int32,
            NetworkingConfigValue::IPAllowWithoutAuth => NetworkingConfigDataType::Int32,
            NetworkingConfigValue::MTUPacketSize => NetworkingConfigDataType::Int32,
            NetworkingConfigValue::MTUDataSize => NetworkingConfigDataType::Int32,
            NetworkingConfigValue::Unencrypted => NetworkingConfigDataType::Int32,
            NetworkingConfigValue::EnumerateDevVars => NetworkingConfigDataType::Int32,
            NetworkingConfigValue::SymmetricConnect => NetworkingConfigDataType::Int32,
            NetworkingConfigValue::LocalVirtualPort => NetworkingConfigDataType::Int32,
            NetworkingConfigValue::CallbackConnectionStatusChanged => {
                NetworkingConfigDataType::Callback
            }
            NetworkingConfigValue::CallbackAuthStatusChanged => NetworkingConfigDataType::Callback,
            NetworkingConfigValue::CallbackRelayNetworkStatusChanged => {
                NetworkingConfigDataType::Callback
            }
            NetworkingConfigValue::CallbackMessagesSessionRequest => {
                NetworkingConfigDataType::Callback
            }
            NetworkingConfigValue::CallbackMessagesSessionFailed => {
                NetworkingConfigDataType::Callback
            }
            NetworkingConfigValue::CallbackCreateConnectionSignaling => {
                NetworkingConfigDataType::Callback
            }
            NetworkingConfigValue::P2PSTUNServerList => NetworkingConfigDataType::String,
            NetworkingConfigValue::P2PTransportICEEnable => NetworkingConfigDataType::Int32,
            NetworkingConfigValue::P2PTransportICEPenalty => NetworkingConfigDataType::Int32,
            NetworkingConfigValue::P2PTransportSDRPenalty => NetworkingConfigDataType::Int32,
            NetworkingConfigValue::SDRClientConsecutitivePingTimeoutsFailInitial => {
                NetworkingConfigDataType::Int32
            }
            NetworkingConfigValue::SDRClientConsecutitivePingTimeoutsFail => {
                NetworkingConfigDataType::Int32
            }
            NetworkingConfigValue::SDRClientMinPingsBeforePingAccurate => {
                NetworkingConfigDataType::Int32
            }
            NetworkingConfigValue::SDRClientSingleSocket => NetworkingConfigDataType::Int32,
            NetworkingConfigValue::SDRClientForceRelayCluster => NetworkingConfigDataType::String,
            NetworkingConfigValue::SDRClientDebugTicketAddress => NetworkingConfigDataType::String,
            NetworkingConfigValue::SDRClientForceProxyAddr => NetworkingConfigDataType::String,
            NetworkingConfigValue::SDRClientFakeClusterPing => NetworkingConfigDataType::String,
            NetworkingConfigValue::LogLevelAckRTT => NetworkingConfigDataType::Int32,
            NetworkingConfigValue::LogLevelPacketDecode => NetworkingConfigDataType::Int32,
            NetworkingConfigValue::LogLevelMessage => NetworkingConfigDataType::Int32,
            NetworkingConfigValue::LogLevelPacketGaps => NetworkingConfigDataType::Int32,
            NetworkingConfigValue::LogLevelP2PRendezvous => NetworkingConfigDataType::Int32,
            NetworkingConfigValue::LogLevelSDRRelayPings => NetworkingConfigDataType::Int32,
        }
    }
}

impl Into<sys::ESteamNetworkingConfigValue> for NetworkingConfigValue {
    fn into(self) -> steamworks_sys::ESteamNetworkingConfigValue {
        match self {
            NetworkingConfigValue::FakePacketLossSend => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_FakePacketLoss_Send,
            NetworkingConfigValue::FakePacketLossRecv => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_FakePacketLoss_Recv,
            NetworkingConfigValue::FakePacketLagSend => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_FakePacketLag_Send,
            NetworkingConfigValue::FakePacketLagRecv => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_FakePacketLag_Recv,
            NetworkingConfigValue::FakePacketReorderSend => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_FakePacketReorder_Send,
            NetworkingConfigValue::FakePacketReorderRecv => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_FakePacketReorder_Recv,
            NetworkingConfigValue::FakePacketReorderTime => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_FakePacketReorder_Time,
            NetworkingConfigValue::FakePacketDupSend => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_FakePacketDup_Send,
            NetworkingConfigValue::FakePacketDupRecv => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_FakePacketDup_Recv,
            NetworkingConfigValue::FakePacketDupTimeMax => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_FakePacketDup_TimeMax,
            NetworkingConfigValue::TimeoutInitial => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_TimeoutInitial,
            NetworkingConfigValue::TimeoutConnected => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_TimeoutConnected,
            NetworkingConfigValue::SendBufferSize => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_SendBufferSize,
            NetworkingConfigValue::SendRateMin => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_SendRateMin,
            NetworkingConfigValue::SendRateMax => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_SendRateMax,
            NetworkingConfigValue::NagleTime => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_NagleTime,
            NetworkingConfigValue::IPAllowWithoutAuth => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_IP_AllowWithoutAuth,
            NetworkingConfigValue::MTUPacketSize => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_MTU_PacketSize,
            NetworkingConfigValue::MTUDataSize => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_MTU_DataSize,
            NetworkingConfigValue::Unencrypted => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_Unencrypted,
            NetworkingConfigValue::EnumerateDevVars => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_EnumerateDevVars,
            NetworkingConfigValue::SymmetricConnect => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_SymmetricConnect,
            NetworkingConfigValue::LocalVirtualPort => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_LocalVirtualPort,
            NetworkingConfigValue::CallbackConnectionStatusChanged => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_Callback_ConnectionStatusChanged,
            NetworkingConfigValue::CallbackAuthStatusChanged => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_Callback_AuthStatusChanged,
            NetworkingConfigValue::CallbackRelayNetworkStatusChanged => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_Callback_RelayNetworkStatusChanged,
            NetworkingConfigValue::CallbackMessagesSessionRequest => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_Callback_MessagesSessionRequest,
            NetworkingConfigValue::CallbackMessagesSessionFailed => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_Callback_MessagesSessionFailed,
            NetworkingConfigValue::CallbackCreateConnectionSignaling => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_Callback_CreateConnectionSignaling,
            NetworkingConfigValue::P2PSTUNServerList => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_P2P_STUN_ServerList,
            NetworkingConfigValue::P2PTransportICEEnable => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_P2P_Transport_ICE_Enable,
            NetworkingConfigValue::P2PTransportICEPenalty => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_P2P_Transport_ICE_Penalty,
            NetworkingConfigValue::P2PTransportSDRPenalty => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_P2P_Transport_SDR_Penalty,
            NetworkingConfigValue::SDRClientConsecutitivePingTimeoutsFailInitial => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_SDRClient_ConsecutitivePingTimeoutsFailInitial,
            NetworkingConfigValue::SDRClientConsecutitivePingTimeoutsFail => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_SDRClient_ConsecutitivePingTimeoutsFail,
            NetworkingConfigValue::SDRClientMinPingsBeforePingAccurate => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_SDRClient_MinPingsBeforePingAccurate,
            NetworkingConfigValue::SDRClientSingleSocket => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_SDRClient_SingleSocket,
            NetworkingConfigValue::SDRClientForceRelayCluster => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_SDRClient_ForceRelayCluster,
            NetworkingConfigValue::SDRClientDebugTicketAddress => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_SDRClient_DebugTicketAddress,
            NetworkingConfigValue::SDRClientForceProxyAddr => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_SDRClient_ForceProxyAddr,
            NetworkingConfigValue::SDRClientFakeClusterPing => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_SDRClient_FakeClusterPing,
            NetworkingConfigValue::LogLevelAckRTT => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_LogLevel_AckRTT,
            NetworkingConfigValue::LogLevelPacketDecode => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_LogLevel_PacketDecode,
            NetworkingConfigValue::LogLevelMessage => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_LogLevel_Message,
            NetworkingConfigValue::LogLevelPacketGaps => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_LogLevel_PacketGaps,
            NetworkingConfigValue::LogLevelP2PRendezvous => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_LogLevel_P2PRendezvous,
            NetworkingConfigValue::LogLevelSDRRelayPings => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_LogLevel_SDRRelayPings,
        }
    }
}

pub struct NetworkingConfigEntry {
    inner: sys::SteamNetworkingConfigValue_t,
}

impl NetworkingConfigEntry {
    pub fn new_int32(value_type: NetworkingConfigValue, value: i32) -> Self {
        debug_assert_eq!(value_type.data_type(), NetworkingConfigDataType::Int32);

        let mut config = MaybeUninit::uninit();
        unsafe {
            sys::SteamAPI_SteamNetworkingConfigValue_t_SetInt32(
                config.as_mut_ptr(),
                value_type.into(),
                value,
            );
            NetworkingConfigEntry {
                inner: config.assume_init(),
            }
        }
    }

    pub fn new_int64(value_type: NetworkingConfigValue, value: i64) -> Self {
        debug_assert_eq!(value_type.data_type(), NetworkingConfigDataType::Int64);

        let mut config = MaybeUninit::uninit();
        unsafe {
            sys::SteamAPI_SteamNetworkingConfigValue_t_SetInt64(
                config.as_mut_ptr(),
                value_type.into(),
                value,
            );
            NetworkingConfigEntry {
                inner: config.assume_init(),
            }
        }
    }

    pub fn new_float(value_type: NetworkingConfigValue, value: f32) -> Self {
        debug_assert_eq!(value_type.data_type(), NetworkingConfigDataType::Int64);

        let mut config = MaybeUninit::uninit();
        unsafe {
            sys::SteamAPI_SteamNetworkingConfigValue_t_SetFloat(
                config.as_mut_ptr(),
                value_type.into(),
                value,
            );
            NetworkingConfigEntry {
                inner: config.assume_init(),
            }
        }
    }

    pub fn new_string(value_type: NetworkingConfigValue, value: &str) -> Self {
        debug_assert_eq!(value_type.data_type(), NetworkingConfigDataType::String);

        let mut config = MaybeUninit::uninit();
        unsafe {
            let c_str = CString::new(value).expect("Rust string could not be converted");
            sys::SteamAPI_SteamNetworkingConfigValue_t_SetString(
                config.as_mut_ptr(),
                value_type.into(),
                c_str.as_ptr(),
            );
            NetworkingConfigEntry {
                inner: config.assume_init(),
            }
        }
    }
}

impl Into<sys::SteamNetworkingConfigValue_t> for NetworkingConfigEntry {
    fn into(self) -> steamworks_sys::SteamNetworkingConfigValue_t {
        self.inner
    }
}

/// A safe wrapper for SteamNetworkingIdentity
pub struct NetworkingIdentity<'a> {
    // Using a enum for NetworkingIdentity with variants for each identity type would be more idiomatic to use,
    // but would require converting between the internal and the rust representation whenever the API is used.
    // Maybe a second type could be used for matching to avoid get_ip, get_steam_id, etc.
    inner: Cow<'a, sys::SteamNetworkingIdentity>,
}

impl NetworkingIdentity<'_> {
    pub fn new() -> Self {
        unsafe {
            let mut id = MaybeUninit::<sys::SteamNetworkingIdentity>::uninit();
            sys::SteamAPI_SteamNetworkingIdentity_Clear(id.as_mut_ptr());
            Self {
                inner: Cow::Owned(id.assume_init()),
            }
        }
    }

    pub fn steam_id(&self) -> Option<SteamId> {
        unsafe {
            let id = sys::SteamAPI_SteamNetworkingIdentity_GetSteamID64(self.inner.as_ref()
                as *const _
                as *mut _);
            if id == 0 {
                None
            } else {
                Some(SteamId(id))
            }
        }
    }
}

impl Default for NetworkingIdentity<'_> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct NetworkingMessage {
    inner: *mut sys::SteamNetworkingMessage_t,
}

impl NetworkingMessage {
    pub fn sender_id(&self) -> NetworkingIdentity {
        unsafe {
            let ident = &mut (*self.inner).m_identityPeer;
            NetworkingIdentity {
                inner: Cow::Borrowed(ident),
            }
        }
    }

    pub fn data(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts((*self.inner).m_pData as _, (*self.inner).m_cbSize as usize)
        }
    }
}

impl Drop for NetworkingMessage {
    fn drop(&mut self) {
        debug_assert!(!self.inner.is_null());

        unsafe { sys::SteamAPI_SteamNetworkingMessage_t_Release(self.inner) }
    }
}

pub(crate) struct SteamIpAddr {
    inner: sys::SteamNetworkingIPAddr,
}

#[allow(dead_code)]
impl SteamIpAddr {
    pub fn new() -> Self {
        unsafe {
            let mut ip = MaybeUninit::<sys::SteamNetworkingIPAddr>::uninit();
            sys::SteamAPI_SteamNetworkingIPAddr_Clear(ip.as_mut_ptr());
            Self {
                inner: ip.assume_init(),
            }
        }
    }

    pub fn new_ip(ip: IpAddr, port: u16) -> Self {
        SocketAddr::new(ip, port).into()
    }

    pub fn set_ip(&mut self, ip: SocketAddr) {
        match ip {
            SocketAddr::V4(ip) => {
                self.set_ipv4(ip);
            }
            SocketAddr::V6(ip) => {
                self.set_ipv6(ip);
            }
        }
    }

    pub fn set_ipv4(&mut self, ip: SocketAddrV4) {
        unsafe {
            sys::SteamAPI_SteamNetworkingIPAddr_SetIPv4(
                &mut self.inner,
                (*ip.ip()).into(),
                ip.port(),
            );
        }
    }

    pub fn set_ipv6(&mut self, ip: SocketAddrV6) {
        unsafe {
            sys::SteamAPI_SteamNetworkingIPAddr_SetIPv6(
                &mut self.inner,
                ip.ip().octets().as_ptr(),
                ip.port(),
            );
        }
    }

    pub fn get_ipv4(&self) -> Option<Ipv4Addr> {
        let ip = unsafe {
            sys::SteamAPI_SteamNetworkingIPAddr_GetIPv4(&self.inner as *const _ as *mut _)
        };
        if ip == 0 {
            None
        } else {
            Some(Ipv4Addr::from(ip))
        }
    }

    pub fn as_ptr(&self) -> *const sys::SteamNetworkingIPAddr {
        &self.inner
    }

    pub fn as_mut_ptr(&mut self) -> *mut sys::SteamNetworkingIPAddr {
        &mut self.inner
    }

    // TODO: Fix this function, for some reason it causes a segfault. Maybe there's something wrong with my setup?
    // pub fn to_string(&self) -> String {
    //     let mut buffer = vec![0; sys::SteamNetworkingIPAddr_k_cchMaxString as usize];
    //     let c_str;
    //     unsafe {
    //         sys::SteamAPI_SteamNetworkingIPAddr_ToString(
    //             &self.inner as *const _ as *mut _,
    //             buffer.as_mut_ptr(),
    //             buffer.len() as _,
    //             false,
    //         );
    //         c_str = CStr::from_ptr(buffer.as_ptr());
    //     }
    //     let str_slice = c_str.to_str().unwrap();
    //     str_slice.to_owned()
    // }
}

impl Default for SteamIpAddr {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for SteamIpAddr {
    fn eq(&self, other: &Self) -> bool {
        unsafe {
            sys::SteamAPI_SteamNetworkingIPAddr_IsEqualTo(
                &self.inner as *const _ as *mut _,
                &other.inner,
            )
        }
    }
}

impl Eq for SteamIpAddr {}

impl From<SocketAddr> for SteamIpAddr {
    fn from(ip: SocketAddr) -> Self {
        let mut steam_ip = Self::new();
        steam_ip.set_ip(ip);
        steam_ip
    }
}

impl From<SocketAddrV4> for SteamIpAddr {
    fn from(ip: SocketAddrV4) -> Self {
        let mut steam_ip = Self::new();
        steam_ip.set_ipv4(ip);
        steam_ip
    }
}

impl From<SocketAddrV6> for SteamIpAddr {
    fn from(ip: SocketAddrV6) -> Self {
        let mut steam_ip = Self::new();
        steam_ip.set_ipv6(ip);
        steam_ip
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_new_ip() {
        let _ip = SteamIpAddr::new();
        // assert_eq!(&ip.to_string(), "[::]:0");
    }

    #[test]
    fn test_set_ipv4() {
        let mut ip = SteamIpAddr::new();
        let addr = Ipv4Addr::new(192, 168, 0, 123);
        ip.set_ipv4(SocketAddrV4::new(addr, 5555));
        assert_eq!(Some(addr), ip.get_ipv4());
        // assert_eq!(&ip.to_string(), "192.168.0.123:5555");
    }
}
