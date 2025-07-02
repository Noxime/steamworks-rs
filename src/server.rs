use super::*;
use crate::networking_types::NetworkingIdentity;
#[cfg(test)]
use serial_test::serial;
use std::net::{Ipv4Addr, SocketAddrV4};

/// The main entry point into the steam client for servers.
///
/// This provides access to all of the steamworks api that
/// servers can use.
#[derive(Clone)]
pub struct Server {
    inner: Arc<Inner>,
    server: *mut sys::ISteamGameServer,
}

unsafe impl Send for Server {}
unsafe impl Sync for Server {}

/// Used to set the mode that a gameserver will run in
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServerMode {
    /// Don't authenticate user logins.
    ///
    /// The server will not appear on the server list
    NoAuthentication,
    /// Authenticate users
    ///
    /// The server will appear on the server list and
    /// VAC will not be run on clients.
    Authentication,
    /// Authenticate users and use anti-cheat.
    ///
    /// The server will appear on the server list and
    /// VAC will be run on clients.
    AuthenticationAndSecure,
}

/// Pass to SteamGameServer_Init to indicate that the same UDP port will be used for game traffic
/// UDP queries for server browser pings and LAN discovery.  In this case, Steam will not open up a
/// socket to handle server browser queries, and you must use ISteamGameServer::HandleIncomingPacket
/// and ISteamGameServer::GetNextOutgoingPacket to handle packets related to server discovery on
/// your socket.
pub const QUERY_PORT_SHARED: u16 = sys::STEAMGAMESERVER_QUERY_PORT_SHARED;

impl Server {
    fn steam_game_server_init_ex(
        un_ip: std::ffi::c_uint,
        us_game_port: std::ffi::c_ushort,
        us_query_port: std::ffi::c_ushort,
        e_server_mode: EServerMode,
        pch_version_string: *const c_char,
        p_out_err_msg: *mut SteamErrMsg,
    ) -> ESteamAPIInitResult {
        let versions: Vec<&[u8]> = vec![
            sys::STEAMUTILS_INTERFACE_VERSION,
            sys::STEAMNETWORKINGUTILS_INTERFACE_VERSION,
            sys::STEAMGAMESERVER_INTERFACE_VERSION,
            sys::STEAMGAMESERVERSTATS_INTERFACE_VERSION,
            sys::STEAMHTTP_INTERFACE_VERSION,
            sys::STEAMINVENTORY_INTERFACE_VERSION,
            sys::STEAMNETWORKING_INTERFACE_VERSION,
            sys::STEAMNETWORKINGMESSAGES_INTERFACE_VERSION,
            sys::STEAMNETWORKINGSOCKETS_INTERFACE_VERSION,
            sys::STEAMUGC_INTERFACE_VERSION,
            b"\0",
        ];

        let merged_versions: Vec<u8> = versions.into_iter().flatten().cloned().collect();
        let merged_versions_ptr = merged_versions.as_ptr().cast::<::std::os::raw::c_char>();

        return unsafe {
            sys::SteamInternal_GameServer_Init_V2(
                un_ip,
                us_game_port,
                us_query_port,
                e_server_mode,
                pch_version_string,
                merged_versions_ptr,
                p_out_err_msg,
            )
        };
    }

