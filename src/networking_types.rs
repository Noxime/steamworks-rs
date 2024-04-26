//! Types that are used by both [`networking_sockets`](../networking_sockets) and [`networking_messages`](../networking_messages).
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::networking_sockets::{InnerSocket, NetConnection};
use crate::networking_types::NetConnectionError::UnhandledType;
use crate::{Callback, Inner, SResult, SteamId};
use std::convert::{TryFrom, TryInto};
use std::ffi::{c_void, CString};
use std::fmt::{Debug, Display, Formatter};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::panic::catch_unwind;
use std::sync::Arc;
use steamworks_sys as sys;
use steamworks_sys::ESteamNetConnectionEnd;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MessageNumber(pub(crate) u64);

impl From<MessageNumber> for u64 {
    fn from(number: MessageNumber) -> Self {
        number.0
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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum NetworkingConfigDataType {
    Int32,
    Int64,
    Float,
    String,
    Callback,
}

impl From<NetworkingConfigDataType> for sys::ESteamNetworkingConfigDataType {
    fn from(ty: NetworkingConfigDataType) -> sys::ESteamNetworkingConfigDataType {
        match ty {
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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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

impl From<NetworkingConfigValue> for sys::ESteamNetworkingConfigValue {
    fn from(value: NetworkingConfigValue) -> steamworks_sys::ESteamNetworkingConfigValue {
        match value {
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
            NetworkingConfigValue::EnumerateDevVars => sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_DELETED_EnumerateDevVars,
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

/// High level connection status
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum NetworkingConnectionState {
    /// Dummy value used to indicate an error condition in the API.
    /// Specified connection doesn't exist or has already been closed.
    None,
    /// We are trying to establish whether peers can talk to each other,
    /// whether they WANT to talk to each other, perform basic auth,
    /// and exchange crypt keys.
    ///
    /// - For connections on the "client" side (initiated locally):
    ///   We're in the process of trying to establish a connection.
    ///   Depending on the connection type, we might not know who they are.
    ///   Note that it is not possible to tell if we are waiting on the
    ///   network to complete handshake packets, or for the application layer
    ///   to accept the connection.
    ///
    /// - For connections on the "server" side (accepted through listen socket):
    ///   We have completed some basic handshake and the client has presented
    ///   some proof of identity.  The connection is ready to be accepted
    ///   using AcceptConnection().
    ///
    /// In either case, any unreliable packets sent now are almost certain
    /// to be dropped.  Attempts to receive packets are guaranteed to fail.
    /// You may send messages if the send mode allows for them to be queued.
    /// but if you close the connection before the connection is actually
    /// established, any queued messages will be discarded immediately.
    /// (We will not attempt to flush the queue and confirm delivery to the
    /// remote host, which ordinarily happens when a connection is closed.)
    Connecting,
    /// Some connection types use a back channel or trusted 3rd party
    /// for earliest communication.  If the server accepts the connection,
    /// then these connections switch into the rendezvous state.  During this
    /// state, we still have not yet established an end-to-end route (through
    /// the relay network), and so if you send any messages unreliable, they
    /// are going to be discarded.
    FindingRoute,
    /// We've received communications from our peer (and we know
    /// who they are) and are all good.  If you close the connection now,
    /// we will make our best effort to flush out any reliable sent data that
    /// has not been acknowledged by the peer.  (But note that this happens
    /// from within the application process, so unlike a TCP connection, you are
    /// not totally handing it off to the operating system to deal with it.)
    Connected,
    /// Connection has been closed by our peer, but not closed locally.
    /// The connection still exists from an API perspective.  You must close the
    /// handle to free up resources.  If there are any messages in the inbound queue,
    /// you may retrieve them.  Otherwise, nothing may be done with the connection
    /// except to close it.
    ///
    /// This stats is similar to CLOSE_WAIT in the TCP state machine.
    ClosedByPeer,
    /// A disruption in the connection has been detected locally.  (E.g. timeout,
    /// local internet connection disrupted, etc.)
    ///
    /// The connection still exists from an API perspective.  You must close the
    /// handle to free up resources.
    ///
    /// Attempts to send further messages will fail.  Any remaining received messages
    /// in the queue are available.
    ProblemDetectedLocally,
}

impl From<NetworkingConnectionState> for sys::ESteamNetworkingConnectionState {
    fn from(state: NetworkingConnectionState) -> Self {
        match state {
            NetworkingConnectionState::None => sys::ESteamNetworkingConnectionState::k_ESteamNetworkingConnectionState_None,
            NetworkingConnectionState::Connecting => sys::ESteamNetworkingConnectionState::k_ESteamNetworkingConnectionState_Connecting,
            NetworkingConnectionState::FindingRoute => sys::ESteamNetworkingConnectionState::k_ESteamNetworkingConnectionState_FindingRoute,
            NetworkingConnectionState::Connected => sys::ESteamNetworkingConnectionState::k_ESteamNetworkingConnectionState_Connected,
            NetworkingConnectionState::ClosedByPeer => sys::ESteamNetworkingConnectionState::k_ESteamNetworkingConnectionState_ClosedByPeer,
            NetworkingConnectionState::ProblemDetectedLocally => sys::ESteamNetworkingConnectionState::k_ESteamNetworkingConnectionState_ProblemDetectedLocally,
        }
    }
}

impl TryFrom<sys::ESteamNetworkingConnectionState> for NetworkingConnectionState {
    type Error = InvalidConnectionState;

    fn try_from(state: sys::ESteamNetworkingConnectionState) -> Result<Self, Self::Error> {
        match state {
            sys::ESteamNetworkingConnectionState::k_ESteamNetworkingConnectionState_None => Ok(NetworkingConnectionState::None),
            sys::ESteamNetworkingConnectionState::k_ESteamNetworkingConnectionState_Connecting => Ok(NetworkingConnectionState::Connecting),
            sys::ESteamNetworkingConnectionState::k_ESteamNetworkingConnectionState_FindingRoute => Ok(NetworkingConnectionState::FindingRoute),
            sys::ESteamNetworkingConnectionState::k_ESteamNetworkingConnectionState_Connected => Ok(NetworkingConnectionState::Connected),
            sys::ESteamNetworkingConnectionState::k_ESteamNetworkingConnectionState_ClosedByPeer => Ok(NetworkingConnectionState::ClosedByPeer),
            sys::ESteamNetworkingConnectionState::k_ESteamNetworkingConnectionState_ProblemDetectedLocally => Ok(NetworkingConnectionState::ProblemDetectedLocally),
            _ => Err(InvalidConnectionState)
        }
    }
}

#[derive(Debug, Error)]
#[error("Invalid state")]
pub struct InvalidConnectionState;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct AppNetConnectionEnd {
    code: i32,
}

impl AppNetConnectionEnd {
    pub fn code(&self) -> i32 {
        self.code
    }

    /// Create a generic normal net connection end.
    pub fn generic_normal() -> Self {
        Self {
            code: ESteamNetConnectionEnd::k_ESteamNetConnectionEnd_App_Generic as i32,
        }
    }

    /// Create a normal net connection end.
    ///
    /// Indicates that the application ended the connection in a "usual" manner.
    /// E.g.: user intentionally disconnected from the server,
    /// gameplay ended normally, etc
    ///
    /// Panics if `code` is not between 1000 and 1999.
    pub fn normal(code: i32) -> Self {
        let min = ESteamNetConnectionEnd::k_ESteamNetConnectionEnd_App_Min as i32;
        let max = ESteamNetConnectionEnd::k_ESteamNetConnectionEnd_App_Max as i32;
        if code < min || code > max {
            panic!("app net connection end code {} is out of range. App normal codes must be between {} and {}", code, min, max);
        }

        Self { code }
    }

    /// Creates a generic exception net connection end code.
    pub fn generic_exception() -> Self {
        Self {
            code: ESteamNetConnectionEnd::k_ESteamNetConnectionEnd_AppException_Generic as i32,
        }
    }

    /// Create a exception net connection end.
    ///
    /// Indicates that the application ended the connection in some sort of exceptional
    /// or unusual manner that might indicate a bug or configuration
    /// issue.
    ///
    /// Panics if `code` is not between 2000 and 2999.
    pub fn exception(code: i32) -> Self {
        let min = ESteamNetConnectionEnd::k_ESteamNetConnectionEnd_AppException_Min as i32;
        let max = ESteamNetConnectionEnd::k_ESteamNetConnectionEnd_AppException_Max as i32;
        if code < min || code > max {
            panic!("app net connection end code {} is out of range. App exceptions code must be between {} and {}", code, min, max);
        }

        Self { code }
    }

    /// Returns true if the connection ended in a "normal" way, like a user intentionally disconnecting.
    pub fn is_normal(&self) -> bool {
        let min = ESteamNetConnectionEnd::k_ESteamNetConnectionEnd_App_Min as i32;
        let max = ESteamNetConnectionEnd::k_ESteamNetConnectionEnd_App_Max as i32;

        self.code >= min && self.code <= max
    }

    /// Returns true if the connection ended because of an error.
    pub fn is_exception(&self) -> bool {
        let min = ESteamNetConnectionEnd::k_ESteamNetConnectionEnd_AppException_Min as i32;
        let max = ESteamNetConnectionEnd::k_ESteamNetConnectionEnd_AppException_Max as i32;

        self.code >= min && self.code <= max
    }
}

/// Enumerate various causes of connection termination.  These are designed to work similar
/// to HTTP error codes: the numeric range gives you a rough classification as to the source
/// of the problem.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum NetConnectionEnd {
    /// Invalid/sentinel value
    ///
    /// Int value: 0
    Invalid,

    // Application codes.  These are the values you will pass to
    // ISteamNetworkingSockets::CloseConnection.  You can use these codes if
    // you want to plumb through application-specific reason codes.  If you don't
    // need this facility, feel free to always pass
    // k_ESteamNetConnectionEnd_App_Generic.
    //
    // The distinction between "normal" and "exceptional" termination is
    // one you may use if you find useful, but it's not necessary for you
    // to do so.  The only place where we distinguish between normal and
    // exceptional is in connection analytics.  If a significant
    // proportion of connections terminates in an exceptional manner,
    // this can trigger an alert.
    //
    // 1xxx: Application ended the connection in a "usual" manner.
    //       E.g.: user intentionally disconnected from the server,
    //             gameplay ended normally, etc
    App(AppNetConnectionEnd),

    //
    // System codes.  These will be returned by the system when
    // the connection state is k_ESteamNetworkingConnectionState_ClosedByPeer
    // or k_ESteamNetworkingConnectionState_ProblemDetectedLocally.  It is
    // illegal to pass a code in this range to ISteamNetworkingSockets::CloseConnection
    //

    // You cannot do what you want to do because you're running in offline mode.
    LocalOfflineMode,

    // We're having trouble contacting many (perhaps all) relays.
    // Since it's unlikely that they all went offline at once, the best
    // explanation is that we have a problem on our end.  Note that we don't
    // bother distinguishing between "many" and "all", because in practice,
    // it takes time to detect a connection problem, and by the time
    // the connection has timed out, we might not have been able to
    // actively probe all of the relay clusters, even if we were able to
    // contact them at one time.  So this code just means that:
    //
    // * We don't have any recent successful communication with any relay.
    // * We have evidence of recent failures to communicate with multiple relays.
    LocalManyRelayConnectivity,

    // A hosted server is having trouble talking to the relay
    // that the client was using, so the problem is most likely
    // on our end
    LocalHostedServerPrimaryRelay,

    // We're not able to get the network config.  This is
    // *almost* always a local issue, since the network config
    // comes from the CDN, which is pretty darn reliable.
    LocalNetworkConfig,

    // Steam rejected our request because we don't have rights
    // to do this.
    LocalRights,

    // ICE P2P rendezvous failed because we were not able to
    // determine our "public" address (e.g. reflexive address via STUN)
    //
    // If relay fallback is available (it always is on Steam), then
    // this is only used internally and will not be returned as a high
    // level failure.
    LocalP2PICENoPublicAddresses,

    // 4xxx: Connection failed or ended, and it appears that the
    //       cause does NOT have to do with the local host or their
    //       connection to the Internet.  It could be caused by the
    //       remote host, or it could be somewhere in between.

    // The connection was lost, and as far as we can tell our connection
    // to relevant services (relays) has not been disrupted.  This doesn't
    // mean that the problem is "their fault", it just means that it doesn't
    // appear that we are having network issues on our end.
    RemoteTimeout,

    // Something was invalid with the cert or crypt handshake
    // info you gave me, I don't understand or like your key types,
    // etc.
    RemoteBadEncrypt,

    // You presented me with a cert that was I was able to parse
    // and *technically* we could use encrypted communication.
    // But there was a problem that prevents me from checking your identity
    // or ensuring that somebody int he middle can't observe our communication.
    // E.g.: - the CA key was missing (and I don't accept unsigned certs)
    // - The CA key isn't one that I trust,
    // - The cert doesn't was appropriately restricted by app, user, time, data center, etc.
    // - The cert wasn't issued to you.
    // - etc
    RemoteBadCert,

    // Something wrong with the protocol version you are using.
    // (Probably the code you are running is too old.)
    RemoteBadProtocolVersion,

    // NAT punch failed failed because we never received any public
    // addresses from the remote host.  (But we did receive some
    // signals form them.)
    //
    // If relay fallback is available (it always is on Steam), then
    // this is only used internally and will not be returned as a high
    // level failure.
    RemoteP2PICENoPublicAddresses,

    // A failure that isn't necessarily the result of a software bug,
    // but that should happen rarely enough that it isn't worth specifically
    // writing UI or making a localized message for.
    // The debug string should contain further details.
    MiscGeneric,

    // Generic failure that is most likely a software bug.
    MiscInternalError,

    // The connection to the remote host timed out, but we
    // don't know if the problem is on our end, in the middle,
    // or on their end.
    MiscTimeout,

    // There's some trouble talking to Steam.
    MiscSteamConnectivity,

    // A server in a dedicated hosting situation has no relay sessions
    // active with which to talk back to a client.  (It's the client's
    // job to open and maintain those sessions.)
    MiscNoRelaySessionsToClient,

    // While trying to initiate a connection, we never received
    // *any* communication from the peer.
    //k_ESteamNetConnectionEnd_Misc_ServerNeverReplied = 5007,

    // P2P rendezvous failed in a way that we don't have more specific
    // information
    MiscP2PRendezvous,

    // NAT punch failed, probably due to NAT/firewall configuration.
    //
    // If relay fallback is available (it always is on Steam), then
    // this is only used internally and will not be returned as a high
    // level failure.
    MiscP2PNATFirewall,

    // Our peer replied that it has no record of the connection.
    // This should not happen ordinarily, but can happen in a few
    // exception cases:
    //
    // - This is an old connection, and the peer has already cleaned
    //   up and forgotten about it.  (Perhaps it timed out and they
    //   closed it and were not able to communicate this to us.)
    // - A bug or internal protocol error has caused us to try to
    //   talk to the peer about the connection before we received
    //   confirmation that the peer has accepted the connection.
    // - The peer thinks that we have closed the connection for some
    //   reason (perhaps a bug), and believes that is it is
    //   acknowledging our closure.
    MiscPeerSentNoConnection,

    /// A code that could not be handled.
    Other(i32),
}

impl From<NetConnectionEnd> for i32 {
    fn from(end: NetConnectionEnd) -> Self {
        use ESteamNetConnectionEnd::*;
        match end {
            NetConnectionEnd::Invalid => k_ESteamNetConnectionEnd_Invalid as i32,
            NetConnectionEnd::App(app_net_connection_end) => app_net_connection_end.code(),
            NetConnectionEnd::LocalOfflineMode => k_ESteamNetConnectionEnd_Local_OfflineMode as i32,
            NetConnectionEnd::LocalManyRelayConnectivity => {
                k_ESteamNetConnectionEnd_Local_ManyRelayConnectivity as i32
            }
            NetConnectionEnd::LocalHostedServerPrimaryRelay => {
                k_ESteamNetConnectionEnd_Local_HostedServerPrimaryRelay as i32
            }
            NetConnectionEnd::LocalNetworkConfig => {
                k_ESteamNetConnectionEnd_Local_NetworkConfig as i32
            }
            NetConnectionEnd::LocalRights => k_ESteamNetConnectionEnd_Local_Rights as i32,
            NetConnectionEnd::LocalP2PICENoPublicAddresses => {
                k_ESteamNetConnectionEnd_Local_P2P_ICE_NoPublicAddresses as i32
            }
            NetConnectionEnd::RemoteTimeout => k_ESteamNetConnectionEnd_Remote_Timeout as i32,
            NetConnectionEnd::RemoteBadEncrypt => k_ESteamNetConnectionEnd_Remote_BadCrypt as i32,
            NetConnectionEnd::RemoteBadCert => k_ESteamNetConnectionEnd_Remote_BadCert as i32,
            NetConnectionEnd::RemoteBadProtocolVersion => {
                k_ESteamNetConnectionEnd_Remote_BadProtocolVersion as i32
            }
            NetConnectionEnd::RemoteP2PICENoPublicAddresses => {
                k_ESteamNetConnectionEnd_Remote_P2P_ICE_NoPublicAddresses as i32
            }
            NetConnectionEnd::MiscGeneric => k_ESteamNetConnectionEnd_Misc_Generic as i32,
            NetConnectionEnd::MiscInternalError => {
                k_ESteamNetConnectionEnd_Misc_InternalError as i32
            }
            NetConnectionEnd::MiscTimeout => k_ESteamNetConnectionEnd_Misc_Timeout as i32,
            NetConnectionEnd::MiscSteamConnectivity => {
                k_ESteamNetConnectionEnd_Misc_SteamConnectivity as i32
            }
            NetConnectionEnd::MiscNoRelaySessionsToClient => {
                k_ESteamNetConnectionEnd_Misc_NoRelaySessionsToClient as i32
            }
            NetConnectionEnd::MiscP2PRendezvous => {
                k_ESteamNetConnectionEnd_Misc_P2P_Rendezvous as i32
            }
            NetConnectionEnd::MiscP2PNATFirewall => {
                k_ESteamNetConnectionEnd_Misc_P2P_NAT_Firewall as i32
            }
            NetConnectionEnd::MiscPeerSentNoConnection => {
                k_ESteamNetConnectionEnd_Misc_PeerSentNoConnection as i32
            }
            NetConnectionEnd::Other(code) => code,
        }
    }
}

impl From<i32> for NetConnectionEnd {
    fn from(end: i32) -> Self {
        use ESteamNetConnectionEnd::*;
        match end {
            end if end == k_ESteamNetConnectionEnd_Invalid as i32 => NetConnectionEnd::Invalid,
            end if end >= k_ESteamNetConnectionEnd_App_Min as i32
                && end <= k_ESteamNetConnectionEnd_AppException_Max as i32 =>
            {
                NetConnectionEnd::App(AppNetConnectionEnd { code: end })
            }
            end if end == k_ESteamNetConnectionEnd_Local_OfflineMode as i32 => {
                NetConnectionEnd::LocalOfflineMode
            }
            end if end == k_ESteamNetConnectionEnd_Local_ManyRelayConnectivity as i32 => {
                NetConnectionEnd::LocalManyRelayConnectivity
            }
            end if end == k_ESteamNetConnectionEnd_Local_HostedServerPrimaryRelay as i32 => {
                NetConnectionEnd::LocalHostedServerPrimaryRelay
            }
            end if end == k_ESteamNetConnectionEnd_Local_NetworkConfig as i32 => {
                NetConnectionEnd::LocalNetworkConfig
            }
            end if end == k_ESteamNetConnectionEnd_Local_Rights as i32 => {
                NetConnectionEnd::LocalRights
            }
            end if end == k_ESteamNetConnectionEnd_Local_P2P_ICE_NoPublicAddresses as i32 => {
                NetConnectionEnd::LocalP2PICENoPublicAddresses
            }
            end if end == k_ESteamNetConnectionEnd_Remote_Timeout as i32 => {
                NetConnectionEnd::RemoteTimeout
            }
            end if end == k_ESteamNetConnectionEnd_Remote_BadCrypt as i32 => {
                NetConnectionEnd::RemoteBadEncrypt
            }
            end if end == k_ESteamNetConnectionEnd_Remote_BadCert as i32 => {
                NetConnectionEnd::RemoteBadCert
            }
            end if end == k_ESteamNetConnectionEnd_Remote_BadProtocolVersion as i32 => {
                NetConnectionEnd::RemoteBadProtocolVersion
            }
            end if end == k_ESteamNetConnectionEnd_Remote_P2P_ICE_NoPublicAddresses as i32 => {
                NetConnectionEnd::RemoteP2PICENoPublicAddresses
            }
            end if end == k_ESteamNetConnectionEnd_Misc_Generic as i32 => {
                NetConnectionEnd::MiscGeneric
            }
            end if end == k_ESteamNetConnectionEnd_Misc_InternalError as i32 => {
                NetConnectionEnd::MiscInternalError
            }
            end if end == k_ESteamNetConnectionEnd_Misc_Timeout as i32 => {
                NetConnectionEnd::MiscTimeout
            }
            end if end == k_ESteamNetConnectionEnd_Misc_SteamConnectivity as i32 => {
                NetConnectionEnd::MiscSteamConnectivity
            }
            end if end == k_ESteamNetConnectionEnd_Misc_NoRelaySessionsToClient as i32 => {
                NetConnectionEnd::MiscNoRelaySessionsToClient
            }
            end if end == k_ESteamNetConnectionEnd_Misc_P2P_Rendezvous as i32 => {
                NetConnectionEnd::MiscP2PRendezvous
            }
            end if end == k_ESteamNetConnectionEnd_Misc_P2P_NAT_Firewall as i32 => {
                NetConnectionEnd::MiscP2PNATFirewall
            }
            end if end == k_ESteamNetConnectionEnd_Misc_PeerSentNoConnection as i32 => {
                NetConnectionEnd::MiscPeerSentNoConnection
            }
            end => Self::Other(end),
        }
    }
}

impl From<ESteamNetConnectionEnd> for NetConnectionEnd {
    fn from(end: ESteamNetConnectionEnd) -> Self {
        (end as i32).into()
    }
}

pub type NetworkingAvailabilityResult = Result<NetworkingAvailability, NetworkingAvailabilityError>;

/// Describe the status of a particular network resource
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum NetworkingAvailability {
    /// We don't know because we haven't ever checked/tried
    NeverTried,
    /// We're waiting on a dependent resource to be acquired.  (E.g. we cannot obtain a cert until we are logged into Steam.  We cannot measure latency to relays until we have the network config.)
    Waiting,
    /// We're actively trying now, but are not yet successful.
    Attempting,
    /// Resource is online/available
    Current,
}

/// Describe a error of a particular network resource
/// In general, we will not automatically retry unless you take some action that
/// depends on of requests this resource, such as querying the status, attempting
/// to initiate a connection, receive a connection, etc.  If you do not take any
#[derive(Debug, Error, Eq, PartialEq, Hash, Copy, Clone)]
pub enum NetworkingAvailabilityError {
    /// Internal dummy/sentinal. The network resource is probably not initialized yet
    #[error("unknown")]
    Unknown,
    /// A dependent resource is missing, so this service is unavailable.  (E.g. we cannot talk to routers because Internet is down or we don't have the network config.)
    #[error("A dependent resource is missing, so this service is unavailable.")]
    CannotTry,
    /// We have tried for enough time that we would expect to have been successful by now.  We have never been successful
    #[error("We have tried for enough time that we would expect to have been successful by now.  We have never been successful")]
    Failed,
    /// We tried and were successful at one time, but now it looks like we have a problem
    #[error("We tried and were successful at one time, but now it looks like we have a problem")]
    Previously,
    /// We previously failed and are currently retrying
    #[error("We previously failed and are currently retrying")]
    Retrying,
}

impl TryFrom<sys::ESteamNetworkingAvailability> for NetworkingAvailability {
    type Error = NetworkingAvailabilityError;

    fn try_from(value: sys::ESteamNetworkingAvailability) -> Result<Self, Self::Error> {
        match value {
            sys::ESteamNetworkingAvailability::k_ESteamNetworkingAvailability_Unknown => {
                Err(NetworkingAvailabilityError::Unknown)
            }
            sys::ESteamNetworkingAvailability::k_ESteamNetworkingAvailability_CannotTry => {
                Err(NetworkingAvailabilityError::CannotTry)
            }
            sys::ESteamNetworkingAvailability::k_ESteamNetworkingAvailability_Failed => {
                Err(NetworkingAvailabilityError::Failed)
            }
            sys::ESteamNetworkingAvailability::k_ESteamNetworkingAvailability_Previously => {
                Err(NetworkingAvailabilityError::Previously)
            }
            sys::ESteamNetworkingAvailability::k_ESteamNetworkingAvailability_Retrying => {
                Err(NetworkingAvailabilityError::Retrying)
            }
            sys::ESteamNetworkingAvailability::k_ESteamNetworkingAvailability_NeverTried => {
                Ok(NetworkingAvailability::NeverTried)
            }
            sys::ESteamNetworkingAvailability::k_ESteamNetworkingAvailability_Waiting => {
                Ok(NetworkingAvailability::Waiting)
            }
            sys::ESteamNetworkingAvailability::k_ESteamNetworkingAvailability_Attempting => {
                Ok(NetworkingAvailability::Attempting)
            }
            sys::ESteamNetworkingAvailability::k_ESteamNetworkingAvailability_Current => {
                Ok(NetworkingAvailability::Current)
            }
            _ => panic!("invalid networking availability {:?}", value),
        }
    }
}

#[derive(Debug, Error)]
#[error("integer value could not be converted to enum")]
pub struct InvalidEnumValue;

/// Internal struct to handle network callbacks
#[derive(Clone)]
pub struct NetConnectionInfo {
    pub(crate) inner: sys::SteamNetConnectionInfo_t,
}

#[allow(dead_code)]
impl NetConnectionInfo {
    /// Return the network identity of the remote peer.
    ///
    /// Depending on the connection type and phase of the connection, it may be unknown, in which case `None` is returned.
    /// If `Some` is returned, the return value is a valid `NetworkingIdentity`.
    pub fn identity_remote(&self) -> Option<NetworkingIdentity> {
        let identity = NetworkingIdentity::from(self.inner.m_identityRemote);
        if identity.is_valid() {
            Some(identity)
        } else {
            None
        }
    }

    pub fn user_data(&self) -> i64 {
        self.inner.m_nUserData
    }

    pub fn listen_socket(&self) -> Option<sys::HSteamNetConnection> {
        let handle = self.inner.m_hListenSocket;
        if handle == sys::k_HSteamListenSocket_Invalid {
            None
        } else {
            Some(handle)
        }
    }

    pub fn state(&self) -> Result<NetworkingConnectionState, InvalidConnectionState> {
        self.inner.m_eState.try_into()
    }

    pub fn end_reason(&self) -> Option<NetConnectionEnd> {
        if self.inner.m_eEndReason == ESteamNetConnectionEnd::k_ESteamNetConnectionEnd_Invalid as _
        {
            None
        } else {
            Some(
                self.inner
                    .m_eEndReason
                    .try_into()
                    .expect("Unknown end reason could not be converted"),
            )
        }
    }
}

impl Debug for NetConnectionInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NetConnectionInfo")
            .field("identity_remote", &self.identity_remote())
            .field("user_data", &self.user_data())
            .field("listen_socket", &self.listen_socket())
            .field("state", &self.state())
            .field("end_reason", &self.end_reason())
            .finish()
    }
}

impl From<sys::SteamNetConnectionInfo_t> for NetConnectionInfo {
    fn from(info: steamworks_sys::SteamNetConnectionInfo_t) -> Self {
        Self { inner: info }
    }
}

/// SteamNetConnectionRealTimeStatus_t structure
#[derive(Clone)]
pub struct NetConnectionRealTimeInfo {
    pub(crate) inner: sys::SteamNetConnectionRealTimeStatus_t,
}

impl NetConnectionRealTimeInfo {
    pub fn connection_state(&self) -> Result<NetworkingConnectionState, InvalidConnectionState> {
        self.inner.m_eState.try_into()
    }

    // ping in ms
    pub fn ping(&self) -> i32 {
        self.inner.m_nPing
    }

    /// Connection quality measured locally, 0...1.  (Percentage of packets delivered)
    pub fn connection_quality_local(&self) -> f32 {
        self.inner.m_flConnectionQualityLocal
    }

    /// Packet delivery success rate as observed from remote host
    pub fn connection_quality_remote(&self) -> f32 {
        self.inner.m_flConnectionQualityRemote
    }

    /// Current data rates from recent history
    pub fn out_packets_per_sec(&self) -> f32 {
        self.inner.m_flOutPacketsPerSec
    }

    /// Current data rates from recent history
    pub fn out_bytes_per_sec(&self) -> f32 {
        self.inner.m_flOutBytesPerSec
    }
    /// Current data rates from recent history
    pub fn in_packets_per_sec(&self) -> f32 {
        self.inner.m_flInPacketsPerSec
    }

    /// Current data rates from recent history
    pub fn in_bytes_per_sec(&self) -> f32 {
        self.inner.m_flInBytesPerSec
    }

    /// Estimate rate that we believe that we can send data to our peer.
    /// Note that this could be significantly higher than m_flOutBytesPerSec,
    /// meaning the capacity of the channel is higher than you are sending data.
    /// (That's OK!)
    pub fn send_rate_bytes_per_sec(&self) -> i32 {
        self.inner.m_nSendRateBytesPerSecond
    }
    /// Number of bytes pending to be sent.  This is data that you have recently
    /// requested to be sent but has not yet actually been put on the wire.  The
    /// reliable number ALSO includes data that was previously placed on the wire,
    /// but has now been scheduled for re-transmission.  Thus, it's possible to
    /// observe m_cbPendingReliable increasing between two checks, even if no
    /// calls were made to send reliable data between the checks.  Data that is
    /// awaiting the Nagle delay will appear in these numbers.
    pub fn pending_unreliable(&self) -> i32 {
        self.inner.m_cbPendingUnreliable
    }
    /// Number of bytes pending to be sent.  This is data that you have recently
    /// requested to be sent but has not yet actually been put on the wire.  The
    /// reliable number ALSO includes data that was previously placed on the wire,
    /// but has now been scheduled for re-transmission.  Thus, it's possible to
    /// observe m_cbPendingReliable increasing between two checks, even if no
    /// calls were made to send reliable data between the checks.  Data that is
    /// awaiting the Nagle delay will appear in these numbers.
    pub fn pending_reliable(&self) -> i32 {
        self.inner.m_cbPendingReliable
    }

    /// Number of bytes of reliable data that has been placed the wire, but
    /// for which we have not yet received an acknowledgment, and thus we may
    /// have to re-transmit.
    pub fn sent_unacked_reliable(&self) -> i32 {
        self.inner.m_cbSentUnackedReliable
    }

    /// If you asked us to send a message right now, how long would that message
    /// sit in the queue before we actually started putting packets on the wire?
    /// (And assuming Nagle does not cause any packets to be delayed.)
    ///
    /// In general, data that is sent by the application is limited by the
    /// bandwidth of the channel.  If you send data faster than this, it must
    /// be queued and put on the wire at a metered rate.  Even sending a small amount
    /// of data (e.g. a few MTU, say ~3k) will require some of the data to be delayed
    /// a bit.
    ///
    /// In general, the estimated delay will be approximately equal to
    ///
    ///		( m_cbPendingUnreliable+m_cbPendingReliable ) / m_nSendRateBytesPerSecond
    ///
    /// plus or minus one MTU.  It depends on how much time has elapsed since the last
    /// packet was put on the wire.  For example, the queue might have *just* been emptied,
    /// and the last packet placed on the wire, and we are exactly up against the send
    /// rate limit.  In that case we might need to wait for one packet's worth of time to
    /// elapse before we can send again.  On the other extreme, the queue might have data
    /// in it waiting for Nagle.  (This will always be less than one packet, because as soon
    /// as we have a complete packet we would send it.)  In that case, we might be ready
    /// to send data now, and this value will be 0.
    pub fn queued_send_bytes(&self) -> i64 {
        self.inner.m_usecQueueTime
    }
}

impl Debug for NetConnectionRealTimeInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NetQuickConnectionInfo")
            .field("connection_state", &self.connection_state())
            .field("ping", &self.ping())
            .field("connection_quality_local", &self.connection_quality_local())
            .field(
                "connection_quality_remote",
                &self.connection_quality_remote(),
            )
            .field("out_packets_per_sec", &self.out_packets_per_sec())
            .field("out_bytes_per_sec", &self.out_bytes_per_sec())
            .field("in_packets_per_sec", &self.in_packets_per_sec())
            .field("in_bytes_per_sec", &self.in_bytes_per_sec())
            .field("send_rate_bytes_per_sec", &self.send_rate_bytes_per_sec())
            .field("pending_unreliable", &self.pending_unreliable())
            .field("pending_reliable", &self.pending_reliable())
            .field("sent_unacked_reliable", &self.sent_unacked_reliable())
            .field("queued_send_bytes", &self.queued_send_bytes())
            .finish()
    }
}

impl From<sys::SteamNetConnectionRealTimeStatus_t> for NetConnectionRealTimeInfo {
    fn from(info: steamworks_sys::SteamNetConnectionRealTimeStatus_t) -> Self {
        Self { inner: info }
    }
}

/// Quick status of a particular lane
#[derive(Clone)]
pub struct NetConnectionRealTimeLaneStatus {
    pub(crate) inner: sys::SteamNetConnectionRealTimeLaneStatus_t,
}

impl NetConnectionRealTimeLaneStatus {
    /// Number of bytes pending to be sent.  This is data that you have recently
    /// requested to be sent but has not yet actually been put on the wire.  The
    /// reliable number ALSO includes data that was previously placed on the wire,
    /// but has now been scheduled for re-transmission.  Thus, it's possible to
    /// observe m_cbPendingReliable increasing between two checks, even if no
    /// calls were made to send reliable data between the checks.  Data that is
    /// awaiting the Nagle delay will appear in these numbers.
    /// Lane-specific, for global look at NetConnectionRealTimeInfo.
    pub fn pending_unreliable(&self) -> i32 {
        self.inner.m_cbPendingUnreliable
    }
    /// Number of bytes pending to be sent.  This is data that you have recently
    /// requested to be sent but has not yet actually been put on the wire.  The
    /// reliable number ALSO includes data that was previously placed on the wire,
    /// but has now been scheduled for re-transmission.  Thus, it's possible to
    /// observe m_cbPendingReliable increasing between two checks, even if no
    /// calls were made to send reliable data between the checks.  Data that is
    /// awaiting the Nagle delay will appear in these numbers.
    /// Lane-specific, for global look at NetConnectionRealTimeInfo.
    pub fn pending_reliable(&self) -> i32 {
        self.inner.m_cbPendingReliable
    }
    /// Number of bytes of reliable data that has been placed the wire, but
    /// for which we have not yet received an acknowledgment, and thus we may
    /// have to re-transmit.
    /// Lane-specific, for global look at NetConnectionRealTimeInfo.
    pub fn sent_unacked_reliable(&self) -> i32 {
        self.inner.m_cbSentUnackedReliable
    }
    /// Lane-specific queue time.  This value takes into consideration lane priorities
    /// and weights, and how much data is queued in each lane, and attempts to predict
    /// how any data currently queued will be sent out.
    pub fn queued_send_bytes(&self) -> i64 {
        self.inner.m_usecQueueTime
    }
}

impl From<sys::SteamNetConnectionRealTimeLaneStatus_t> for NetConnectionRealTimeLaneStatus {
    fn from(info: steamworks_sys::SteamNetConnectionRealTimeLaneStatus_t) -> Self {
        Self { inner: info }
    }
}

/// This callback is posted whenever a connection is created, destroyed, or changes state.
/// The m_info field will contain a complete description of the connection at the time the
/// change occurred and the callback was posted.  In particular, m_eState will have the
/// new connection state.
///
/// You will usually need to listen for this callback to know when:
/// - A new connection arrives on a listen socket.
///   m_info.m_hListenSocket will be set, m_eOldState = k_ESteamNetworkingConnectionState_None,
///   and m_info.m_eState = k_ESteamNetworkingConnectionState_Connecting.
///   See ISteamNetworkigSockets::AcceptConnection.
/// - A connection you initiated has been accepted by the remote host.
///   m_eOldState = k_ESteamNetworkingConnectionState_Connecting, and
///   m_info.m_eState = k_ESteamNetworkingConnectionState_Connected.
///   Some connections might transition to k_ESteamNetworkingConnectionState_FindingRoute first.
/// - A connection has been actively rejected or closed by the remote host.
///   m_eOldState = k_ESteamNetworkingConnectionState_Connecting or k_ESteamNetworkingConnectionState_Connected,
///   and m_info.m_eState = k_ESteamNetworkingConnectionState_ClosedByPeer.  m_info.m_eEndReason
///   and m_info.m_szEndDebug will have for more details.
///   NOTE: upon receiving this callback, you must still destroy the connection using
///   ISteamNetworkingSockets::CloseConnection to free up local resources.  (The details
///   passed to the function are not used in this case, since the connection is already closed.)
/// - A problem was detected with the connection, and it has been closed by the local host.
///   The most common failure is timeout, but other configuration or authentication failures
///   can cause this.  m_eOldState = k_ESteamNetworkingConnectionState_Connecting or
///   k_ESteamNetworkingConnectionState_Connected, and m_info.m_eState = k_ESteamNetworkingConnectionState_ProblemDetectedLocally.
///   m_info.m_eEndReason and m_info.m_szEndDebug will have for more details.
///   NOTE: upon receiving this callback, you must still destroy the connection using
///   ISteamNetworkingSockets::CloseConnection to free up local resources.  (The details
///   passed to the function are not used in this case, since the connection is already closed.)
///
/// Remember that callbacks are posted to a queue, and networking connections can
/// change at any time.  It is possible that the connection has already changed
/// state by the time you process this callback.
///
/// Also note that callbacks will be posted when connections are created and destroyed by your own API calls.
#[derive(Debug, Clone)]
pub struct NetConnectionStatusChanged {
    /// The handle of the connection that has changed state
    // (only important for the ListenSocketEvent, so it can stay for now in the crate visibility)
    pub(crate) connection: sys::HSteamNetConnection,
    /// Full connection info
    pub connection_info: NetConnectionInfo,

    // Debug is intentionally ignored during dead-code analysis
    #[allow(dead_code)]
    /// Previous state.  (Current state is in m_info.m_eState)
    pub old_state: NetworkingConnectionState,
}

unsafe impl Callback for NetConnectionStatusChanged {
    const ID: i32 = sys::SteamNetConnectionStatusChangedCallback_t_k_iCallback as _;
    const SIZE: i32 = std::mem::size_of::<sys::SteamNetConnectionStatusChangedCallback_t>() as _;

    unsafe fn from_raw(raw: *mut c_void) -> Self {
        let val = &mut *(raw as *mut sys::SteamNetConnectionStatusChangedCallback_t);

        NetConnectionStatusChanged {
            connection: val.m_hConn,
            connection_info: val.m_info.into(),
            old_state: val.m_eOldState.try_into().unwrap(),
        }
    }
}

impl NetConnectionStatusChanged {
    pub(crate) fn into_listen_socket_event<Manager: 'static>(
        self,
        socket: Arc<InnerSocket<Manager>>,
    ) -> Result<ListenSocketEvent<Manager>, NetConnectionError> {
        match self.connection_info.state() {
            Ok(NetworkingConnectionState::None) => {
                Err(UnhandledType(NetworkingConnectionState::None))
            }
            Ok(NetworkingConnectionState::Connecting) => {
                if let Some(remote) = self.connection_info.identity_remote() {
                    Ok(ListenSocketEvent::Connecting(ConnectionRequest {
                        remote,
                        user_data: self.connection_info.user_data(),
                        connection: NetConnection::new(
                            self.connection,
                            socket.sockets,
                            socket.inner.clone(),
                            socket,
                        ),
                    }))
                } else {
                    return Err(NetConnectionError::InvalidRemote);
                }
            }
            Ok(NetworkingConnectionState::FindingRoute) => {
                Err(UnhandledType(NetworkingConnectionState::FindingRoute))
            }
            Ok(NetworkingConnectionState::Connected) => {
                if let Some(remote) = self.connection_info.identity_remote() {
                    Ok(ListenSocketEvent::Connected(ConnectedEvent {
                        remote,
                        user_data: self.connection_info.user_data(),
                        connection: NetConnection::new(
                            self.connection,
                            socket.sockets,
                            socket.inner.clone(),
                            socket.clone(),
                        ),
                    }))
                } else {
                    return Err(NetConnectionError::InvalidRemote);
                }
            }
            Ok(NetworkingConnectionState::ClosedByPeer)
            | Ok(NetworkingConnectionState::ProblemDetectedLocally) => {
                if let Some(remote) = self.connection_info.identity_remote() {
                    Ok(ListenSocketEvent::Disconnected(DisconnectedEvent {
                        remote,
                        user_data: self.connection_info.user_data(),
                        end_reason: self
                            .connection_info
                            .end_reason()
                            .expect("disconnect event received, but no valid end reason was given"),
                    }))
                } else {
                    return Err(NetConnectionError::InvalidRemote);
                }
            }
            Err(err) => Err(NetConnectionError::UnknownType(err)),
        }
    }
}

pub enum ListenSocketEvent<Manager> {
    Connecting(ConnectionRequest<Manager>),
    Connected(ConnectedEvent<Manager>),
    Disconnected(DisconnectedEvent),
}

pub struct ConnectionRequest<Manager> {
    remote: NetworkingIdentity,
    user_data: i64,
    connection: NetConnection<Manager>,
}

impl<Manager: 'static> ConnectionRequest<Manager> {
    pub fn remote(&self) -> NetworkingIdentity {
        self.remote.clone()
    }

    pub fn user_data(&self) -> i64 {
        self.user_data
    }

    pub fn accept(self) -> SResult<()> {
        self.connection.accept()
    }

    pub fn reject(self, end_reason: NetConnectionEnd, debug_string: Option<&str>) -> bool {
        self.connection.close(end_reason, debug_string, false)
    }
}

pub struct ConnectedEvent<Manager> {
    remote: NetworkingIdentity,
    user_data: i64,
    connection: NetConnection<Manager>,
}

impl<Manager> ConnectedEvent<Manager> {
    pub fn remote(&self) -> NetworkingIdentity {
        self.remote.clone()
    }
    pub fn user_data(&self) -> i64 {
        self.user_data
    }
    pub fn connection(&self) -> &NetConnection<Manager> {
        &self.connection
    }

