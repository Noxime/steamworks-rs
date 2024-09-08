use std::net::Ipv4Addr;
use std::ptr;
use std::rc::Rc;
use std::time::Duration;

use super::*;

macro_rules! matchmaking_servers_callback {
    (
        $name:ident;
        $self:ident;
        ($($additional_name:ident : $additional_type:ty where $additional_content:block),*);
        $(
            $fn_name:ident($clear_after_call:tt): ( $( $fn_arg_name:ident: $cpp_fn_arg:ty => $rust_fn_arg:ty where $normalize:tt ),* )
        ),*
    ) => {
        paste::item! {
            $(
                #[allow(unused_variables)]
                extern fn [<$name:lower _ $fn_name _virtual>]($self: *mut [<$name CallbacksReal>] $(, $fn_arg_name: $cpp_fn_arg)*) {
                    unsafe {
                        $(
                            #[allow(unused_parens)]
                            let [<$fn_arg_name _norm>]: $rust_fn_arg = $normalize;
                        )*
                        // In case of dropping rust_callbacks inside $fn_name
                        let rc_fn = Rc::clone(&(*(*$self).rust_callbacks).$fn_name);
                        (*rc_fn)($([<$fn_arg_name _norm>]),*);
                        $clear_after_call;
                    }
                }
            )*

            pub struct [<$name Callbacks>] {
                $(
                    pub (self) $fn_name: Rc<Box<dyn Fn($($rust_fn_arg),*)>>,
                )*
                $(
                    pub (self) $additional_name: $additional_type,
                )*
            }

            impl [<$name Callbacks>] {
                // Arc can also be here, without Box
                pub fn new($($fn_name: Box<dyn Fn($($rust_fn_arg),*)>),*) -> Self {
                    Self {
                        $($fn_name: Rc::new($fn_name),)*
                        $($additional_name: $additional_content,)*
                    }
                }
            }

            #[repr(C)]
            struct [<$name CallbacksReal>] {
                pub vtable: *mut [<$name CallbacksVirtual>],
                pub rust_callbacks: *mut [<$name Callbacks>],
            }

            #[repr(C)]
            struct [<$name CallbacksVirtual>] {
                $(
                    pub $fn_name: extern fn(*mut [<$name CallbacksReal>] $(, $cpp_fn_arg)*)
                ),*
            }

            unsafe fn [<create_ $name:lower>](rust_callbacks: [<$name Callbacks>]) -> *mut [<$name CallbacksReal>] {
                let rust_callbacks = Box::into_raw(Box::new(rust_callbacks));
                let vtable = Box::into_raw(Box::new([<$name CallbacksVirtual>] {
                    $(
                        $fn_name: [<$name:lower _ $fn_name _virtual>]
                    ),*
                }));
                let real = Box::into_raw(Box::new([<$name CallbacksReal>] {
                    vtable,
                    rust_callbacks,
                }));

                real
            }

            unsafe fn [<free_ $name:lower>](real: *mut [<$name CallbacksReal>]) {
                drop(Box::from_raw((*real).rust_callbacks));
                drop(Box::from_raw((*real).vtable));
                drop(Box::from_raw(real));
            }
        }
    };
}

