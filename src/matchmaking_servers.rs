use super::*;
#[cfg(test)]
use serial_test::serial;

unsafe impl Sync for ServerListRequest {}
unsafe impl Send for ServerListRequest {}
impl ServerListRequest {
    unsafe fn clone(&self) -> Self {
        Self {
            callbacks: self.callbacks.clone(),
            handle: self.handle.clone(),
            mms: self.mms.clone(),
        }
    }
}

struct RawVec(*mut steamworks_sys::MatchMakingKeyValuePair_t, usize);
unsafe impl Sync for RawVec {}
unsafe impl Send for RawVec {}

lazy_static! {
    static ref SERVER_LIST_REQUESTS: Mutex<Vec<(ServerListRequest, u8, RawVec)>> = Mutex::new(Vec::new());
}

unsafe fn add_servers_request(request: &ServerListRequest) -> Option<()> {
    let mut list = SERVER_LIST_REQUESTS.lock().ok()?;
    let index = list.binary_search_by(
        |i| i.0.handle.cmp(&request.handle)
    ).ok()?;
    list[index].1 += 1;
    
    Some(())
}

unsafe fn remove_servers_request(request: &ServerListRequest) -> Option<()> {
    let mut list = SERVER_LIST_REQUESTS.lock().ok()?;
    let index = list.binary_search_by(
        |i| i.0.handle.cmp(&request.handle)
    ).ok()?;
    list[index].1 -= 1;
    
    if list[index].1 <= 0 {
        steamworks_sys::SteamAPI_ISteamMatchmakingServers_CancelQuery(
            steamworks_sys::SteamAPI_SteamMatchmakingServers_v002(),
            list[index].0.handle,
        );
        steamworks_sys::SteamAPI_ISteamMatchmakingServers_ReleaseRequest(
            steamworks_sys::SteamAPI_SteamMatchmakingServers_v002(),
            list[index].0.handle,
        );
        
        drop_filters(list[index].2.0, list[index].2.1);
        // TODO: Возможно это здесь не нужно, ибо вызывается ReleaseRequest
        free_serverlist(list[index].0.callbacks);
    }
    
    Some(())
}