    pub fn take_connection(self) -> NetConnection<Manager> {
        self.connection
    }
}

pub struct DisconnectedEvent {
    remote: NetworkingIdentity,
    user_data: i64,
    end_reason: NetConnectionEnd,
}

impl DisconnectedEvent {
    pub fn remote(&self) -> NetworkingIdentity {
        self.remote.clone()
    }
    pub fn user_data(&self) -> i64 {
        self.user_data
    }
    pub fn end_reason(&self) -> NetConnectionEnd {
        self.end_reason
    }
}

#[derive(Debug, Error)]
pub(crate) enum NetConnectionError {
    #[error("internal event type has no corresponding external event")]
    UnhandledType(NetworkingConnectionState),
    #[error("invalid event type")]
    UnknownType(InvalidConnectionState),
    #[error("invalid remote")]
    InvalidRemote,
}

#[derive(Clone)]
pub struct NetworkingConfigEntry {
    inner: sys::SteamNetworkingConfigValue_t,
}

impl NetworkingConfigEntry {
    fn new_uninitialized_config_value() -> sys::SteamNetworkingConfigValue_t {
        sys::SteamNetworkingConfigValue_t {
            m_eValue: sys::ESteamNetworkingConfigValue::k_ESteamNetworkingConfig_Invalid,
            m_eDataType: sys::ESteamNetworkingConfigDataType::k_ESteamNetworkingConfig_Int32,
            m_val: sys::SteamNetworkingConfigValue_t__bindgen_ty_1 { m_int32: 0 },
        }
    }