macro_rules! gen_server_list_fn {
    (
        $name:ident, $sys_method:ident
    ) => {
        /// # Usage
        ///
        /// Request must be released at the end of using. For more details see [`ServerListRequest::release`]
        ///
        /// # Arguments
        ///
        /// * app_id: The app to request the server list of.
        /// * filters: An array of filters to only retrieve servers the user cares about.
        /// A list of the keys & values can be found
        /// [here](https://partner.steamgames.com/doc/api/ISteamMatchmakingServers#MatchMakingKeyValuePair_t).
        ///
        /// # Errors
        ///
        /// Every filter's key and value must take 255 bytes or under, otherwise `Err` is returned.
        pub fn $name<ID: Into<AppId>>(
            &self,
            app_id: ID,
            filters: &HashMap<&str, &str>,
            callbacks: ServerListCallbacks,
        ) -> Result<Arc<Mutex<ServerListRequest>>, ()> {
            let app_id = app_id.into().0;
            let mut filters = {
                let mut vec = Vec::with_capacity(filters.len());
                for i in filters {
                    let key_bytes = i.0.as_bytes();
                    let value_bytes = i.1.as_bytes();

                    // Max length is 255, so 256th byte will always be nul-terminator
                    if key_bytes.len() >= 256 || value_bytes.len() >= 256 {
                        return Err(());
                    }

                    let mut key = [0i8; 256];
                    let mut value = [0i8; 256];

                    unsafe {
                        key.as_mut_ptr()
                            .copy_from(key_bytes.as_ptr().cast(), key_bytes.len());
                        value
                            .as_mut_ptr()
                            .copy_from(value_bytes.as_ptr().cast(), value_bytes.len());
                    }

                    vec.push(sys::MatchMakingKeyValuePair_t {
                        m_szKey: key,
                        m_szValue: value,
                    });
                }
                vec.shrink_to_fit();

                vec
            };

            unsafe {
                let callbacks = create_serverlist(callbacks);

                let request_arc = ServerListRequest::get(callbacks);
                let mut request = request_arc.lock().unwrap();

                let handle = sys::$sys_method(
                    self.mms,
                    app_id,
                    &mut filters.as_mut_ptr() as *mut *mut _,
                    filters.len().try_into().unwrap(),
                    callbacks.cast(),
                );

                request.mms = self.mms;
                request.real = callbacks;
                request.h_req = handle;

                drop(request);

                Ok(request_arc)
            }
        }
    };
}

pub struct GameServerItem {
    pub appid: u32,
    pub players: i32,
    pub do_not_refresh: bool,
    pub successful_response: bool,
    pub have_password: bool,
    pub secure: bool,
    pub bot_players: i32,
    pub ping: Duration,
    pub max_players: i32,
    pub server_version: i32,
    pub steamid: u64,
    pub last_time_played: Duration,
    pub addr: Ipv4Addr,
    pub query_port: u16,
    pub connection_port: u16,
    pub game_description: String,
    pub server_name: String,
    pub game_dir: String,
    pub map: String,
    pub tags: String,
}

impl GameServerItem {
    unsafe fn from_ptr(raw: *const sys::gameserveritem_t) -> Self {
        let raw = *raw;
        Self {
            appid: raw.m_nAppID,
            players: raw.m_nPlayers,
            bot_players: raw.m_nBotPlayers,
            ping: Duration::from_millis(raw.m_nPing.try_into().unwrap()),
            max_players: raw.m_nMaxPlayers,
            server_version: raw.m_nServerVersion,
            steamid: raw.m_steamID.m_steamid.m_unAll64Bits,

            do_not_refresh: raw.m_bDoNotRefresh,
            successful_response: raw.m_bHadSuccessfulResponse,
            have_password: raw.m_bPassword,
            secure: raw.m_bSecure,

            addr: Ipv4Addr::from(raw.m_NetAdr.m_unIP),
            query_port: raw.m_NetAdr.m_usQueryPort,
            connection_port: raw.m_NetAdr.m_usConnectionPort,

            game_description: CStr::from_ptr(raw.m_szGameDescription.as_ptr())
                .to_string_lossy()
                .into_owned(),
            server_name: CStr::from_ptr(raw.m_szServerName.as_ptr())
                .to_string_lossy()
                .into_owned(),
            game_dir: CStr::from_ptr(raw.m_szGameDir.as_ptr())
                .to_string_lossy()
                .into_owned(),
            map: CStr::from_ptr(raw.m_szMap.as_ptr())
                .to_string_lossy()
                .into_owned(),
            tags: CStr::from_ptr(raw.m_szGameTags.as_ptr())
                .to_string_lossy()
                .into_owned(),

            last_time_played: Duration::from_secs(raw.m_ulTimeLastPlayed.into()),
        }
    }
}

matchmaking_servers_callback!(
    Ping;
    _self;
    ();
    responded({}): (info: *const sys::gameserveritem_t => GameServerItem where { GameServerItem::from_ptr(info) }),
    failed({ free_ping(_self) }): ()
);

matchmaking_servers_callback!(
    PlayerDetails;
    _self;
    ();
    add_player({}): (
        name: *const std::os::raw::c_char => &CStr where { CStr::from_ptr(name) },
        score: i32 => i32 where {score},
        time_played: f32 => f32 where {time_played}
    ),
    failed({ free_playerdetails(_self) }): (),
    refresh_complete({ free_playerdetails(_self) }): ()
);

