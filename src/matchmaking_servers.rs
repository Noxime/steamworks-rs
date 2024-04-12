use super::*;

macro_rules! matchmaking_servers_callback {
    (
        $name:ident;
        $(
            $fn_name:ident($clear_after_call:tt): ( $( $fn_arg_name:ident: $cpp_fn_arg:ty => $rust_fn_arg:ty where $normalize_fn:tt),* )
        ),*
    ) => {
        paste::item! {
            $(
                extern fn [<$name:lower _ $fn_name _virtual>](_self: *mut [<$name CallbacksReal>] $(, $fn_arg_name: $cpp_fn_arg)*) {
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
            
            unsafe fn [<create_ $name:lower>](rust_callbacks: [<$name Callbacks>]) -> Option<*mut [<$name CallbacksReal>]> {
                let vtable_layout = std::alloc::Layout::new::<[<$name CallbacksVirtual>]>();
                let callbacks_layout = std::alloc::Layout::new::<[<$name CallbacksReal>]>();
                let rustcallbacks_layout = std::alloc::Layout::new::<[<$name Callbacks>]>();
                let __callbacks: *mut [<$name Callbacks>] = std::alloc::alloc(rustcallbacks_layout).cast();
                if __callbacks.is_null() {
                    return None;
                }
                __callbacks.write(rust_callbacks);
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
                        rust_callbacks: __callbacks,
                    };
                    callbacks.write(strct);
                }
                
                Some(callbacks)
            }
            
            unsafe fn [<free_ $name:lower>](real: *mut [<$name CallbacksReal>]) {
                let vtable_layout = std::alloc::Layout::new::<[<$name CallbacksVirtual>]>();
                let callbacks_layout = std::alloc::Layout::new::<[<$name CallbacksReal>]>();
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
}

#[test]
fn test_history() {
    let (client, single) = Client::init_app(304930).unwrap();
    client.matchmaking_servers().ping_server(
        std::net::Ipv4Addr::new(213, 32, 111, 44),
        27015,
        PingCallbacks {
            responded: Box::new(|info| {
                println!("server name: {}", info.server_name.unwrap())
            }),
            failed: Box::new(|| {

            }),
        }
    ).unwrap();
    
    for _ in 0..50 {
        single.run_callbacks();
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}