    pub fn new_int32(value_type: NetworkingConfigValue, value: i32) -> Self {
        debug_assert_eq!(value_type.data_type(), NetworkingConfigDataType::Int32);

        let mut config = Self::new_uninitialized_config_value();
        unsafe {
            sys::SteamAPI_SteamNetworkingConfigValue_t_SetInt32(
                &mut config,
                value_type.into(),
                value,
            );
            NetworkingConfigEntry { inner: config }
        }
    }

    pub fn new_int64(value_type: NetworkingConfigValue, value: i64) -> Self {
        debug_assert_eq!(value_type.data_type(), NetworkingConfigDataType::Int64);

        let mut config = Self::new_uninitialized_config_value();
        unsafe {
            sys::SteamAPI_SteamNetworkingConfigValue_t_SetInt64(
                &mut config,
                value_type.into(),
                value,
            );
            NetworkingConfigEntry { inner: config }
        }
    }

    pub fn new_float(value_type: NetworkingConfigValue, value: f32) -> Self {
        debug_assert_eq!(value_type.data_type(), NetworkingConfigDataType::Float);

        let mut config = Self::new_uninitialized_config_value();
        unsafe {
            sys::SteamAPI_SteamNetworkingConfigValue_t_SetFloat(
                &mut config,
                value_type.into(),
                value,
            );
            NetworkingConfigEntry { inner: config }
        }
    }

