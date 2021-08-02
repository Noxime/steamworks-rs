use crate::networking_types::{NetworkingAvailability, NetworkingAvailabilityError, NetworkingMessage, NetworkingAvailabilityResult};
use crate::{Callback, Inner, register_callback};
use std::convert::TryInto;
use std::ffi::{c_void, CString};
use std::sync::Arc;

/// Access to the steam networking sockets interface
pub struct NetworkingUtils<Manager> {
    pub(crate) utils: *mut sys::ISteamNetworkingUtils,
    pub(crate) inner: Arc<Inner<Manager>>,
}

unsafe impl<T> Send for NetworkingUtils<T> {}
unsafe impl<T> Sync for NetworkingUtils<T> {}

impl<Manager> NetworkingUtils<Manager> {
    /// Allocate and initialize a message object.  Usually the reason
    /// you call this is to pass it to ISteamNetworkingSockets::SendMessages.
    /// The returned object will have all of the relevant fields cleared to zero.
    ///
    /// Optionally you can also request that this system allocate space to
    /// hold the payload itself.  If cbAllocateBuffer is nonzero, the system
    /// will allocate memory to hold a payload of at least cbAllocateBuffer bytes.
    /// m_pData will point to the allocated buffer, m_cbSize will be set to the
    /// size, and m_pfnFreeData will be set to the proper function to free up
    /// the buffer.
    ///
    /// If cbAllocateBuffer=0, then no buffer is allocated.  m_pData will be NULL,
    /// m_cbSize will be zero, and m_pfnFreeData will be NULL.  You will need to
    /// set each of these.
    pub fn allocate_message(&self, buffer_size: usize) -> NetworkingMessage<Manager> {
        unsafe {
            let message =
                sys::SteamAPI_ISteamNetworkingUtils_AllocateMessage(self.utils, buffer_size as _);
            NetworkingMessage {
                message,
                _inner: self.inner.clone(),
            }
        }
    }

    /// If you know that you are going to be using the relay network (for example,
    /// because you anticipate making P2P connections), call this to initialize the
    /// relay network.  If you do not call this, the initialization will
    /// be delayed until the first time you use a feature that requires access
    /// to the relay network, which will delay that first access.
    ///
    /// You can also call this to force a retry if the previous attempt has failed.
    /// Performing any action that requires access to the relay network will also
    /// trigger a retry, and so calling this function is never strictly necessary,
    /// but it can be useful to call it a program launch time, if access to the
    /// relay network is anticipated.
    ///
    /// Use GetRelayNetworkStatus or listen for SteamRelayNetworkStatus_t
    /// callbacks to know when initialization has completed.
    /// Typically initialization completes in a few seconds.
    ///
    /// Note: dedicated servers hosted in known data centers do *not* need
    /// to call this, since they do not make routing decisions.  However, if
    /// the dedicated server will be using P2P functionality, it will act as
    /// a "client" and this should be called.
    pub fn init_relay_network_access(&self) {
        unsafe {
            sys::SteamAPI_ISteamNetworkingUtils_InitRelayNetworkAccess(self.utils);
        }
    }

    /// Fetch current status of the relay network.
    ///
    /// If you want more detailed information use [`detailed_relay_network_status`] instead.
    pub fn relay_network_status(&self) -> NetworkingAvailabilityResult {
        unsafe {
            sys::SteamAPI_ISteamNetworkingUtils_GetRelayNetworkStatus(self.utils, std::ptr::null_mut()).try_into()
        }
    }

    /// Fetch current detailed status of the relay network.
    pub fn detailed_relay_network_status(&self) -> RelayNetworkStatus {
        unsafe {
            let mut status = std::mem::MaybeUninit::uninit();
            sys::SteamAPI_ISteamNetworkingUtils_GetRelayNetworkStatus(self.utils, status.as_mut_ptr());
            status.assume_init().into()
        }
    }

    /// Register the callback for relay network status updates.
    ///
    /// Calling this more than once replaces the previous callback.
    pub fn relay_network_status_callback(&self, mut callback: impl FnMut(RelayNetworkStatus)) {
        unsafe {
            register_callback(&self.inner, |status: RelayNetworkStatusCallback| {
                callback(status.status);
            });
        }
    }
}

pub struct RelayNetworkStatus {
    availability: Result<NetworkingAvailability, NetworkingAvailabilityError>,
    is_ping_measurement_in_progress: bool,
    network_config: Result<NetworkingAvailability, NetworkingAvailabilityError>,
    any_relay: Result<NetworkingAvailability, NetworkingAvailabilityError>,

    debugging_message: String,
}

impl From<sys::SteamRelayNetworkStatus_t> for RelayNetworkStatus {
    fn from(mut status: steamworks_sys::SteamRelayNetworkStatus_t) -> Self {
        unsafe {
            Self {
                availability: status.m_eAvail.try_into(),
                is_ping_measurement_in_progress: status.m_bPingMeasurementInProgress != 0,
                network_config: status.m_eAvailNetworkConfig.try_into(),
                any_relay: status.m_eAvailAnyRelay.try_into(),
                debugging_message: CString::from_raw(status.m_debugMsg.as_mut_ptr())
                    .into_string()
                    .expect("invalid debug string"),
            }
        }
    }
}

/// The relay network status callback.
struct RelayNetworkStatusCallback {
    status: RelayNetworkStatus,
}

unsafe impl Callback for RelayNetworkStatusCallback {
    const ID: i32 = sys::SteamRelayNetworkStatus_t_k_iCallback as _;
    const SIZE: i32 = std::mem::size_of::<sys::SteamRelayNetworkStatus_t>() as _;

    unsafe fn from_raw(raw: *mut c_void) -> Self {
        let status = *(raw as *mut sys::SteamRelayNetworkStatus_t);
        Self {
            status: status.into(),
        }
    }
}
