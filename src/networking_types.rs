use crate::SteamId;
use std::ffi::CStr;
use std::net::IpAddr;

bitflags! {
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[repr(C)]
    pub struct SendFlags: i32 {
        const UNRELIABLE = sys::k_nSteamNetworkingSend_Unreliable;
        const NO_NAGLE = sys::k_nSteamNetworkingSend_NoNagle;
        const UNRELIABLE_NO_NAGLE = sys::k_nSteamNetworkingSend_UnreliableNoNagle;
        const NO_DELAY = sys::k_nSteamNetworkingSend_NoDelay;
        const UNRELIABLE_NO_DELAY = sys::k_nSteamNetworkingSend_UnreliableNoDelay;
        const RELIABLE = sys::k_nSteamNetworkingSend_Reliable;
        const RELIABLE_NO_NAGLE = sys::k_nSteamNetworkingSend_ReliableNoNagle;
        const USE_CURRENT_THREAD = sys::k_nSteamNetworkingSend_UseCurrentThread;
        const AUTO_RESTART_BROKEN_SESSION = sys::k_nSteamNetworkingSend_AutoRestartBrokenSession;
    }
}

pub enum NetworkingIdentity<'a> {
    IpAddress(IpAddr, u16),
    Generic(&'a CStr),
    SteamId(SteamId),
}

impl From<SteamId> for NetworkingIdentity<'_> {
    fn from(id: SteamId) -> Self {
        NetworkingIdentity::SteamId(id)
    }
}

pub struct NetworkingMessage {
    inner: *mut sys::SteamNetworkingMessage_t,
}

impl NetworkingMessage {
    pub fn sender_id(&self) -> NetworkingIdentity {
        use sys::ESteamNetworkingIdentityType::*;

        unsafe {
            let ident = &mut (*self.inner).m_identityPeer;

            match ident.m_eType {
                k_ESteamNetworkingIdentityType_SteamID => {
                    NetworkingIdentity::SteamId(SteamId::from_raw(ident.GetSteamID64()))
                }
                k_ESteamNetworkingIdentityType_IPAddress => {
                    let addr = &(*sys::SteamAPI_SteamNetworkingIdentity_GetIPAddr(ident as *mut _));
                    let ip_bytes = addr.__bindgen_anon_1.m_ipv6;

                    NetworkingIdentity::IpAddress(IpAddr::from(ip_bytes), (*addr).m_port)
                }
                k_ESteamNetworkingIdentityType_GenericString => {
                    NetworkingIdentity::Generic(CStr::from_ptr(ident.GetGenericString()))
                }
                _ => unimplemented!("TODO: must be a steamworks bug"),
            }
        }
    }

    pub fn data(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts((*self.inner).m_pData as _, (*self.inner).m_cbSize as usize)
        }
    }
}

impl Drop for NetworkingMessage {
    fn drop(&mut self) {
        debug_assert!(!self.inner.is_null());

        unsafe { sys::SteamAPI_SteamNetworkingMessage_t_Release(self.inner) }
    }
}