    pub fn new_string(value_type: NetworkingConfigValue, value: &str) -> Self {
        debug_assert_eq!(value_type.data_type(), NetworkingConfigDataType::String);

        let mut config = Self::new_uninitialized_config_value();
        unsafe {
            let c_str = CString::new(value).expect("Rust string could not be converted");
            sys::SteamAPI_SteamNetworkingConfigValue_t_SetString(
                &mut config,
                value_type.into(),
                c_str.as_ptr(),
            );
            NetworkingConfigEntry { inner: config }
        }
    }
}

impl From<NetworkingConfigEntry> for sys::SteamNetworkingConfigValue_t {
    fn from(entry: NetworkingConfigEntry) -> sys::SteamNetworkingConfigValue_t {
        entry.inner
    }
}

/// A safe wrapper for SteamNetworkingIdentity
#[derive(Clone)]
pub struct NetworkingIdentity {
    // Using a enum for NetworkingIdentity with variants for each identity type would be more idiomatic to use,
    // but would require converting between the internal and the rust representation whenever the API is used.
    // Maybe a second type could be used for matching to avoid get_ip, get_steam_id, etc.
    inner: sys::SteamNetworkingIdentity,
}

// const NETWORK_IDENTITY_STRING_BUFFER_SIZE: usize =
//     sys::SteamNetworkingIdentity__bindgen_ty_1::k_cchMaxString as usize;

impl NetworkingIdentity {
    pub fn new() -> Self {
        unsafe {
            let mut id = sys::SteamNetworkingIdentity {
                m_eType: sys::ESteamNetworkingIdentityType::k_ESteamNetworkingIdentityType_Invalid,
                m_cbSize: 0,
                __bindgen_anon_1: sys::SteamNetworkingIdentity__bindgen_ty_2 { m_steamID64: 0 },
            };
            sys::SteamAPI_SteamNetworkingIdentity_Clear(&mut id);
            Self { inner: id }
        }
    }