matchmaking_servers_callback!(
    ServerRules;
    _self;
    ();
    add_rule({}): (
        rule: *const std::os::raw::c_char => &CStr where { CStr::from_ptr(rule) },
        value: *const std::os::raw::c_char => &CStr where { CStr::from_ptr(value) }
    ),
    failed({ free_serverrules(_self) }): (),
    refresh_complete({ free_serverrules(_self) }): ()
);

matchmaking_servers_callback!(
    ServerList;
    _self;
    (
        req: Arc<Mutex<ServerListRequest>> where {
            Arc::new(Mutex::new(ServerListRequest {
                h_req: ptr::null_mut(),
                released: false,
                mms: ptr::null_mut(),
                real: ptr::null_mut(),
            }))
        }
    );
    responded({}): (
        request: sys::HServerListRequest => Arc<Mutex<ServerListRequest>> where { ServerListRequest::get(_self) },
        server: i32 => i32 where {server}
    ),
    failed({}): (
        request: sys::HServerListRequest => Arc<Mutex<ServerListRequest>> where { ServerListRequest::get(_self) },
        server: i32 => i32 where {server}
    ),
    refresh_complete({}): (
        request: sys::HServerListRequest => Arc<Mutex<ServerListRequest>> where { ServerListRequest::get(_self) },
        response: ServerResponse => ServerResponse where {response}
    )
);

#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum ServerResponse {
    ServerResponded = 0,
    ServerFailedToRespond = 1,
    NoServersListedOnMasterServer = 2,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum ReleaseError {
    /// Further using methods on this request after `release`
    /// called will always result in `Err(ReleseError::Released)`.
    Released,
    /// Due to wrapper limitations releasing request while query
    /// is still refreshing (`is_refreshing()`) is impossible.
    /// `Err(ReleaseError::Refreshing)` will be returned.
    Refreshing,
}

pub struct ServerListRequest {
    pub(self) h_req: sys::HServerListRequest,
    pub(self) released: bool,
    pub(self) mms: *mut sys::ISteamMatchmakingServers,
    pub(self) real: *mut ServerListCallbacksReal,
}

unsafe impl Send for ServerListRequest {}

impl ServerListRequest {
    pub(self) unsafe fn get(_self: *mut ServerListCallbacksReal) -> Arc<Mutex<Self>> {
        let rust_callbacks = &*(*_self).rust_callbacks;
        Arc::clone(&rust_callbacks.req)
    }

    /// # Usage
    ///
    /// Cancels any pending query on it if there's a pending
    /// query in progress. Releasing all heap allocated
    /// structures used for callbacks. The `refresh_complete`
    /// callback will not be posted when request is released.
    ///
    /// Further using methods on this request after `release`
    /// called will always result in `Err(ReleseError::Released)`.
    ///
    /// Due to wrapper limitations releasing request while query
    /// is still refreshing (`is_refreshing()`) is impossible.
    /// `Err(ReleaseError::Refreshing)` will be returned.
    pub fn release(&mut self) -> Result<(), ReleaseError> {
        unsafe {
            if self.released {
                return Err(ReleaseError::Released);
            }
            if sys::SteamAPI_ISteamMatchmakingServers_IsRefreshing(self.mms, self.h_req) {
                return Err(ReleaseError::Refreshing);
            }

            self.released = true;
            sys::SteamAPI_ISteamMatchmakingServers_ReleaseRequest(self.mms, self.h_req);

            free_serverlist(self.real);

            Ok(())
        }
    }

    fn released(&self) -> Result<(), ()> {
        if self.released {
            Err(())
        } else {
            Ok(())
        }
    }

    /// # Errors
    ///
    /// Err if called on the released request
    pub fn get_server_count(&self) -> Result<i32, ()> {
        unsafe {
            self.released()?;

            Ok(sys::SteamAPI_ISteamMatchmakingServers_GetServerCount(
                self.mms, self.h_req,
            ))
        }
    }