    /// Attempts to initialize the steamworks api and returns
    /// a server to access the rest of the api.
    ///
    /// This should only ever have one instance per a program.
    ///
    /// Currently the steamworks api doesn't support IPv6.
    ///
    /// # Errors
    ///
    /// This can fail if:
    /// * The steam client isn't running
    /// * The app ID of the game couldn't be determined.
    ///
    ///   If the game isn't being run through steam this can be provided by
    ///   placing a `steam_appid.txt` with the ID inside in the current
    ///   working directory
    /// * The game isn't running on the same user/level as the steam client
    /// * The user doesn't own a license for the game.
    /// * The app ID isn't completely set up.
    pub fn init(
        ip: Ipv4Addr,
        game_port: u16,
        query_port: u16,
        server_mode: ServerMode,
        version: &str,
    ) -> SIResult<(Server, Client)> {
        unsafe {
            let version = CString::new(version).unwrap();

            // let internal_check_interface_versions =

            let raw_ip: u32 = ip.into();
            let server_mode = match server_mode {
                ServerMode::NoAuthentication => sys::EServerMode::eServerModeNoAuthentication,
                ServerMode::Authentication => sys::EServerMode::eServerModeAuthentication,
                ServerMode::AuthenticationAndSecure => {
                    sys::EServerMode::eServerModeAuthenticationAndSecure
                }
            };

            let mut err_msg: sys::SteamErrMsg = [0; 1024];
            let result = Self::steam_game_server_init_ex(
                raw_ip,
                game_port,
                query_port,
                server_mode,
                version.as_ptr(),
                &mut err_msg,
            );

            if result != sys::ESteamAPIInitResult::k_ESteamAPIInitResult_OK {
                return Err(SteamAPIInitError::from_result_and_message(result, err_msg));
            }

            sys::SteamAPI_ManualDispatch_Init();
            let server_raw = sys::SteamAPI_SteamGameServer_v015();
            let server = Arc::new(Inner {
                manager: Box::new(ServerManager),
                callbacks: Callbacks {
                    callbacks: Mutex::new(HashMap::new()),
                    call_results: Mutex::new(HashMap::new()),
                },
                networking_sockets_data: Mutex::new(NetworkingSocketsData {
                    sockets: Default::default(),
                    independent_connections: Default::default(),
                    connection_callback: Default::default(),
                }),
            });
            Ok((
                Server {
                    inner: server.clone(),
                    server: server_raw,
                },
                Client { inner: server },
            ))
        }
    }

    /// Runs any currently pending callbacks
    ///
    /// This runs all currently pending callbacks on the current
    /// thread.
    ///
    /// This should be called frequently (e.g. once per a frame)
    /// in order to reduce the latency between recieving events.
    pub fn run_callbacks(&self) {
        self.inner.run_callbacks()
    }

    /// Runs any currently pending callbacks.
    ///
    /// This is identical to `run_callbacks` in every way, except that
    /// `callback_handler` is called for every callback invoked.
    ///
    /// This option provides an alternative for handling callbacks that
    /// can doesn't require the handler to be `Send`, and `'static`.
    ///
    /// This should be called frequently (e.g. once per a frame)
    /// in order to reduce the latency between recieving events.
    pub fn process_callbacks(&self, mut callback_handler: impl FnMut(CallbackResult)) {
        self.inner.process_callbacks(&mut callback_handler)
    }

    /// Registers the passed function as a callback for the
    /// given type.
    ///
    /// The callback will be run on the thread that [`run_callbacks`]
    /// is called when the event arrives.
    ///
    /// If the callback handler cannot be made `Send` or `'static`
    /// the call to [`run_callbacks`] should be replaced with a call to
    /// [`process_callbacks`] instead.
    ///
    /// [`run_callbacks`]: Self::run_callbacks
    /// [`process_callbacks`]: Self::proceses_callbacks
    pub fn register_callback<C, F>(&self, f: F) -> CallbackHandle
    where
        C: Callback,
        F: FnMut(C) + 'static + Send,
    {
        unsafe { register_callback(&self.inner, f) }
    }

    /// Returns the steam id of the current server
    pub fn steam_id(&self) -> SteamId {
        unsafe { SteamId(sys::SteamAPI_ISteamGameServer_GetSteamID(self.server)) }
    }

    /// Retrieve an authentication session ticket that can be sent
    /// to an entity that wishes to verify you.
    ///
    /// This ticket should not be reused.
    ///
    /// When creating ticket for use by the web API you should wait
    /// for the `AuthSessionTicketResponse` event before trying to
    /// use the ticket.
    ///
    /// When the multiplayer session terminates you must call
    /// `cancel_authentication_ticket`
    pub fn authentication_session_ticket_with_steam_id(
        &self,
        steam_id: SteamId,
    ) -> (AuthTicket, Vec<u8>) {
        self.authentication_session_ticket(NetworkingIdentity::new_steam_id(steam_id))
    }
    pub fn authentication_session_ticket(
        &self,
        network_identity: NetworkingIdentity,
    ) -> (AuthTicket, Vec<u8>) {
        unsafe {
            let mut ticket = vec![0; 1024];
            let mut ticket_len = 0;
            let auth_ticket = sys::SteamAPI_ISteamGameServer_GetAuthSessionTicket(
                self.server,
                ticket.as_mut_ptr().cast(),
                1024,
                &mut ticket_len,
                network_identity.as_ptr(),
            );
            ticket.truncate(ticket_len as usize);
            (AuthTicket(auth_ticket), ticket)
        }
    }