    pub fn new_steam_id(id: SteamId) -> Self {
        let mut identity = Self::new();
        identity.set_steam_id(id);
        identity
    }

    pub fn new_ip(addr: SocketAddr) -> Self {
        let mut identity = Self::new();
        identity.set_ip_addr(addr);
        identity
    }

    pub fn steam_id(&self) -> Option<SteamId> {
        unsafe {
            let id = sys::SteamAPI_SteamNetworkingIdentity_GetSteamID64(self.as_ptr() as *mut _);
            if id == 0 {
                None
            } else {
                Some(SteamId(id))
            }
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.is_invalid()
    }

    pub fn is_invalid(&self) -> bool {
        unsafe { sys::SteamAPI_SteamNetworkingIdentity_IsInvalid(self.as_ptr() as *mut _) }
    }

    pub fn set_steam_id(&mut self, id: SteamId) {
        unsafe { sys::SteamAPI_SteamNetworkingIdentity_SetSteamID64(self.as_mut_ptr(), id.0) }
    }

    pub fn set_ip_addr(&mut self, addr: SocketAddr) {
        let addr = SteamIpAddr::from(addr);
        unsafe {
            sys::SteamAPI_SteamNetworkingIdentity_SetIPAddr(self.as_mut_ptr(), addr.as_ptr());
        }
    }

    #[allow(dead_code)]
    pub(crate) fn ip_addr(&self) -> Option<SteamIpAddr> {
        unsafe {
            let ip = sys::SteamAPI_SteamNetworkingIdentity_GetIPAddr(self.as_ptr() as *mut _);
            if ip.is_null() {
                None
            } else {
                Some(SteamIpAddr { inner: (*ip) })
            }
        }
    }

    pub fn set_local_host(&mut self) {
        unsafe { sys::SteamAPI_SteamNetworkingIdentity_SetLocalHost(self.as_mut_ptr()) }
    }

    pub fn is_local_host(&self) -> bool {
        unsafe { sys::SteamAPI_SteamNetworkingIdentity_IsLocalHost(self.as_ptr() as *mut _) }
    }

    pub fn debug_string(&self) -> String {
        // For some reason I can't get the original function to work,
        // so I decided to recreate the original from https://github.com/ValveSoftware/GameNetworkingSockets/blob/529901e7c1caf50928ac8814cad205d192bbf27d/src/steamnetworkingsockets/steamnetworkingsockets_shared.cpp

        // let mut buffer = vec![0i8; NETWORK_IDENTITY_STRING_BUFFER_SIZE];
        // let string = unsafe {
        //     sys::SteamAPI_SteamNetworkingIdentity_ToString(
        //         self.as_ptr() as *mut sys::SteamNetworkingIdentity,
        //         buffer.as_mut_ptr(),
        //         NETWORK_IDENTITY_STRING_BUFFER_SIZE as u32,
        //     );
        //     CString::from_raw(buffer.as_mut_ptr())
        // };
        // string.into_string().unwrap()

        unsafe {
            match self.inner.m_eType {
                sys::ESteamNetworkingIdentityType::k_ESteamNetworkingIdentityType_Invalid => {
                    "invalid".to_string()
                }
                sys::ESteamNetworkingIdentityType::k_ESteamNetworkingIdentityType_SteamID => {
                    let id = self.inner.__bindgen_anon_1.m_steamID64;
                    format!("steamid:{}", id)
                }
                sys::ESteamNetworkingIdentityType::k_ESteamNetworkingIdentityType_IPAddress => {
                    let ip = SteamIpAddr::from(self.inner.__bindgen_anon_1.m_ip);
                    format!("ip:{}", ip)
                }
                sys::ESteamNetworkingIdentityType::k_ESteamNetworkingIdentityType_GenericString => {
                    unimplemented!()
                }
                sys::ESteamNetworkingIdentityType::k_ESteamNetworkingIdentityType_GenericBytes => {
                    unimplemented!()
                }
                sys::ESteamNetworkingIdentityType::k_ESteamNetworkingIdentityType_UnknownType => {
                    unimplemented!()
                }
                ty => format!("bad_type:{}", ty as u32),
            }
        }
    }

    pub(crate) fn as_ptr(&self) -> *const sys::SteamNetworkingIdentity {
        &self.inner
    }

    pub(crate) fn as_mut_ptr(&mut self) -> *mut sys::SteamNetworkingIdentity {
        &mut self.inner
    }
}

impl Debug for NetworkingIdentity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.debug_string())
    }
}

