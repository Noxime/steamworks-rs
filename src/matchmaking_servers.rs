use std::ptr;
use std::rc::Rc;
use std::time::Duration;
use std::net::Ipv4Addr;

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
        /// Every filter's key and value must take 255 bytes or under, otherwise `None` is returned.
        pub fn $name<ID: Into<AppId>>(
            &self,
            app_id: ID,
            filters: &HashMap<&str, &str>,
            callbacks: ServerListCallbacks
        ) -> Option<Arc<Mutex<ServerListRequest>>> {
            unsafe {
                let app_id = app_id.into().0;
    
                let mut filters = create_filters(filters)?;
                let callbacks = create_serverlist(callbacks);
                
                let arc = Arc::clone(&(*(*callbacks).rust_callbacks).req);
                let mut req = arc.lock().unwrap();
    
                let handle = steamworks_sys::$sys_method(
                    self.mms,
                    app_id,
                    &mut filters.0,
                    filters.1.try_into().unwrap(),
                    callbacks.cast()
                );
    
                req.mms = self.mms;
                req.real = callbacks;
                req.filters = filters;
                req.h_req = handle;
    
                drop(req);
    
                Some(arc)
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
    pub ping: i32,
    pub max_players: i32,
    pub server_version: i32,
    pub steamid: u64,
    pub last_time_played: std::time::Duration,
    pub addr: Ipv4Addr,
    pub query_port: u16,
    pub connection_port: u16,
    pub game_description: String,
    pub server_name: String,
    pub game_dir: String,
    pub map: String,
}

impl GameServerItem {
    fn from_ptr(raw: *const steamworks_sys::gameserveritem_t) -> Self {
        unsafe {
            let raw = *raw;
            Self {
                appid: raw.m_nAppID,
                players: raw.m_nPlayers,
                bot_players: raw.m_nBotPlayers,
                ping: raw.m_nPing,
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
            
                game_description: CStr::from_ptr(raw.m_szGameDescription.as_ptr()).to_string_lossy().into_owned(),
                server_name: CStr::from_ptr(raw.m_szServerName.as_ptr()).to_string_lossy().into_owned(),
                game_dir: CStr::from_ptr(raw.m_szGameDir.as_ptr()).to_string_lossy().into_owned(),
                map: CStr::from_ptr(raw.m_szMap.as_ptr()).to_string_lossy().into_owned(),
            
                last_time_played: Duration::from_secs(raw.m_ulTimeLastPlayed.into())
            }
        }
    }
}

matchmaking_servers_callback!(
    Ping;
    _self;
    ();
    responded({}): (info: *const steamworks_sys::gameserveritem_t => GameServerItem where { GameServerItem::from_ptr(info) }),
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
                filters: (std::ptr::null_mut(), 0),
            }))
        }
    );
    responded({}): (
        request: steamworks_sys::HServerListRequest => Arc<Mutex<ServerListRequest>> where { ServerListRequest::get(_self, request) },
        server: i32 => i32 where {server}
    ),
    failed({}): (
        request: steamworks_sys::HServerListRequest => Arc<Mutex<ServerListRequest>> where { ServerListRequest::get(_self, request) },
        server: i32 => i32 where {server}
    ),
    refresh_complete({}): (
        request: steamworks_sys::HServerListRequest => Arc<Mutex<ServerListRequest>> where { ServerListRequest::get(_self, request) },
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

pub struct ServerListRequest {
    pub (self) h_req: steamworks_sys::HServerListRequest,
    pub (self) released: bool,
    pub (self) mms: *mut sys::ISteamMatchmakingServers,
    pub (self) real: *mut ServerListCallbacksReal,
    pub (self) filters: (*mut steamworks_sys::MatchMakingKeyValuePair_t, usize),
}

impl ServerListRequest {
    pub (self) unsafe fn get(
        _self: *mut ServerListCallbacksReal,
        _request: steamworks_sys::HServerListRequest
    ) -> Arc<Mutex<Self>> {
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
    /// called will always result in `None`
    pub fn release(&mut self) {
        unsafe {
            if self.released {
                return;
            }

            self.released = true;
            steamworks_sys::SteamAPI_ISteamMatchmakingServers_ReleaseRequest(
                self.mms,
                self.h_req
            );

            if !self.filters.0.is_null() {
                free_filters(self.filters.0, self.filters.1);
            }
            
            free_serverlist(self.real);
        }
    }

    fn released(&self) -> Option<()> {
        if self.released {
            None
        } else {
            Some(())
        }
    }

    /// # Errors
    /// 
    /// None if called on the released request
    pub fn get_server_count(&self) -> Option<i32> {
        unsafe {
            self.released()?;

            Some(steamworks_sys::SteamAPI_ISteamMatchmakingServers_GetServerCount(
                self.mms,
                self.h_req
            ))
        }
    }
    
    /// # Errors
    /// 
    /// None if called on the released request
    pub fn get_server_details(&self, server: i32) -> Option<GameServerItem> {
        unsafe {
            self.released()?;

            // Should we then free this pointer?
            let server_item = steamworks_sys::SteamAPI_ISteamMatchmakingServers_GetServerDetails(
                self.mms,
                self.h_req,
                server
            );
            
            Some(GameServerItem::from_ptr(server_item))
        }
    }
    
    /// # Errors
    /// 
    /// None if called on the released request
    pub fn refresh_query(&self) -> Option<()> {
        unsafe {
            self.released()?;

            steamworks_sys::SteamAPI_ISteamMatchmakingServers_RefreshQuery(
                self.mms,
                self.h_req,
            );

            Some(())
        }
    }
    
    /// # Errors
    /// 
    /// None if called on the released request
    pub fn refresh_server(&self, server: i32) -> Option<()> {
        unsafe {
            self.released()?;

            steamworks_sys::SteamAPI_ISteamMatchmakingServers_RefreshServer(
                self.mms,
                self.h_req,
                server,
            );

            Some(())
        }
    }
    
    /// # Errors
    /// 
    /// None if called on the released request
    pub fn is_refreshing(&self) -> Option<bool> {
        unsafe {
            self.released()?;

            Some(steamworks_sys::SteamAPI_ISteamMatchmakingServers_IsRefreshing(
                self.mms,
                self.h_req,
            ))
        }
    }
}

unsafe fn create_filters(map: &HashMap<&str, &str>) -> Option<(*mut steamworks_sys::MatchMakingKeyValuePair_t, usize)> {
    let mut vec = Vec::with_capacity(map.len());
    for i in map {
        let key_bytes = i.0.as_bytes();
        let value_bytes = i.1.as_bytes();

        // Max length is 255, so 256th byte will always be nul-terminator
        if key_bytes.len() >= 256 || value_bytes.len() >= 256 {
            return None;
        }

        let mut key = [0i8; 256];
        let mut value = [0i8; 256];

        key.as_mut_ptr().copy_from(key_bytes.as_ptr().cast(), key_bytes.len());
        value.as_mut_ptr().copy_from(value_bytes.as_ptr().cast(), value_bytes.len());

        vec.push(steamworks_sys::MatchMakingKeyValuePair_t {
            m_szKey: key,
            m_szValue: value,
        });
    }
    vec.shrink_to_fit();
    let filters = (vec.as_mut_ptr(), vec.len());
    std::mem::forget(vec);

    Some(filters)
}

unsafe fn free_filters(ptr: *mut steamworks_sys::MatchMakingKeyValuePair_t, count: usize) {
    drop(Vec::from_raw_parts(ptr, count, count))
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
        
            steamworks_sys::SteamAPI_ISteamMatchmakingServers_PingServer(
                self.mms,
                ip.into(),
                port,
                callbacks.cast(),
            );
        }
    }
    
    pub fn player_details(&self, ip: std::net::Ipv4Addr, port: u16, callbacks: PlayerDetailsCallbacks) {
        unsafe {
            let callbacks = create_playerdetails(callbacks);
            
            steamworks_sys::SteamAPI_ISteamMatchmakingServers_PlayerDetails(
                self.mms,
                ip.into(),
                port,
                callbacks.cast()
            );
        }
    }
    
    pub fn server_rules(&self, ip: std::net::Ipv4Addr, port: u16, callbacks: ServerRulesCallbacks) {
        unsafe {
            let callbacks = create_serverrules(callbacks);
    
            steamworks_sys::SteamAPI_ISteamMatchmakingServers_ServerRules(
                self.mms,
                ip.into(),
                port,
                callbacks.cast()
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
        callbacks: ServerListCallbacks
    ) -> Arc<Mutex<ServerListRequest>> {
        unsafe {
            let app_id = app_id.into().0;

            let callbacks = create_serverlist(callbacks);
            
            let arc = Arc::clone(&(*(*callbacks).rust_callbacks).req);
            let mut req = arc.lock().unwrap();

            let handle = steamworks_sys::SteamAPI_ISteamMatchmakingServers_RequestLANServerList(
                self.mms,
                app_id,
                callbacks.cast()
            );

            req.mms = self.mms;
            req.real = callbacks;
            // No filters here, leaving defaults. Then check its in ServerListRequest::release
            // req.filters = filters;
            req.h_req = handle;

            drop(req);

            arc
        }
    }

    gen_server_list_fn!(internet_server_list, SteamAPI_ISteamMatchmakingServers_RequestInternetServerList);
    gen_server_list_fn!(favorites_server_list, SteamAPI_ISteamMatchmakingServers_RequestFavoritesServerList);
    gen_server_list_fn!(history_server_list, SteamAPI_ISteamMatchmakingServers_RequestHistoryServerList);
    gen_server_list_fn!(friends_server_list, SteamAPI_ISteamMatchmakingServers_RequestFriendsServerList);
}

#[test]
fn test_internet_servers() {
    let (client, single) = Client::init_app(304930).unwrap();

    let data = std::rc::Rc::new(Mutex::new(0));
    let data2 = std::rc::Rc::clone(&data);
    let data3 = std::rc::Rc::clone(&data);
    let callbacks = ServerListCallbacks::new(
        Box::new(move |list, server| {
            let details = list.lock().unwrap().get_server_details(server).unwrap();
            println!("{} : {}", details.server_name, details.map);
            *data.lock().unwrap() += 1;
        }),
        Box::new(move |_list, _server| {
            *data2.lock().unwrap() += 1;
        }),
        Box::new(move |list, _response| {
            list.lock().unwrap().release();
            println!("{}", data3.lock().unwrap());
        })
    );

    let mut map = HashMap::new();
    map.insert("map", "PEI");
    let _ = client.matchmaking_servers().internet_server_list(304930, &map, callbacks);
    
    for _ in 0..2000 {
        single.run_callbacks();
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
