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
            
            unsafe fn [<create_ $name:lower>](rust_callbacks: Box<[<$name Callbacks>]>) -> Option<*mut [<$name CallbacksReal>]> {
                let vtable_layout = std::alloc::Layout::new::<[<$name CallbacksVirtual>]>();
                let callbacks_layout = std::alloc::Layout::new::<[<$name CallbacksReal>]>();
                let rust_callbacks: *mut [<$name Callbacks>] = Box::into_raw(rust_callbacks);
                let vtable: *mut [<$name CallbacksVirtual>] = std::alloc::alloc(vtable_layout).cast();
                if vtable.is_null() {
                    return None;
                }
                {
                    let strct = [<$name CallbacksVirtual>] {
                        $(
                            $fn_name: [<$name:lower _ $fn_name _virtual>]
                        ),*
                    };
                    vtable.write(strct);
                }
                let callbacks: *mut [<$name CallbacksReal>] = std::alloc::alloc(callbacks_layout).cast();
                if callbacks.is_null() {
                    return None;
                }
                {
                    let strct = [<$name CallbacksReal>] {
                        vtable: vtable,
                        rust_callbacks,
                    };
                    callbacks.write(strct);
                }
                
                Some(callbacks)
            }
            
            unsafe fn [<free_ $name:lower>](real: *mut [<$name CallbacksReal>]) {
                let vtable_layout = std::alloc::Layout::new::<[<$name CallbacksVirtual>]>();
                let callbacks_layout = std::alloc::Layout::new::<[<$name CallbacksReal>]>();
                
                drop(Box::from_raw((*real).rust_callbacks));
                std::alloc::dealloc((*real).vtable.cast(), vtable_layout);
                std::alloc::dealloc(real.cast(), callbacks_layout);
            }
        }
    };
}