impl From<sys::SteamNetworkingIdentity> for NetworkingIdentity {
    fn from(id: steamworks_sys::SteamNetworkingIdentity) -> Self {
        NetworkingIdentity { inner: id }
    }
}

impl From<SteamId> for NetworkingIdentity {
    fn from(id: SteamId) -> Self {
        Self::new_steam_id(id)
    }
}

impl Default for NetworkingIdentity {
    fn default() -> Self {
        Self::new()
    }
}

pub struct NetworkingMessage<Manager> {
    pub(crate) message: *mut sys::SteamNetworkingMessage_t,

    // Not sure if this is necessary here, we may not need a Manager to use free on messages
    pub(crate) _inner: Arc<Inner<Manager>>,
}

impl<Manager> NetworkingMessage<Manager> {
    /// For messages received on connections: what connection did this come from?
    /// For outgoing messages: what connection to send it to?
    /// Not used when using the ISteamNetworkingMessages interface
    #[allow(dead_code)]
    pub(crate) fn connection(&self) -> Option<sys::HSteamNetConnection> {
        let handle = unsafe { (*self.message).m_conn };
        if handle == sys::k_HSteamNetConnection_Invalid {
            None
        } else {
            Some(handle)
        }
    }

    /// Set the target connection for the connection.
    /// Make sure you don't close or drop the `NetConnection` before sending your message.
    ///
    /// Use this with `ListenSocket::send_messages` for efficient sending.
    pub fn set_connection(&mut self, connection: &NetConnection<Manager>) {
        unsafe { (*self.message).m_conn = connection.handle }
    }