    /// Cancels an authentication session ticket received from
    /// `authentication_session_ticket`.
    ///
    /// This should be called when you are no longer playing with
    /// the specified entity.
    pub fn cancel_authentication_ticket(&self, ticket: AuthTicket) {
        unsafe {
            sys::SteamAPI_ISteamGameServer_CancelAuthTicket(self.server, ticket.0);
        }
    }

    /// Authenticate the ticket from the steam ID to make sure it is
    /// valid and not reused.
    ///
    /// A `ValidateAuthTicketResponse` callback will be fired if
    /// the entity goes offline or cancels the ticket.
    ///
    /// When the multiplayer session terminates you must call
    /// `end_authentication_session`
    pub fn begin_authentication_session(
        &self,
        user: SteamId,
        ticket: &[u8],
    ) -> Result<(), AuthSessionError> {
        unsafe {
            let res = sys::SteamAPI_ISteamGameServer_BeginAuthSession(
                self.server,
                ticket.as_ptr().cast(),
                ticket.len() as _,
                user.0,
            );
            Err(match res {
                sys::EBeginAuthSessionResult::k_EBeginAuthSessionResultOK => return Ok(()),
                sys::EBeginAuthSessionResult::k_EBeginAuthSessionResultInvalidTicket => {
                    AuthSessionError::InvalidTicket
                }
                sys::EBeginAuthSessionResult::k_EBeginAuthSessionResultDuplicateRequest => {
                    AuthSessionError::DuplicateRequest
                }
                sys::EBeginAuthSessionResult::k_EBeginAuthSessionResultInvalidVersion => {
                    AuthSessionError::InvalidVersion
                }
                sys::EBeginAuthSessionResult::k_EBeginAuthSessionResultGameMismatch => {
                    AuthSessionError::GameMismatch
                }
                sys::EBeginAuthSessionResult::k_EBeginAuthSessionResultExpiredTicket => {
                    AuthSessionError::ExpiredTicket
                }
                _ => unreachable!(),
            })
        }
    }

    /// Ends an authentication session that was started with
    /// `begin_authentication_session`.
    ///
    /// This should be called when you are no longer playing with
    /// the specified entity.
    pub fn end_authentication_session(&self, user: SteamId) {
        unsafe {
            sys::SteamAPI_ISteamGameServer_EndAuthSession(self.server, user.0);
        }
    }

    /// Server browser related query packet processing for shared socket mode.  These are used
    /// when you pass STEAMGAMESERVER_QUERY_PORT_SHARED as the query port to SteamGameServer_Init.
    /// IP address and port are in host order, i.e 127.0.0.1 == 0x7f000001
    ///
    /// Source games use this to simplify the job of the server admins, so they
    /// don't have to open up more ports on their firewalls.
    ///
    /// Call this when a packet that starts with 0xFFFFFFFF comes in on the shared socket.
    pub fn handle_incoming_packet(&self, data: &[u8], addr: SocketAddrV4) -> bool {
        unsafe {
            let result = sys::SteamAPI_ISteamGameServer_HandleIncomingPacket(
                self.server,
                data.as_ptr() as _,
                data.len() as _,
                addr.ip().to_bits(),
                addr.port(),
            );
            return result;
        }
    }

