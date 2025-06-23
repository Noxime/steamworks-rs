use crate::networking_types::{NetworkingAvailabilityResult, NetworkingMessage};
use crate::{register_callback, Callback, Inner};
use std::convert::TryInto;
use std::ffi::{c_void, CStr};
use std::sync::Arc;

use steamworks_sys as sys;

/// Access to the steam networking sockets interface
pub struct NetworkingUtils {
    pub(crate) utils: *mut sys::ISteamNetworkingUtils,
    pub(crate) inner: Arc<Inner>,
}

unsafe impl Send for NetworkingUtils {}
unsafe impl Sync for NetworkingUtils {}

impl NetworkingUtils {
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
    pub fn allocate_message(&self, buffer_size: usize) -> NetworkingMessage {
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
    /// If you want more detailed information use [`detailed_relay_network_status`](#method.detailed_relay_network_status) instead.
    pub fn relay_network_status(&self) -> NetworkingAvailabilityResult {
        unsafe {
            sys::SteamAPI_ISteamNetworkingUtils_GetRelayNetworkStatus(
                self.utils,
                std::ptr::null_mut(),
            )
            .try_into()
        }
    }

    /// Fetch current detailed status of the relay network.
    pub fn detailed_relay_network_status(&self) -> RelayNetworkStatus {
        unsafe {
            let mut status = sys::SteamRelayNetworkStatus_t {
                m_eAvail: sys::ESteamNetworkingAvailability::k_ESteamNetworkingAvailability_Unknown,
                m_bPingMeasurementInProgress: 0,
                m_eAvailNetworkConfig:
                    sys::ESteamNetworkingAvailability::k_ESteamNetworkingAvailability_Unknown,
                m_eAvailAnyRelay:
                    sys::ESteamNetworkingAvailability::k_ESteamNetworkingAvailability_Unknown,
                m_debugMsg: [0; 256],
            };
            sys::SteamAPI_ISteamNetworkingUtils_GetRelayNetworkStatus(self.utils, &mut status);
            status.into()
        }
    }

    /// Register the callback for relay network status updates.
    ///
    /// Calling this more than once replaces the previous callback.
    pub fn relay_network_status_callback(
        &self,
        mut callback: impl FnMut(RelayNetworkStatus) + Send + 'static,
    ) {
        unsafe {
            register_callback(&self.inner, move |status: RelayNetworkStatusCallback| {
                callback(status.status);
            });
        }
    }
}

pub struct RelayNetworkStatus {
    availability: NetworkingAvailabilityResult,
    is_ping_measurement_in_progress: bool,
    network_config: NetworkingAvailabilityResult,
    any_relay: NetworkingAvailabilityResult,

    debugging_message: String,
}

impl RelayNetworkStatus {
    /// Summary status.  When this is "current", initialization has
    /// completed.  Anything else means you are not ready yet, or
    /// there is a significant problem.
    pub fn availability(&self) -> NetworkingAvailabilityResult {
        self.availability.clone()
    }

    /// True if latency measurement is in progress (or pending, awaiting a prerequisite).
    pub fn is_ping_measurement_in_progress(&self) -> bool {
        self.is_ping_measurement_in_progress
    }

    /// Status obtaining the network config.  This is a prerequisite
    /// for relay network access.
    ///
    /// Failure to obtain the network config almost always indicates
    /// a problem with the local internet connection.
    pub fn network_config(&self) -> NetworkingAvailabilityResult {
        self.network_config.clone()
    }

    /// Current ability to communicate with ANY relay.  Note that
    /// the complete failure to communicate with any relays almost
    /// always indicates a problem with the local Internet connection.
    /// (However, just because you can reach a single relay doesn't
    /// mean that the local connection is in perfect health.)
    pub fn any_relay(&self) -> NetworkingAvailabilityResult {
        self.any_relay.clone()
    }

    /// Non-localized English language status.  For diagnostic/debugging
    /// purposes only.
    pub fn debugging_message(&self) -> &str {
        &self.debugging_message
    }
}

impl From<sys::SteamRelayNetworkStatus_t> for RelayNetworkStatus {
    fn from(status: steamworks_sys::SteamRelayNetworkStatus_t) -> Self {
        unsafe {
            Self {
                availability: status.m_eAvail.try_into(),
                is_ping_measurement_in_progress: status.m_bPingMeasurementInProgress != 0,
                network_config: status.m_eAvailNetworkConfig.try_into(),
                any_relay: status.m_eAvailAnyRelay.try_into(),
                debugging_message: CStr::from_ptr(status.m_debugMsg.as_ptr())
                    .to_str()
                    .expect("invalid debug string")
                    .to_owned(),
            }
        }
    }
}

/// The relay network status callback.
pub struct RelayNetworkStatusCallback {
    status: RelayNetworkStatus,
}

unsafe impl Callback for RelayNetworkStatusCallback {
    const ID: i32 = sys::SteamRelayNetworkStatus_t_k_iCallback as _;

    unsafe fn from_raw(raw: *mut c_void) -> Self {
        let status = *(raw as *mut sys::SteamRelayNetworkStatus_t);
        Self {
            status: status.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Client;
    use std::time::Duration;

    use serial_test::serial;

    #[test]
    #[serial]
    fn test_get_networking_status() {
        let client = Client::init().unwrap();
        let callback_client = client.clone();
        std::thread::spawn(move || callback_client.run_callbacks());

        let utils = client.networking_utils();
        let status = utils.detailed_relay_network_status();
        println!(
            "status: {:?}, network_config: {:?}, any_relay: {:?}, message: {}",
            status.availability(),
            status.network_config(),
            status.any_relay(),
            status.debugging_message()
        );

        utils.init_relay_network_access();

        let status = utils.detailed_relay_network_status();
        println!(
            "status: {:?}, network_config: {:?}, any_relay: {:?}, message: {}",
            status.availability(),
            status.network_config(),
            status.any_relay(),
            status.debugging_message()
        );

        std::thread::sleep(Duration::from_millis(500));

        let status = utils.detailed_relay_network_status();
        println!(
            "status: {:?}, network_config: {:?}, any_relay: {:?}, message: {}",
            status.availability(),
            status.network_config(),
            status.any_relay(),
            status.debugging_message()
        );
    }
}