    /// For inbound messages: Who sent this to us?
    /// For outbound messages on connections: not used.
    /// For outbound messages on the ad-hoc ISteamNetworkingMessages interface: who should we send this to?
    pub fn identity_peer(&self) -> NetworkingIdentity {
        unsafe {
            let ident = &mut (*self.message).m_identityPeer;
            NetworkingIdentity { inner: *ident }
        }
    }

    /// The identity of the sender or, the receiver when used with the NetworkingMessages interface.
    pub fn set_identity_peer(&mut self, identity: NetworkingIdentity) {
        unsafe { (*self.message).m_identityPeer = identity.inner }
    }

    /// For messages received on connections, this is the user data
    /// associated with the connection.
    ///
    /// This is *usually* the same as calling GetConnection() and then
    /// fetching the user data associated with that connection, but for
    /// the following subtle differences:
    ///
    /// - This user data will match the connection's user data at the time
    ///   is captured at the time the message is returned by the API.
    ///   If you subsequently change the userdata on the connection,
    ///   this won't be updated.
    /// - This is an inline call, so it's *much* faster.
    /// - You might have closed the connection, so fetching the user data
    ///   would not be possible.
    ///
    /// Not used when sending messages,
    pub fn connection_user_data(&self) -> i64 {
        unsafe { (*self.message).m_nConnUserData }
    }

    /// Message number assigned by the sender.
    /// This is not used for outbound messages
    pub fn message_number(&self) -> MessageNumber {
        unsafe { MessageNumber((*self.message).m_nMessageNumber as u64) }
    }

    /// Bitmask of k_nSteamNetworkingSend_xxx flags.
    /// For received messages, only the k_nSteamNetworkingSend_Reliable bit is valid.
    /// For outbound messages, all bits are relevant
    pub fn send_flags(&self) -> SendFlags {
        unsafe {
            SendFlags::from_bits((*self.message).m_nFlags)
                .expect("send flags could not be converted to rust representation")
        }
    }

    pub fn set_send_flags(&mut self, send_flags: SendFlags) {
        unsafe { (*self.message).m_nFlags = send_flags.bits() }
    }

    /// Bitmask of k_nSteamNetworkingSend_xxx flags.
    /// For received messages, only the k_nSteamNetworkingSend_Reliable bit is valid.
    /// For outbound messages, all bits are relevant
    pub fn channel(&self) -> i32 {
        unsafe { (*self.message).m_nChannel }
    }

    pub fn set_channel(&mut self, channel: i32) {
        unsafe {
            (*self.message).m_nChannel = channel;
        }
    }