    /// Call this function after calling handle_incoming_packet. The callback
    /// function `cb` will be called for each outgoing packet that needs to be sent
    /// to an address. The buffer must be at least 16384 bytes in size.
    pub fn get_next_outgoing_packet(&self, buffer: &mut [u8], cb: impl Fn(SocketAddrV4, &[u8])) {
        assert!(
            buffer.len() >= 16 * 1024,
            "Buffer size must be at least 16 KiB"
        );

        loop {
            let mut addr = 0u32;
            let mut port = 0u16;

            let len = unsafe {
                sys::SteamAPI_ISteamGameServer_GetNextOutgoingPacket(
                    self.server,
                    buffer.as_mut_ptr() as *mut _,
                    buffer.len() as _,
                    &mut addr,
                    &mut port,
                )
            };

            if len == 0 {
                break; // No more packets to process
            }

            let addr = SocketAddrV4::new(Ipv4Addr::from_bits(addr), port);
            cb(addr, &buffer[..len as usize]);
        }
    }

    /// Sets the game product identifier. This is currently used by the master server for version
    /// checking purposes. Converting the games app ID to a string for this is recommended.
    ///
    /// This is required for all game servers and can only be set before calling
    /// log_on() or log_on_anonymous().
    pub fn set_product(&self, product: &str) {
        let product = CString::new(product).unwrap();
        unsafe {
            sys::SteamAPI_ISteamGameServer_SetProduct(self.server, product.as_ptr());
        }
    }

    /// Sets the game description. Setting this to the full name of your game is recommended.
    ///
    /// This is required for all game servers and can only be set before calling
    /// log_on() or log_on_anonymous().
    pub fn set_game_description(&self, desc: &str) {
        let desc = CString::new(desc).unwrap();
        unsafe {
            sys::SteamAPI_ISteamGameServer_SetGameDescription(self.server, desc.as_ptr());
        }
    }

    /// Sets a string defining the "gamedata" for this server, this is optional, but if set it
    /// allows users to filter in the matchmaking/server-browser interfaces based on the value.
    ///
    /// This is usually formatted as a comma or semicolon separated list.
    ///
    /// Don't set this unless it actually changes, its only uploaded to the master once; when
    /// acknowledged.
    pub fn set_game_data(&self, data: &str) {
        let desc = CString::new(data).unwrap();
        unsafe {
            sys::SteamAPI_ISteamGameServer_SetGameData(self.server, desc.as_ptr());
        }
    }

    /// Sets whether this server is dedicated or a listen server.
    pub fn set_dedicated_server(&self, dedicated: bool) {
        unsafe {
            sys::SteamAPI_ISteamGameServer_SetDedicatedServer(self.server, dedicated);
        }
    }

    /// Login to a generic anonymous account
    pub fn log_on_anonymous(&self) {
        unsafe {
            sys::SteamAPI_ISteamGameServer_LogOnAnonymous(self.server);
        }
    }

    /// Login to a generic account by token
    pub fn log_on(&self, token: &str) {
        let token = CString::new(token).unwrap();
        unsafe {
            sys::SteamAPI_ISteamGameServer_LogOn(self.server, token.as_ptr());
        }
    }

    /// If active, updates the master server with this server's presence so players can find it via
    /// the steam matchmaking/server browser interfaces.
    pub fn enable_heartbeats(&self, active: bool) {
        unsafe {
            sys::SteamAPI_ISteamGameServer_SetAdvertiseServerActive(self.server, active);
        }
    }

    /// Indicate whether you wish to be listed on the master server list
    /// and/or respond to server browser / LAN discovery packets.
    /// The server starts with this value set to false.  You should set all
    /// relevant server parameters before enabling advertisement on the server.
    ///
    /// (This function used to be named EnableHeartbeats, so if you are wondering
    /// where that function went, it's right here.  It does the same thing as before,
    /// the old name was just confusing.)
    #[inline(always)]
    pub fn set_advertise_server_active(&self, active: bool) {
        self.enable_heartbeats(active);
    }

    /// If your game is a "mod," pass the string that identifies it.  The default is an empty
    /// string, meaning this application is the original game, not a mod.
    pub fn set_mod_dir(&self, mod_dir: &str) {
        let mod_dir = CString::new(mod_dir).unwrap();
        unsafe {
            sys::SteamAPI_ISteamGameServer_SetModDir(self.server, mod_dir.as_ptr());
        }
    }

