use super::*;

use std::net::Ipv4Addr;

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
    ) -> SResult<Server> {
        unsafe {
            let version = CString::new(version).unwrap();
            let raw_ip: u32 = ip.into();
            let server_mode = match server_mode {
                ServerMode::NoAuthentication => sys::ServerMode::NoAuthentication,
                ServerMode::Authentication => sys::ServerMode::Authentication,
                ServerMode::AuthenticationAndSecure => sys::ServerMode::AuthenticationAndSecure,
            };
            if sys::steam_rust_game_server_init(
                raw_ip, steam_port,
                game_port, query_port,
                server_mode,
                version.as_ptr() as *const _,
            ) == 0 {
                return Err(SteamError::InitFailed);
            }
            let server_raw = sys::steam_rust_get_server();
            let server = Arc::new(Inner {
                _manager: ServerManager { _priv: () },
                callbacks: Mutex::new(Callbacks {
                    callbacks: Vec::new(),
                    call_results: HashMap::new(),
                }),
            });
            Ok(Server {
                inner: server,
                server: server_raw,
            })
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
        unsafe {
            sys::SteamGameServer_RunCallbacks();
        }
    }

    /// Registers the passed function as a callback for the
    /// given type.
    ///
    /// The callback will be run on the thread that `run_callbacks`
    /// is called when the event arrives.
    pub fn register_callback<C, F>(&self, f: F)
        where C: Callback,
              F: FnMut(C) + 'static + Send + Sync
    {
        unsafe {
            register_callback(&self.inner, f, true);
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
                sys::BeginAuthSessionResult::Ok => return Ok(()),
                sys::BeginAuthSessionResult::InvalidTicket => AuthSessionError::InvalidTicket,
                sys::BeginAuthSessionResult::DuplicateRequest => AuthSessionError::DuplicateRequest,
                sys::BeginAuthSessionResult::InvalidVersion => AuthSessionError::InvalidVersion,
                sys::BeginAuthSessionResult::GameMismatch => AuthSessionError::GameMismatch,
                sys::BeginAuthSessionResult::ExpiredTicket => AuthSessionError::ExpiredTicket,
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

    /// Sets the game product identifier.
    ///
    /// Used by the master server for version checking. Required
    /// field but it will go away eventually.
    pub fn set_product(&self, product: &str) {
        unsafe {
            let product = CString::new(product).unwrap();
            sys::SteamAPI_ISteamGameServer_SetProduct(self.server, product.as_ptr() as *const _);
        }
    }

    /// Sets the game description.
    ///
    /// Displayed in the steam server browser (for now). Required
    /// field but it will go away eventually.
    pub fn set_game_description(&self, desc: &str) {
        unsafe {
            let desc = CString::new(desc).unwrap();
            sys::SteamAPI_ISteamGameServer_SetGameDescription(self.server, desc.as_ptr() as *const _);
        }
    }

    /// Sets whether this server is dedicated or a listen server.
    pub fn set_dedicated_server(&self, dedicated: bool) {
        unsafe {
            sys::SteamAPI_ISteamGameServer_SetDedicatedServer(self.server, dedicated as u8);
        }
    }

    /// Login to a generic anonymous account
    pub fn log_on_anonymous(&self) {
        unsafe {
            sys::SteamAPI_ISteamGameServer_LogOnAnonymous(self.server);
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
fn test() {
    let server = Server::init(
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

    server.register_callback(|v: AuthSessionTicketResponse| println!("{:?}", v));
    server.register_callback(|v: ValidateAuthTicketResponse| println!("{:?}", v));

    let id = server.steam_id();
    let (auth, ticket) = server.authentication_session_ticket();

    println!("{:?}", server.begin_authentication_session(id, &ticket));

    for _ in 0 .. 20 {
        server.run_callbacks();
        ::std::thread::sleep(::std::time::Duration::from_millis(50));
    }

    println!("END");

    server.cancel_authentication_ticket(auth);

    for _ in 0 .. 20 {
        server.run_callbacks();
        ::std::thread::sleep(::std::time::Duration::from_millis(50));
    }

    server.end_authentication_session(id);
}


/// Manages keeping the steam api active for servers
pub struct ServerManager {
    _priv: (),
}

impl Drop for ServerManager {
    fn drop(&mut self) {
        unsafe {
            sys::SteamGameServer_Shutdown();
        }
    }
}