    /// # Errors
    ///
    /// Err if called on the released request
    pub fn get_server_details(&self, server: i32) -> Result<GameServerItem, ()> {
        unsafe {
            self.released()?;

            // Should we then free this pointer?
            let server_item = sys::SteamAPI_ISteamMatchmakingServers_GetServerDetails(
                self.mms, self.h_req, server,
            );

            Ok(GameServerItem::from_ptr(server_item))
        }
    }

    /// # Errors
    ///
    /// Err if called on the released request
    pub fn refresh_query(&self) -> Result<(), ()> {
        unsafe {
            self.released()?;

            sys::SteamAPI_ISteamMatchmakingServers_RefreshQuery(self.mms, self.h_req);

            Ok(())
        }
    }

    /// # Errors
    ///
    /// Err if called on the released request
    pub fn refresh_server(&self, server: i32) -> Result<(), ()> {
        unsafe {
            self.released()?;

            sys::SteamAPI_ISteamMatchmakingServers_RefreshServer(self.mms, self.h_req, server);

            Ok(())
        }
    }

    /// # Errors
    ///
    /// Err if called on the released request
    pub fn is_refreshing(&self) -> Result<bool, ()> {
        unsafe {
            self.released()?;

            Ok(sys::SteamAPI_ISteamMatchmakingServers_IsRefreshing(
                self.mms, self.h_req,
            ))
        }
    }
}

/// Access to the steam MatchmakingServers interface
pub struct MatchmakingServers<Manager> {
    pub(crate) mms: *mut sys::ISteamMatchmakingServers,
    pub(crate) _inner: Arc<Inner<Manager>>,
}

impl<Manager> MatchmakingServers<Manager> {
    pub fn ping_server(&self, ip: std::net::Ipv4Addr, port: u16, callbacks: PingCallbacks) {
        unsafe {
            let callbacks = create_ping(callbacks);

            sys::SteamAPI_ISteamMatchmakingServers_PingServer(
                self.mms,
                ip.into(),
                port,
                callbacks.cast(),
            );
        }
    }

    pub fn player_details(
        &self,
        ip: std::net::Ipv4Addr,
        port: u16,
        callbacks: PlayerDetailsCallbacks,
    ) {
        unsafe {
            let callbacks = create_playerdetails(callbacks);

            sys::SteamAPI_ISteamMatchmakingServers_PlayerDetails(
                self.mms,
                ip.into(),
                port,
                callbacks.cast(),
            );
        }
    }

    pub fn server_rules(&self, ip: std::net::Ipv4Addr, port: u16, callbacks: ServerRulesCallbacks) {
        unsafe {
            let callbacks = create_serverrules(callbacks);

            sys::SteamAPI_ISteamMatchmakingServers_ServerRules(
                self.mms,
                ip.into(),
                port,
                callbacks.cast(),
            );
        }
    }

    /// # Usage
    ///
    /// Request must be released at the end of using. For more details see [`ServerListRequest::release`]
    ///
    /// # Arguments
    ///
    /// * app_id: The app to request the server list of.
    pub fn lan_server_list<ID: Into<AppId>>(
        &self,
        app_id: ID,
        callbacks: ServerListCallbacks,
    ) -> Arc<Mutex<ServerListRequest>> {
        unsafe {
            let app_id = app_id.into().0;

            let callbacks = create_serverlist(callbacks);

            let request_arc = ServerListRequest::get(callbacks);
            let mut request = request_arc.lock().unwrap();

            let handle = sys::SteamAPI_ISteamMatchmakingServers_RequestLANServerList(
                self.mms,
                app_id,
                callbacks.cast(),
            );

            request.mms = self.mms;
            request.real = callbacks;
            request.h_req = handle;

            drop(request);

            request_arc
        }
    }

    gen_server_list_fn!(
        internet_server_list,
        SteamAPI_ISteamMatchmakingServers_RequestInternetServerList
    );
    gen_server_list_fn!(
        favorites_server_list,
        SteamAPI_ISteamMatchmakingServers_RequestFavoritesServerList
    );
    gen_server_list_fn!(
        history_server_list,
        SteamAPI_ISteamMatchmakingServers_RequestHistoryServerList
    );
    gen_server_list_fn!(
        friends_server_list,
        SteamAPI_ISteamMatchmakingServers_RequestFriendsServerList
    );
}