    /// Set name of map to report in the server browser
    pub fn set_map_name(&self, map_name: &str) {
        let map_name = CString::new(map_name).unwrap();
        unsafe {
            sys::SteamAPI_ISteamGameServer_SetMapName(self.server, map_name.as_ptr());
        }
    }

    /// Set the name of server as it will appear in the server browser
    pub fn set_server_name(&self, server_name: &str) {
        let server_name = CString::new(server_name).unwrap();
        unsafe {
            sys::SteamAPI_ISteamGameServer_SetServerName(self.server, server_name.as_ptr());
        }
    }

    /// Sets the maximum number of players allowed on the server at once.
    ///
    /// This value may be changed at any time.
    pub fn set_max_players(&self, count: i32) {
        unsafe {
            sys::SteamAPI_ISteamGameServer_SetMaxPlayerCount(self.server, count);
        }
    }

    /// Sets a string defining the "gametags" for this server, this is optional, but if set it
    /// allows users to filter in the matchmaking/server-browser interfaces based on the value.
    ///
    /// This is usually formatted as a comma or semicolon separated list.
    ///
    /// Don't set this unless it actually changes, its only uploaded to the master once;
    /// when acknowledged.
    ///
    /// The new "gametags" value to set. Must not be an empty string ("").
    /// This can not be longer than 127.
    pub fn set_game_tags(&self, tags: &str) {
        assert!(tags.len() != 0, "tags must not be an empty string (\"\").");
        assert!(tags.len() < 128, "tags can not be longer than 127.");

        let tags = CString::new(tags).unwrap();
        unsafe {
            sys::SteamAPI_ISteamGameServer_SetGameTags(self.server, tags.as_ptr());
        }
    }

    /// Add/update a rules key/value pair.
    pub fn set_key_value(&self, key: &str, value: &str) {
        let key = CString::new(key).unwrap();
        let value = CString::new(value).unwrap();

        unsafe {
            sys::SteamAPI_ISteamGameServer_SetKeyValue(self.server, key.as_ptr(), value.as_ptr());
        }
    }

    /// Clears the whole list of key/values that are sent in rules queries.
    pub fn clear_all_key_values(&self) {
        unsafe {
            sys::SteamAPI_ISteamGameServer_ClearAllKeyValues(self.server);
        }
    }

    /// Let people know if your server will require a password
    pub fn set_password_protected(&self, b_password_protected: bool) {
        unsafe {
            sys::SteamAPI_ISteamGameServer_SetPasswordProtected(self.server, b_password_protected);
        }
    }

    /// Number of bots.  Default value is zero
    pub fn set_bot_player_count(&self, c_bot_players: i32) {
        unsafe {
            sys::SteamAPI_ISteamGameServer_SetBotPlayerCount(self.server, c_bot_players);
        }
    }

    /// Returns an accessor to the steam UGC interface (steam workshop)
    ///
    /// **For this to work properly, you need to call `UGC::init_for_game_server()`!**
    pub fn ugc(&self) -> UGC {
        unsafe {
            let ugc = sys::SteamAPI_SteamGameServerUGC_v021();
            debug_assert!(!ugc.is_null());
            UGC {
                ugc,
                inner: self.inner.clone(),
            }
        }
    }

    /// Returns an accessor to the steam utils interface
    pub fn utils(&self) -> Utils {
        unsafe {
            let utils = sys::SteamAPI_SteamGameServerUtils_v010();
            debug_assert!(!utils.is_null());
            Utils {
                utils: utils,
                _inner: self.inner.clone(),
            }
        }
    }

    /// Returns an accessor to the steam networking interface
    pub fn networking(&self) -> Networking {
        unsafe {
            let net = sys::SteamAPI_SteamGameServerNetworking_v006();
            debug_assert!(!net.is_null());
            Networking {
                net: net,
                _inner: self.inner.clone(),
            }
        }
    }

    pub fn networking_messages(&self) -> networking_messages::NetworkingMessages {
        unsafe {
            let net = sys::SteamAPI_SteamGameServerNetworkingMessages_SteamAPI_v002();
            debug_assert!(!net.is_null());
            networking_messages::NetworkingMessages {
                net,
                inner: self.inner.clone(),
            }
        }
    }

