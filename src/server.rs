use super::*;
use std::net::Ipv4Addr;
#[cfg(test)]
use serial_test_derive::serial;

/// The main entry point into the steam client for servers.
///
/// This provides access to all of the steamworks api that
/// servers can use.
#[derive(Clone)]
pub struct Server {
    inner: Arc<Inner<ServerManager>>,
    server: *mut sys::ISteamGameServer,
}

unsafe impl Send for Server {}
unsafe impl Sync for Server {}

/// Used to set the mode that a gameserver will run in
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

impl Server {
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
        ip: Ipv4Addr, steam_port: u16,
        game_port: u16, query_port: u16,
        server_mode: ServerMode, version: &str,
    ) -> SResult<(Server, SingleClient<ServerManager>)> {
        unsafe {
            let version = CString::new(version).unwrap();
            let raw_ip: u32 = ip.into();
            let server_mode = match server_mode {
                ServerMode::NoAuthentication => sys::EServerMode::eServerModeNoAuthentication,
                ServerMode::Authentication => sys::EServerMode::eServerModeAuthentication,
                ServerMode::AuthenticationAndSecure => sys::EServerMode::eServerModeAuthenticationAndSecure,
            };
            if !sys::SteamInternal_GameServer_Init(
                raw_ip, steam_port,
                game_port, query_port,
                server_mode,
                version.as_ptr(),
            ) {
                return Err(SteamError::InitFailed);
            }
            sys::SteamAPI_ManualDispatch_Init();
            let server_raw = sys::SteamAPI_SteamGameServer_v014();
            let server = Arc::new(Inner {
                _manager: ServerManager { _priv: () },
                callbacks: Mutex::new(Callbacks {
                    callbacks: HashMap::new(),
                    call_results: HashMap::new(),
                }),
                networking_sockets_data: Mutex::new(NetworkingSocketsData {
                    sockets: Default::default(),
                    independent_connections: Default::default(),
                    connection_callback: Default::default()
                })
            });
            Ok((Server {
                inner: server.clone(),
                server: server_raw,
            }, SingleClient {
                inner: server,
                _not_sync: PhantomData,
            }))
        }
    }

    /// Registers the passed function as a callback for the
    /// given type.
    ///
    /// The callback will be run on the thread that `run_callbacks`
    /// is called when the event arrives.
    pub fn register_callback<C, F>(&self, f: F) -> CallbackHandle<ServerManager>
        where C: Callback,
              F: FnMut(C) + 'static + Send
    {
        unsafe {
            register_callback(&self.inner, f)
        }
    }

    /// Returns the steam id of the current server
    pub fn steam_id(&self) -> SteamId {
        unsafe {
            SteamId(sys::SteamAPI_ISteamGameServer_GetSteamID(self.server))
        }
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
    pub fn authentication_session_ticket(&self) -> (AuthTicket, Vec<u8>) {
        unsafe {
            let mut ticket = vec![0; 1024];
            let mut ticket_len = 0;
            let auth_ticket = sys::SteamAPI_ISteamGameServer_GetAuthSessionTicket(self.server, ticket.as_mut_ptr() as *mut _, 1024, &mut ticket_len);
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
    pub fn begin_authentication_session(&self, user: SteamId, ticket: &[u8]) -> Result<(), AuthSessionError> {
        unsafe {
            let res = sys::SteamAPI_ISteamGameServer_BeginAuthSession(
                self.server,
                ticket.as_ptr() as *const _, ticket.len() as _,
                user.0
            );
            Err(match res {
                sys::EBeginAuthSessionResult::k_EBeginAuthSessionResultOK => return Ok(()),
                sys::EBeginAuthSessionResult::k_EBeginAuthSessionResultInvalidTicket => AuthSessionError::InvalidTicket,
                sys::EBeginAuthSessionResult::k_EBeginAuthSessionResultDuplicateRequest => AuthSessionError::DuplicateRequest,
                sys::EBeginAuthSessionResult::k_EBeginAuthSessionResultInvalidVersion => AuthSessionError::InvalidVersion,
                sys::EBeginAuthSessionResult::k_EBeginAuthSessionResultGameMismatch => AuthSessionError::GameMismatch,
                sys::EBeginAuthSessionResult::k_EBeginAuthSessionResultExpiredTicket => AuthSessionError::ExpiredTicket,
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

    /// Sets the game product identifier. This is currently used by the master server for version
    /// checking purposes. Converting the games app ID to a string for this is recommended.
    ///
    /// This is required for all game servers and can only be set before calling
    /// log_on() or log_on_anonymous().
    pub fn set_product(&self, product: &str) {
        let product = CString::new(product).unwrap();
        unsafe {
            sys::SteamAPI_ISteamGameServer_SetProduct(self.server, product.as_ptr() as *const _);
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

    /// If active, updates the master server with this server's presence so players can find it via
    /// the steam matchmaking/server browser interfaces.
    pub fn enable_heartbeats(&self, active: bool) {
        unsafe {
            sys::SteamAPI_ISteamGameServer_SetAdvertiseServerActive(self.server, active);
        }
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
            sys::SteamAPI_ISteamGameServer_SetMapName(self.server, server_name.as_ptr());
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

    /// Returns an accessor to the steam UGC interface (steam workshop)
    /// 
    /// **For this to work properly, you need to call `UGC::init_for_game_server()`!**
    pub fn ugc(&self) -> UGC<ServerManager> {
        unsafe {
            let ugc = sys::SteamAPI_SteamGameServerUGC_v016();
            debug_assert!(!ugc.is_null());
            UGC {
                ugc,
                inner: self.inner.clone(),
            }
        }
    }

    /* TODO: Buggy currently?
    /// Returns an accessor to the steam apps interface
    pub fn apps(&self) -> Apps<ServerManager> {
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
        23333, 23334, 23335,
        ServerMode::Authentication, "0.0.1"
    ).unwrap();

    println!("{:?}", server.steam_id());

    server.set_product("steamworks-rs test");
    server.set_game_description("basic server test");
    server.set_dedicated_server(true);
    server.log_on_anonymous();

    println!("{:?}", server.steam_id());

    let _cb = server.register_callback(|v: AuthSessionTicketResponse| println!("Got response: {:?}", v.result));
    let _cb = server.register_callback(|v: ValidateAuthTicketResponse| println!("{:?}", v));

    let id = server.steam_id();
    let (auth, ticket) = server.authentication_session_ticket();

    println!("{:?}", server.begin_authentication_session(id, &ticket));

    for _ in 0 .. 20 {
        single.run_callbacks();
        ::std::thread::sleep(::std::time::Duration::from_millis(50));
    }

    println!("END");

    server.cancel_authentication_ticket(auth);

    for _ in 0 .. 20 {
        single.run_callbacks();
        ::std::thread::sleep(::std::time::Duration::from_millis(50));
    }

    server.end_authentication_session(id);
}


/// Manages keeping the steam api active for servers
pub struct ServerManager {
    _priv: (),
}

unsafe impl Manager for ServerManager {
    unsafe fn get_pipe() -> sys::HSteamPipe {
        sys::SteamGameServer_GetHSteamPipe()
    }
}

impl Drop for ServerManager {
    fn drop(&mut self) {
        unsafe {
            sys::SteamGameServer_Shutdown();
        }
    }
}