macro_rules! gen_server_list_fn {
    (
        $name:ident, $sys_method:ident
    ) => {
        pub fn $name<ID: Into<AppId>>(
            &self,
            app_id: ID,
            filters: &HashMap<&str, &str>,
            callbacks: ServerListCallbacks
        ) -> Result<Arc<Mutex<ServerListRequest>>, ServerListErrors> {
            unsafe {
                let app_id = app_id.into().0;
    
                let mut filters = create_filters(filters).ok_or(ServerListErrors::FiltersError)?;
                let callbacks = create_serverlist(Box::new(callbacks)).ok_or(ServerListErrors::CreationError)?;
                
                let arc = Arc::clone(&(*(*callbacks).rust_callbacks).req);
                let mut req = arc.lock().unwrap();
    
                let handle = steamworks_sys::$sys_method(
                    self.mms,
                    app_id,
                    &mut filters.0,
                    filters.1.try_into().unwrap(),
                    callbacks.cast()
                );
                if handle.is_null() {
                    free_filters(filters.0, filters.1);
                    free_serverlist(callbacks);
                    return Err(ServerListErrors::RequestError);
                }
    
                req.mms = self.mms;
                req.real = callbacks;
                req.filters = filters;
                req.h_req = handle;
                req.initialized = true;
    
                drop(req);
    
                Ok(arc)
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
        // Rc can be used but then we should not give ServerListRequest
        // from *server_list method to prevent using Rc from different
        // threads. Arc doesn't affect performance that much to stop using it
        req: Arc<Mutex<ServerListRequest>> where {
            Arc::new(Mutex::new(ServerListRequest {
                h_req: ptr::null_mut(),
                released: false,
                initialized: false,
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
        response: steamworks_sys::EMatchMakingServerResponse => steamworks_sys::EMatchMakingServerResponse where {response}
    )
);

pub struct ServerListRequest {
    pub (self) h_req: steamworks_sys::HServerListRequest,
    pub (self) released: bool,
    // Todo: probably not needed
    pub (self) initialized: bool,
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
        if !self.initialized || self.released {
            None
        } else {
            Some(())
        }
    }

    pub fn get_server_count(&self) -> Option<i32> {
        unsafe {
            self.released()?;

            Some(steamworks_sys::SteamAPI_ISteamMatchmakingServers_GetServerCount(
                self.mms,
                self.h_req
            ))
        }
    }
    
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
    let len = vec.len();
    let ptr = vec.as_mut_ptr();
    std::mem::forget(vec);

    Some((ptr, len))
}

unsafe fn free_filters(ptr: *mut steamworks_sys::MatchMakingKeyValuePair_t, count: usize) {
    drop(Vec::from_raw_parts(ptr, count, count))
}

#[derive(Debug)]
pub enum MMSErrors {
    CreationError, RequestError
}

#[derive(Debug)]
pub enum ServerListErrors {
    CreationError, RequestError, FiltersError
}

/// Access to the steam MatchmakingServers interface
pub struct MatchmakingServers<Manager> {
    pub(crate) mms: *mut sys::ISteamMatchmakingServers,
    pub(crate) _inner: Arc<Inner<Manager>>,
}

impl<Manager> MatchmakingServers<Manager> {
    pub fn ping_server(&self, ip: std::net::Ipv4Addr, port: u16, callbacks: PingCallbacks) -> Result<(), MMSErrors> {
        unsafe {
            let callbacks = create_ping(Box::new(callbacks)).ok_or(MMSErrors::CreationError)?;
        
            let query = steamworks_sys::SteamAPI_ISteamMatchmakingServers_PingServer(
                self.mms,
                ip.into(),
                port,
                callbacks.cast(),
            );
            if query == 0 {
                free_ping(callbacks);
                return Err(MMSErrors::RequestError);
            }
            
            Ok(())
        }
    }
    
    pub fn player_details(&self, ip: std::net::Ipv4Addr, port: u16, callbacks: PlayerDetailsCallbacks) -> Result<(), MMSErrors> {
        unsafe {
            let callbacks = create_playerdetails(Box::new(callbacks)).ok_or(MMSErrors::CreationError)?;
            
            let query = steamworks_sys::SteamAPI_ISteamMatchmakingServers_PlayerDetails(
                self.mms,
                ip.into(),
                port,
                callbacks.cast()
            );
            if query == 0 {
                free_playerdetails(callbacks);
                return Err(MMSErrors::RequestError);
            }
    
            Ok(())
        }
    }
    
    pub fn server_rules(&self, ip: std::net::Ipv4Addr, port: u16, callbacks: ServerRulesCallbacks) -> Result<(), MMSErrors> {
        unsafe {
            let callbacks = create_serverrules(Box::new(callbacks)).ok_or(MMSErrors::CreationError)?;
    
            let query = steamworks_sys::SteamAPI_ISteamMatchmakingServers_ServerRules(
                self.mms,
                ip.into(),
                port,
                callbacks.cast()
            );
            if query == 0 {
                free_serverrules(callbacks);
                return Err(MMSErrors::RequestError);
            }
    
            Ok(())
        }
    }

    pub fn lan_server_list<ID: Into<AppId>>(
        &self,
        app_id: ID,
        callbacks: ServerListCallbacks
    ) -> Result<Arc<Mutex<ServerListRequest>>, ServerListErrors> {
        unsafe {
            let app_id = app_id.into().0;

            let callbacks = create_serverlist(Box::new(callbacks)).ok_or(ServerListErrors::CreationError)?;
            
            let arc = Arc::clone(&(*(*callbacks).rust_callbacks).req);
            let mut req = arc.lock().unwrap();

            let handle = steamworks_sys::SteamAPI_ISteamMatchmakingServers_RequestLANServerList(
                self.mms,
                app_id,
                callbacks.cast()
            );
            if handle.is_null() {
                free_serverlist(callbacks);
                return Err(ServerListErrors::RequestError);
            }

            req.mms = self.mms;
            req.real = callbacks;
            // No filters here, leaving defaults. Then check its in ServerListRequest::release
            // req.filters = filters;
            req.h_req = handle;
            req.initialized = true;

            drop(req);

            Ok(arc)
        }
     }

    gen_server_list_fn!(internet_server_list, SteamAPI_ISteamMatchmakingServers_RequestInternetServerList);
    gen_server_list_fn!(favorites_server_list, SteamAPI_ISteamMatchmakingServers_RequestFavoritesServerList);
    gen_server_list_fn!(history_server_list, SteamAPI_ISteamMatchmakingServers_RequestHistoryServerList);
    gen_server_list_fn!(friends_server_list, SteamAPI_ISteamMatchmakingServers_RequestFriendsServerList);
}

#[test]
fn test_history() {
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
        Box::new(move |list, server| {
            *data2.lock().unwrap() += 1;
        }),
        Box::new(move |list, response| {
            list.lock().unwrap().release();
            println!("{}", data3.lock().unwrap());
        })
    );

    let mut map = HashMap::new();
    map.insert("map", "PEI");
    let _ = client.matchmaking_servers().internet_server_list(304930, &map, callbacks).unwrap();
    
    for _ in 0..5000 {
        single.run_callbacks();
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