    pub fn networking_sockets(&self) -> networking_sockets::NetworkingSockets {
        unsafe {
            let sockets = sys::SteamAPI_SteamGameServerNetworkingSockets_SteamAPI_v012();
            debug_assert!(!sockets.is_null());
            networking_sockets::NetworkingSockets {
                sockets,
                inner: self.inner.clone(),
            }
        }
    }

    /* TODO: Buggy currently?
    /// Returns an accessor to the steam apps interface
    pub fn apps(&self) -> Apps {
        unsafe {
            let apps = sys::steam_rust_get_server_apps();
            debug_assert!(!apps.is_null());
            Apps {
                apps: apps,
                _inner: self.inner.clone(),
            }
        }
    }
    */
}

#[test]
#[serial]
fn test() {
    let (server, single) = Server::init(
        [127, 0, 0, 1].into(),
        23334,
        23335,
        ServerMode::Authentication,
        "0.0.1",
    )
    .unwrap();

    println!("{:?}", server.steam_id());

    server.set_product("steamworks-rs test");
    server.set_game_description("basic server test");
    server.set_dedicated_server(true);
    server.log_on_anonymous();

    println!("{:?}", server.steam_id());

    let _cb = server.register_callback(|v: AuthSessionTicketResponse| {
        println!("Got auth ticket response: {:?}", v.result)
    });
    let _cb = server.register_callback(|v: ValidateAuthTicketResponse| {
        println!("Got validate auth ticket response: {:?}", v)
    });

    let id = server.steam_id();
    let (auth, ticket) = server.authentication_session_ticket_with_steam_id(id);

    println!("{:?}", server.begin_authentication_session(id, &ticket));

    for _ in 0..20 {
        single.run_callbacks();
        ::std::thread::sleep(::std::time::Duration::from_millis(50));
    }

    println!("END");

    server.cancel_authentication_ticket(auth);

    for _ in 0..20 {
        single.run_callbacks();
        ::std::thread::sleep(::std::time::Duration::from_millis(50));
    }

    server.end_authentication_session(id);
}

/// Manages keeping the steam api active for servers
struct ServerManager;

impl Manager for ServerManager {
    fn get_pipe(&self) -> sys::HSteamPipe {
        // SAFETY: This is considered unsafe only because of FFI, the function is otherwise
        // always safe to call from any thread.
        unsafe { sys::SteamGameServer_GetHSteamPipe() }
    }
}

impl Drop for ServerManager {
    fn drop(&mut self) {
        // SAFETY: This is considered unsafe only because of FFI, the function is otherwise
        // always safe to call from any thread.
        unsafe { sys::SteamGameServer_Shutdown() }
    }
}

/// Called when a client has been approved to connect to this game server
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GSClientApprove {
    /// The steam ID of the user requesting a p2p session
    pub user: SteamId,
    /// Owner of the game, may be different from user when Family Sharing is being used
    pub owner: SteamId,
}

unsafe impl Callback for GSClientApprove {
    const ID: i32 = steamworks_sys::GSClientApprove_t_k_iCallback as i32;

    unsafe fn from_raw(raw: *mut c_void) -> Self {
        let val = raw.cast::<sys::GSClientApprove_t>().read_unaligned();

        GSClientApprove {
            user: SteamId(val.m_SteamID.m_steamid.m_unAll64Bits),
            owner: SteamId(val.m_OwnerSteamID.m_steamid.m_unAll64Bits),
        }
    }
}

/// Reason for when a client fails to join or is kicked from a game server.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum DenyReason {
    Invalid,
    InvalidVersion,
    Generic,
    NotLoggedOn,
    NoLicense,
    Cheater,
    LoggedInElseWhere,
    UnknownText,
    IncompatibleAnticheat,
    MemoryCorruption,
    IncompatibleSoftware,
    SteamConnectionLost,
    SteamConnectionError,
    SteamResponseTimedOut,
    SteamValidationStalled,
    SteamOwnerLeftGuestUser,
}

