use super::*;

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

callback_in_struct!(
    ping;
    responded(true): (info: *mut steamworks_sys::gameserveritem_t => normalized_gameserver_t where (|info: *mut steamworks_sys::gameserveritem_t| {
        unsafe {
            let raw = *info;
            normalized_gameserver_t {
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
    })),
    failed(true): ()
);

/// Access to the steam matchmaking_servers interface
pub struct MatchmakingServers<Manager> {
    pub(crate) mm: *mut sys::ISteamMatchmakingServers,
    pub(crate) inner: Arc<Inner<Manager>>,
}

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
}