    /// Message payload
    pub fn data(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                (*self.message).m_pData as _,
                (*self.message).m_cbSize as usize,
            )
        }
    }

    pub fn copy_data_into_buffer(&mut self, data: &[u8]) -> Result<(), MessageError> {
        unsafe {
            if (*self.message).m_pData.is_null() {
                return Err(MessageError::NullBuffer);
            }

            if ((*self.message).m_cbSize as usize) < data.len() {
                return Err(MessageError::BufferTooSmall);
            }

            ((*self.message).m_pData as *mut u8).copy_from(data.as_ptr(), data.len());
        }

        Ok(())
    }

    /// Set a new buffer for the message.
    ///
    /// Returns `Err(MessageError::BufferAlreadySet)` if the current buffer is not NULL.
    pub fn set_data(&mut self, data: Vec<u8>) -> Result<(), MessageError> {
        unsafe {
            if !(*self.message).m_pData.is_null() {
                return Err(MessageError::BufferAlreadySet);
            }

            let mut data = data.into_boxed_slice();
            (*self.message).m_pData = data.as_mut_ptr() as *mut c_void;
            (*self.message).m_cbSize = data.len() as _;
            (*self.message).m_pfnFreeData = Some(free_rust_message_buffer);
            std::mem::forget(data);
        }

        Ok(())
    }

    /// Arbitrary user data that you can use when sending messages using
    /// ISteamNetworkingUtils::AllocateMessage and ISteamNetworkingSockets::SendMessage.
    /// (The callback you set in m_pfnFreeData might use this field.)
    ///
    /// Not used for received messages.
    pub fn user_data(&self) -> i64 {
        unsafe { (*self.message).m_nUserData }
    }

    /// Arbitrary user data that you can use when sending messages using
    /// ISteamNetworkingUtils::AllocateMessage and ISteamNetworkingSockets::SendMessage.
    /// (The callback you set in m_pfnFreeData might use this field.)
    ///
    /// Not used for received messages.
    pub fn set_user_data(&mut self, user_data: i64) {
        unsafe {
            (*self.message).m_nUserData = user_data;
        }
    }

    /// Return the message pointer and prevent rust from releasing it
    pub(crate) fn take_message(mut self) -> *mut sys::SteamNetworkingMessage_t {
        let message = self.message;
        self.message = std::ptr::null_mut();
        message
    }
}

extern "C" fn free_rust_message_buffer(message: *mut sys::SteamNetworkingMessage_t) {
    // Panic in code called by C is undefined behaviour
    if let Err(e) = catch_unwind(|| unsafe {
        let buffer =
            std::slice::from_raw_parts_mut((*message).m_pData, (*message).m_cbSize as usize);
        // Create the box again and drop it immediately
        let _ = Box::from_raw(buffer.as_mut_ptr());
    }) {
        eprintln!("{:?}", e);
    }
}

impl<Manager> Drop for NetworkingMessage<Manager> {
    fn drop(&mut self) {
        if !self.message.is_null() {
            unsafe { sys::SteamAPI_SteamNetworkingMessage_t_Release(self.message) }
        }
    }
}

#[derive(Debug, Error)]
pub enum MessageError {
    #[error("failed to write data to message, the buffer is not set")]
    NullBuffer,
    #[error("copied data is too large for the current buffer")]
    BufferTooSmall,
    #[error("cannot set a new buffer, the message already has a valid buffer")]
    BufferAlreadySet,
}

#[derive(Copy, Clone)]
pub(crate) struct SteamIpAddr {
    inner: sys::SteamNetworkingIPAddr,
}

#[allow(dead_code)]
impl SteamIpAddr {
    pub fn new() -> Self {
        unsafe {
            let mut ip = sys::SteamNetworkingIPAddr {
                __bindgen_anon_1: sys::SteamNetworkingIPAddr__bindgen_ty_2 {
                    m_ipv4: sys::SteamNetworkingIPAddr_IPv4MappedAddress {
                        m_8zeros: 0,
                        m_0000: 0,
                        m_ffff: 0,
                        m_ip: [0; 4],
                    },
                },
                m_port: 0,
            };
            sys::SteamAPI_SteamNetworkingIPAddr_Clear(&mut ip);
            Self { inner: ip }
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

    pub fn is_ipv4(&self) -> bool {
        unsafe { sys::SteamAPI_SteamNetworkingIPAddr_IsIPv4(self.as_ptr() as *mut _) }
    }

    pub fn as_ptr(&self) -> *const sys::SteamNetworkingIPAddr {
        &self.inner
    }

    pub fn as_mut_ptr(&mut self) -> *mut sys::SteamNetworkingIPAddr {
        &mut self.inner
    }

    pub fn to_string(&self, with_port: bool) -> String {
        // Similar as with NetworkIdentity, I wasn't able to get the C function to work,
        // so I'm recreating it from https://github.com/ValveSoftware/GameNetworkingSockets/blob/529901e7c1caf50928ac8814cad205d192bbf27d/src/steamnetworkingsockets/steamnetworkingsockets_shared.cpp
        // let mut buffer = vec![0; sys::SteamNetworkingIPAddr_k_cchMaxString as usize];
        // let c_str;
        // unsafe {
        //     sys::SteamAPI_SteamNetworkingIPAddr_ToString(
        //         &self.inner as *const _ as *mut _,
        //         buffer.as_mut_ptr(),
        //         buffer.len() as _,
        //         false,
        //     );
        //     c_str = CStr::from_ptr(buffer.as_ptr());
        // }
        // let str_slice = c_str.to_str().unwrap();
        // str_slice.to_owned()

        unsafe {
            if self.is_ipv4() {
                let ip4 = self.inner.__bindgen_anon_1.m_ipv4.m_ip;
                if with_port {
                    // This variable is necessary, format will create a unaligned reference to m_port, which can cause undefined bahavior
                    let port = self.inner.m_port;
                    format!("{}.{}.{}.{}:{}", ip4[0], ip4[1], ip4[2], ip4[3], port)
                } else {
                    format!("{}.{}.{}.{}", ip4[0], ip4[1], ip4[2], ip4[3])
                }
            } else {
                // I'm just assuming that steam and rust have the same representation of ip6
                let ip6 = Ipv6Addr::from(self.inner.__bindgen_anon_1.m_ipv6);
                if with_port {
                    // Same as with ipv4, don't remove this temp variable
                    let port = self.inner.m_port;
                    format!("[{}]:{}", ip6, port)
                } else {
                    format!("{}", ip6)
                }
            }
        }
    }
}

impl Debug for SteamIpAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_string(true))
    }
}

impl Display for SteamIpAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_string(true))
    }
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
impl From<sys::SteamNetworkingIPAddr> for SteamIpAddr {
    fn from(inner: sys::SteamNetworkingIPAddr) -> Self {
        Self { inner }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Client;
    use std::net::Ipv4Addr;

    #[test]
    fn test_new_ip() {
        let ip = SteamIpAddr::new();
        assert_eq!(&ip.to_string(true), "[::]:0");
    }

    #[test]
    fn test_set_ipv4() {
        let mut ip = SteamIpAddr::new();
        let addr = Ipv4Addr::new(192, 168, 0, 123);
        ip.set_ipv4(SocketAddrV4::new(addr, 5555));
        assert_eq!(Some(addr), ip.get_ipv4());
        assert_eq!(&ip.to_string(true), "192.168.0.123:5555");
    }

    #[test]
    fn test_network_identity_steam_id() {
        let id = NetworkingIdentity::new_steam_id(SteamId(123456));
        assert_eq!("steamid:123456", &id.debug_string())
    }

    #[test]
    fn test_network_identity_ip() {
        let id =
            NetworkingIdentity::new_ip(SocketAddr::new(Ipv4Addr::new(192, 168, 0, 5).into(), 1234));
        assert_eq!("ip:192.168.0.5:1234", &id.debug_string())
    }

    #[test]
    fn test_allocate_and_free_message() {
        let (client, _single) = Client::init().unwrap();
        let utils = client.networking_utils();

        // With C buffer
        {
            let _message = utils.allocate_message(200);
            // Drop it immediately
        }

        // With rust buffer
        {
            let mut message = utils.allocate_message(0);
            message.set_data(vec![1, 2, 3, 4, 5]).unwrap();

            // Drop it immediately
        }
    }
}