impl From<sys::EDenyReason> for DenyReason {
    fn from(r: sys::EDenyReason) -> Self {
        match r {
            sys::EDenyReason::k_EDenyInvalid => DenyReason::Invalid,
            sys::EDenyReason::k_EDenyInvalidVersion => DenyReason::InvalidVersion,
            sys::EDenyReason::k_EDenyGeneric => DenyReason::Generic,
            sys::EDenyReason::k_EDenyNotLoggedOn => DenyReason::NotLoggedOn,
            sys::EDenyReason::k_EDenyNoLicense => DenyReason::NoLicense,
            sys::EDenyReason::k_EDenyCheater => DenyReason::Cheater,
            sys::EDenyReason::k_EDenyLoggedInElseWhere => DenyReason::LoggedInElseWhere,
            sys::EDenyReason::k_EDenyUnknownText => DenyReason::UnknownText,
            sys::EDenyReason::k_EDenyIncompatibleAnticheat => DenyReason::IncompatibleAnticheat,
            sys::EDenyReason::k_EDenyMemoryCorruption => DenyReason::MemoryCorruption,
            sys::EDenyReason::k_EDenyIncompatibleSoftware => DenyReason::IncompatibleSoftware,
            sys::EDenyReason::k_EDenySteamConnectionLost => DenyReason::SteamConnectionLost,
            sys::EDenyReason::k_EDenySteamConnectionError => DenyReason::SteamConnectionError,
            sys::EDenyReason::k_EDenySteamResponseTimedOut => DenyReason::SteamResponseTimedOut,
            sys::EDenyReason::k_EDenySteamValidationStalled => DenyReason::SteamValidationStalled,
            sys::EDenyReason::k_EDenySteamOwnerLeftGuestUser => DenyReason::SteamOwnerLeftGuestUser,
            _ => DenyReason::Invalid,
        }
    }
}

/// Called when a user has been denied to connection to this game server
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GSClientDeny {
    /// The steam ID of the user requesting a p2p session
    pub user: SteamId,
    pub deny_reason: DenyReason,
    pub optional_text: String,
}

unsafe impl Callback for GSClientDeny {
    const ID: i32 = steamworks_sys::GSClientDeny_t_k_iCallback as i32;

    unsafe fn from_raw(raw: *mut c_void) -> Self {
        let val = raw.cast::<sys::GSClientDeny_t>().read_unaligned();

        let deny_text = unsafe {
            let cstr = CStr::from_ptr(val.m_rgchOptionalText.as_ptr() as *const c_char);
            cstr.to_string_lossy().to_owned().into_owned()
        };

        GSClientDeny {
            user: SteamId(val.m_SteamID.m_steamid.m_unAll64Bits),
            deny_reason: DenyReason::from(val.m_eDenyReason),
            optional_text: deny_text,
        }
    }
}

/// Called when a user has been denied to connection to this game server
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GSClientKick {
    /// The steam ID of the user requesting a p2p session
    pub user: SteamId,
    pub deny_reason: DenyReason,
}

unsafe impl Callback for GSClientKick {
    const ID: i32 = steamworks_sys::GSClientKick_t_k_iCallback as i32;

    unsafe fn from_raw(raw: *mut c_void) -> Self {
        let val = raw.cast::<sys::GSClientKick_t>().read_unaligned();

        GSClientKick {
            user: SteamId(val.m_SteamID.m_steamid.m_unAll64Bits),
            deny_reason: DenyReason::from(val.m_eDenyReason),
        }
    }
}

/// Called when we have received the group status of a user
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GSClientGroupStatus {
    /// The steam ID of the user we queried
    pub user: SteamId,
    pub group: SteamId,
    pub member: bool,
    pub officer: bool,
}

unsafe impl Callback for GSClientGroupStatus {
    const ID: i32 = steamworks_sys::GSClientGroupStatus_t_k_iCallback as i32;

    unsafe fn from_raw(raw: *mut c_void) -> Self {
        let val = raw.cast::<sys::GSClientGroupStatus_t>().read_unaligned();

        GSClientGroupStatus {
            user: SteamId(val.m_SteamIDUser.m_steamid.m_unAll64Bits),
            group: SteamId(val.m_SteamIDGroup.m_steamid.m_unAll64Bits),
            member: val.m_bMember,
            officer: val.m_bOfficer,
        }
    }
}