// TODO: Привести макрос в нормальный вид
macro_rules! matchmaking_servers_callback {
    (
    
    ) => {
    
    };
    (
        $name:ident;
        $(
            $fn_name:ident($clear_after_call:tt): ( $( $fn_arg_name:ident: $cpp_fn_arg:ty => $rust_fn_arg:ty where $normalize_fn:tt),* )
        ),*
    ) => {
        paste::item! {
            $(
                extern fn [<$name:lower _ $fn_name _virtual>](_self: *mut [<$name _callbacks_real>] $(, $fn_arg_name: $cpp_fn_arg)*) {
                    unsafe {
                        ((*(*_self).rust_callbacks).$fn_name)($($normalize_fn ($fn_arg_name)),*);
                        $clear_after_call (
                            _self,
                            $(
                                $fn_arg_name
                            ),*
                        );
                    }
                }
            )*
            
            pub struct [<$name Callbacks>] {
                $(
                    pub $fn_name: Box<dyn Fn($($rust_fn_arg),*)>
                ),*
            }
            
            #[repr(C)]
            struct [<$name _callbacks_real>] {
                pub vtable: *mut [<$name _callbacks_virtual>],
                pub rust_callbacks: *mut [<$name Callbacks>],
            }
            
            #[repr(C)]
            struct [<$name _callbacks_virtual>] {
                $(
                    pub $fn_name: extern fn(*mut [<$name _callbacks_real>] $(, $cpp_fn_arg)*)
                ),*
            }
            
            unsafe fn [<create_ $name:lower>](rust_callbacks: [<$name Callbacks>]) -> Option<*mut [<$name _callbacks_real>]> {
                let vtable_layout = std::alloc::Layout::new::<[<$name _callbacks_virtual>]>();
                let callbacks_layout = std::alloc::Layout::new::<[<$name _callbacks_real>]>();
                let rustcallbacks_layout = std::alloc::Layout::new::<[<$name Callbacks>]>();
                let __callbacks: *mut [<$name Callbacks>] = std::alloc::alloc(rustcallbacks_layout).cast();
                if __callbacks.is_null() {
                    return None;
                }
                __callbacks.write(rust_callbacks);
                let vtable: *mut [<$name _callbacks_virtual>] = std::alloc::alloc(vtable_layout).cast();
                if vtable.is_null() {
                    return None;
                }
                {
                    let strct = [<$name _callbacks_virtual>] {
                        $(
                            $fn_name: [<$name:lower _ $fn_name _virtual>]
                        ),*
                    };
                    vtable.write(strct);
                }
                let callbacks: *mut [<$name _callbacks_real>] = std::alloc::alloc(callbacks_layout).cast();
                if callbacks.is_null() {
                    return None;
                }
                {
                    let strct = [<$name _callbacks_real>] {
                        vtable: vtable,
                        rust_callbacks: __callbacks,
                    };
                    callbacks.write(strct);
                }
                
                Some(callbacks)
            }
            
            unsafe fn [<free_ $name:lower>](real: *mut [<$name _callbacks_real>]) {
                let vtable_layout = std::alloc::Layout::new::<[<$name _callbacks_virtual>]>();
                let callbacks_layout = std::alloc::Layout::new::<[<$name _callbacks_real>]>();
                let rustcallbacks_layout = std::alloc::Layout::new::<[<$name Callbacks>]>();
                
                std::alloc::dealloc((*real).rust_callbacks.cast(), rustcallbacks_layout);
                std::alloc::dealloc((*real).vtable.cast(), vtable_layout);
                std::alloc::dealloc(real.cast(), callbacks_layout);
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
    pub addr: std::net::Ipv4Addr,
    pub query_port: u16,
    pub connection_port: u16,
    pub game_description: Option<String>,
    pub server_name: Option<String>,
    pub game_dir: Option<String>,
    pub map: Option<String>,
}

impl GameServerItem {
    unsafe fn ptr_to_string(ptr: *const std::os::raw::c_char) -> Option<String> {
        Some(CStr::from_ptr(ptr).to_str().ok()?.to_string())
    }
    
    fn from_raw_ptr(raw: *mut steamworks_sys::gameserveritem_t) -> Self {
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
            
                addr: raw.m_NetAdr.m_unIP.into(),
                query_port: raw.m_NetAdr.m_usQueryPort,
                connection_port: raw.m_NetAdr.m_usConnectionPort,
            
                game_description: Self::ptr_to_string(raw.m_szGameDescription.as_ptr()),
                server_name: Self::ptr_to_string(raw.m_szServerName.as_ptr()),
                game_dir: Self::ptr_to_string(raw.m_szGameDir.as_ptr()),
                map: Self::ptr_to_string(raw.m_szMap.as_ptr()),
            
                last_time_played: // Это так работает?
                    std::time::Duration::from_secs(raw.m_ulTimeLastPlayed.into())
            }
        }
    }
}

matchmaking_servers_callback!(
    Ping;
    responded(||): (info: *mut steamworks_sys::gameserveritem_t => GameServerItem where (GameServerItem::from_raw_ptr)),
    failed(free_ping): ()
);

matchmaking_servers_callback!(
    PlayerDetails;
    add_player(||): (
        name: *const std::os::raw::c_char => Result<&str, std::str::Utf8Error> where (|name| CStr::from_ptr(name).to_str()),
        score: i32 => i32 where (|i| i),
        time_played: f32 => std::time::Duration where (|raw| std::time::Duration::from_secs_f32(raw))
    ),
    failed(free_playerdetails): (),
    refresh_complete(free_playerdetails): ()
);

matchmaking_servers_callback!(
    ServerRules;
    add_rule(||): (
        rule: *const std::os::raw::c_char => Result<&str, std::str::Utf8Error> where (|name| CStr::from_ptr(name).to_str()),
        value: *const std::os::raw::c_char => Result<&str, std::str::Utf8Error> where (|name| CStr::from_ptr(name).to_str())
    ),
    failed(free_serverrules): (),
    refresh_complete(free_serverrules): ()
);

// TODO: Переработать эту хуйню
matchmaking_servers_callback!(
    ServerList;
    responded(||): (
        request: steamworks_sys::HServerListRequest => Option<ServerListRequest> where (ServerListRequest::get),
        server: i32 => i32 where (|i| i)
    ),
    failed(||): (
        request: steamworks_sys::HServerListRequest => Option<ServerListRequest> where (ServerListRequest::get),
        server: i32 => i32 where (|i| i)
    ),
    refresh_complete((|real, request, _| {
        let list = SERVER_LIST_REQUESTS.lock().unwrap(); // TODO: Убрать unwrap
        let index = list.binary_search_by(
            |i| i.0.callbacks.cmp(&real)
        ).unwrap();
        remove_servers_request(&list[index].0).unwrap();
    })): (
        request: steamworks_sys::HServerListRequest => Option<ServerListRequest> where (ServerListRequest::get),
        response: steamworks_sys::EMatchMakingServerResponse => steamworks_sys::EMatchMakingServerResponse where (|i| i)
    )
);

unsafe fn create_filters(
    filters: HashMap<String, String>
) -> Option<(*mut steamworks_sys::MatchMakingKeyValuePair_t, usize)> {
    let mut list = Vec::with_capacity(filters.len());
    
    for filter in filters {
        let filter = (filter.0.as_bytes(), filter.1.as_bytes());
        
        if filter.0.len() <= 255 || filter.1.len() <= 255 {
            // TODO: return enum with error name
            return None;
        }
        
        let key = &*(filter.0 as *const [u8] as *const [i8]);
        let value = &*(filter.1 as *const [u8] as *const [i8]);
        
        list.push(steamworks_sys::MatchMakingKeyValuePair_t {
            m_szKey: key.try_into().ok()?,
            m_szValue: value.try_into().ok()?,
        });
    }
    
    list.shrink_to_fit();
    
    let len = list.len();
    let ptr = list.as_mut_ptr();
    
    std::mem::forget(list);
    
    Some((ptr, len))
}

unsafe fn drop_filters(
    vec: *mut steamworks_sys::MatchMakingKeyValuePair_t,
    size: usize,
) {
    drop(Vec::from_raw_parts(vec, size, size))
}

pub struct ServerListRequest {
    handle: steamworks_sys::HServerListRequest,
    callbacks: *mut ServerList_callbacks_real,
    mms: *mut steamworks_sys::ISteamMatchmakingServers,
}

impl ServerListRequest {
    unsafe fn init(
        handle: steamworks_sys::HServerListRequest,
        callbacks: *mut ServerList_callbacks_real,
        mms: *mut steamworks_sys::ISteamMatchmakingServers,
        vec: (*mut steamworks_sys::MatchMakingKeyValuePair_t, usize)
    ) -> Option<Self> {
        let mut list = SERVER_LIST_REQUESTS.lock().ok()?;
        let request = Self {
            handle,
            callbacks,
            mms,
        };
        list.binary_search_by(
            |i| i.0.handle.cmp(&handle)
        ).err()?;
        
        list.push(
            (request.clone(), 1, RawVec(vec.0, vec.1))
        );
        
        Some(request)
    }
    
    unsafe fn get(
        handle: steamworks_sys::HServerListRequest,
    ) -> Option<Self> {
        let list = SERVER_LIST_REQUESTS.lock().ok()?;
        let index = list.binary_search_by(
            |i| i.0.handle.cmp(&handle)
        ).ok()?;
        add_servers_request(&list[index].0)?;
        Some(list[index].0.clone())
    }
    
    pub fn get_server_count(&self) -> i32 {
        unsafe {
            steamworks_sys::SteamAPI_ISteamMatchmakingServers_GetServerCount(
                self.mms,
                self.handle
            )
        }
    }
    
    pub fn get_server_details(&self, server: i32) -> GameServerItem {
        unsafe {
            let server_item = steamworks_sys::SteamAPI_ISteamMatchmakingServers_GetServerDetails(
                self.mms,
                self.handle,
                server
            );
            
            GameServerItem::from_raw_ptr(server_item)
        }
    }
    
    pub fn refresh_query(&self) -> Option<()> {
        unsafe {
            steamworks_sys::SteamAPI_ISteamMatchmakingServers_RefreshQuery(
                self.mms,
                self.handle,
            );
    
            add_servers_request(self)?;
    
            Some(())
        }
    }
    
    pub fn refresh_server(&self, server: i32) -> Option<()> {
        unsafe {
            steamworks_sys::SteamAPI_ISteamMatchmakingServers_RefreshServer(
                self.mms,
                self.handle,
                server,
            );
            
            add_servers_request(self)?;
            
            Some(())
        }
    }
}

impl Drop for ServerListRequest {
    fn drop(&mut self) {
        unsafe {
            remove_servers_request(self).unwrap(); // TODO: убрать unwrap
        }
    }
}

/// Access to the steam MatchmakingServers interface
pub struct MatchmakingServers<Manager> {
    pub(crate) mms: *mut sys::ISteamMatchmakingServers,
    pub(crate) inner: Arc<Inner<Manager>>,
}

impl<Manager> MatchmakingServers<Manager> {
    pub fn ping_server(&self, ip: std::net::Ipv4Addr, port: u16, callbacks: PingCallbacks) -> Option<()> {
        unsafe {
            let callbacks = create_ping(callbacks)?;
        
            let query = steamworks_sys::SteamAPI_ISteamMatchmakingServers_PingServer(
                self.mms,
                ip.into(),
                port,
                callbacks.cast(),
            );
            if query == 0 {
                free_ping(callbacks);
            }
            
            Some(())
        }
    }
    
    pub fn player_details(&self, ip: std::net::Ipv4Addr, port: u16, callbacks: PlayerDetailsCallbacks) -> Option<()> {
        unsafe {
            let callbacks = create_playerdetails(callbacks)?;
            
            let query = steamworks_sys::SteamAPI_ISteamMatchmakingServers_PlayerDetails(
                self.mms,
                ip.into(),
                port,
                callbacks.cast()
            );
            if query == 0 {
                free_playerdetails(callbacks);
            }
    
            Some(())
        }
    }
    
    pub fn server_rules(&self, ip: std::net::Ipv4Addr, port: u16, callbacks: ServerRulesCallbacks) -> Option<()> {
        unsafe {
            let callbacks = create_serverrules(callbacks)?;
    
            let query = steamworks_sys::SteamAPI_ISteamMatchmakingServers_ServerRules(
                self.mms,
                ip.into(),
                port,
                callbacks.cast()
            );
            if query == 0 {
                free_serverrules(callbacks);
            }
    
            Some(())
        }
    }
    
    pub fn request_history_servers(
        &self,
        id: impl Into<AppId>,
        filters: HashMap<String, String>,
        callbacks: ServerListCallbacks,
    ) -> Option<ServerListRequest> {
        unsafe {
            let mut filters = create_filters(filters)?;
            let callbacks = create_serverlist(callbacks)?;
            
            let request = steamworks_sys::SteamAPI_ISteamMatchmakingServers_RequestHistoryServerList(
                self.mms,
                id.into().0,
                &mut filters.0 as *mut *mut _,
                filters.1 as u32,
                callbacks.cast()
            );
            
            let request = ServerListRequest::init(
                request, callbacks, self.mms, filters
            )?;
            
            Some(request)
        }
    }
}

// TODO: Tests