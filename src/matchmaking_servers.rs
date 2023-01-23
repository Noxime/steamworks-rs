// #![allow(dead_code)]

use super::*;

macro_rules! matchmaking_server_callback {
    (
        $name:ident;
        $(
            $fn_name:ident($clear_after_call:expr): ( $( $fn_arg_name:ident: $cpp_fn_arg:ty => $rust_fn_arg:ty where $normalize_fn:tt),* )
        ),*
    ) => {
        paste::item! {
            $(
                extern fn [<$name _ $fn_name _virtual>](_self: *mut [<$name _callbacks_real>] $(, $fn_arg_name: $cpp_fn_arg)*) {
                    unsafe {
                        ((*(*_self).rust_callbacks).$fn_name)($($normalize_fn ($fn_arg_name)),*);
                        if ($clear_after_call == true) {
                            [<free_ $name>](_self);
                        }
                    }
                }
            )*
            
            pub struct [<$name _rust_callbacks>] {
                $(
                    pub $fn_name: Box<dyn Fn($($rust_fn_arg),*)>
                ),*
            }
            
            #[repr(C)]
            struct [<$name _callbacks_real>] {
                pub vtable: *mut [<$name _callbacks_virtual>],
                pub rust_callbacks: *mut [<$name _rust_callbacks>],
            }
            
            #[repr(C)]
            struct [<$name _callbacks_virtual>] {
                $(
                    pub $fn_name: extern fn(*mut [<$name _callbacks_real>] $(, $cpp_fn_arg)*)
                ),*
            }
            
            unsafe fn [<create_ $name>](rust_callbacks: [<$name _rust_callbacks>]) -> *mut [<$name _callbacks_real>] {
                let vtable_layout = std::alloc::Layout::new::<[<$name _callbacks_virtual>]>();
                let callbacks_layout = std::alloc::Layout::new::<[<$name _callbacks_real>]>();
                let rustcallbacks_layout = std::alloc::Layout::new::<[<$name _rust_callbacks>]>();
                let __callbacks: *mut [<$name _rust_callbacks>] = std::alloc::alloc(rustcallbacks_layout).cast();
                if __callbacks.is_null() {
                    std::alloc::handle_alloc_error(rustcallbacks_layout);
                }
                __callbacks.write(rust_callbacks);
                let vtable: *mut [<$name _callbacks_virtual>] = std::alloc::alloc(vtable_layout).cast();
                if vtable.is_null() {
                    std::alloc::handle_alloc_error(vtable_layout);
                }
                {
                    let strct = [<$name _callbacks_virtual>] {
                        $(
                            $fn_name: [<$name _ $fn_name _virtual>]
                        ),*
                    };
                    vtable.write(strct);
                }
                let callbacks: *mut [<$name _callbacks_real>] = std::alloc::alloc(callbacks_layout).cast();
                if callbacks.is_null() {
                    std::alloc::handle_alloc_error(callbacks_layout);
                }
                {
                    let strct = [<$name _callbacks_real>] {
                        vtable: vtable,
                        rust_callbacks: __callbacks,
                    };
                    callbacks.write(strct);
                }
                
                callbacks
            }
            
            unsafe fn [<free_ $name>](real: *mut [<$name _callbacks_real>]) {
                let vtable_layout = std::alloc::Layout::new::<[<$name _callbacks_virtual>]>();
                let callbacks_layout = std::alloc::Layout::new::<[<$name _callbacks_real>]>();
                let rustcallbacks_layout = std::alloc::Layout::new::<[<$name _rust_callbacks>]>();
                
                std::alloc::dealloc((*real).rust_callbacks.cast(), rustcallbacks_layout);
                std::alloc::dealloc((*real).vtable.cast(), vtable_layout);
                std::alloc::dealloc(real.cast(), callbacks_layout);
            }
        }
    };
}

#[inline]
unsafe fn ptr_to_string(ptr: *const std::os::raw::c_char) -> Option<String> {
    Some(CStr::from_ptr(ptr).to_str().ok()?.to_string())
}

pub struct normalized_gameserver_t {
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

impl normalized_gameserver_t {
    pub(crate) fn from_raw_ptr(raw: *mut steamworks_sys::gameserveritem_t) -> Self {
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
            
                game_description: ptr_to_string(raw.m_szGameDescription.as_ptr()),
                server_name: ptr_to_string(raw.m_szServerName.as_ptr()),
                game_dir: ptr_to_string(raw.m_szGameDir.as_ptr()),
                map: ptr_to_string(raw.m_szMap.as_ptr()),
            
                last_time_played: // Это так работает?
                std::time::Duration::from_secs(raw.m_ulTimeLastPlayed.into())
            }
        }
    }
}

matchmaking_server_callback!(
    ping;
    responded(true): (info: *mut steamworks_sys::gameserveritem_t => normalized_gameserver_t where (normalized_gameserver_t::from_raw_ptr)),
    failed(true): ()
);

matchmaking_server_callback!(
    player_details;
    add_player(false): (
        name: *const std::os::raw::c_char => &str where (|name| CStr::from_ptr(name).to_str().unwrap()),
        score: i32 => i32 where (|i| i),
        time_played: f32 => std::time::Duration where (|raw| std::time::Duration::from_secs_f32(raw))
    ),
    failed(true): (),
    refresh_complete(true): ()
);

/// Access to the steam matchmaking_servers interface
pub struct MatchmakingServers<Manager> {
    pub(crate) mm: *mut sys::ISteamMatchmakingServers,
    pub(crate) inner: Arc<Inner<Manager>>,
}

#[cfg(feature = "matchmaking_servers_callbacks")]
impl<Manager> MatchmakingServers<Manager> {
    pub fn ping_server(&self, ip: std::net::Ipv4Addr, port: u16, callbacks: ping_rust_callbacks) {
        unsafe {
            let mut callbacks = create_ping(callbacks);
        
            let query = steamworks_sys::SteamAPI_ISteamMatchmakingServers_PingServer(
                self.mm,
                ip.into(),
                port,
                callbacks.cast(),
            );
            if query == 0 {
                free_ping(callbacks);
            }
        }
    }
    
    pub fn player_details(&self, ip: std::net::Ipv4Addr, port: u16, callbacks: player_details_rust_callbacks) {
        unsafe {
            let mut callbacks = create_player_details(callbacks);
            
            let query = steamworks_sys::SteamAPI_ISteamMatchmakingServers_PlayerDetails(
                self.mm,
                ip.into(),
                port,
                callbacks.cast()
            );
            if query == 0 {
                free_player_details(callbacks);
            }
        }
    }
}

#[cfg(feature = "matchmaking_servers_blocking")]
impl<Manager> MatchmakingServers<Manager> {

}