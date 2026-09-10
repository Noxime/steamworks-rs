use crate::networking_types::{
    NetworkingAvailabilityResult, NetworkingConfigData, NetworkingConfigDataType,
    NetworkingConfigEntry, NetworkingConfigValue, NetworkingMessage,
};
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
            std::mem::forget(register_callback(
                &self.inner,
                move |status: RelayNetworkStatusCallback| {
                    callback(status.status);
                },
            ));
        }
    }

    /// Set a networking configuration value globally.
    ///
    /// Returns true if the setting was successful.
    pub fn set_config_value(&self, config_entry: NetworkingConfigEntry) -> bool {
        unsafe {
            self.set_config_value_internal(
                0, // handle is ignored for global
                sys::ESteamNetworkingConfigScope::k_ESteamNetworkingConfig_Global,
                config_entry,
            )
        }
    }

    /// Set a configuration value for different scopes. Internal use only.
    ///
    /// Returns true if the parameter was successfully set
    pub(crate) unsafe fn set_config_value_internal(
        &self,
        scope_handle: u32,
        scope: sys::ESteamNetworkingConfigScope,
        config_entry: NetworkingConfigEntry,
    ) -> bool {
        // FIXME: unfortunately I can't seem to make set_config_value work for strings...
        // I can retrieve them with `get_config_value`, and it works for floats and ints, sure,
        // but I can't edit string values specifically.
        unsafe {
            sys::SteamAPI_ISteamNetworkingUtils_SetConfigValue(
                self.utils,
                config_entry.inner.m_eValue,
                scope,
                scope_handle as isize,
                config_entry.inner.m_eDataType,
                &config_entry.inner.m_val as *const _ as *const c_void,
            )
        }
    }

    /// Unsets a config value globally, using the system defaults instead.
    ///
    /// Returns true if the setting was successfully unset.
    pub fn unset_config_value(&self, value: NetworkingConfigValue) -> bool {
        unsafe {
            self.unset_config_value_internal(
                0,
                sys::ESteamNetworkingConfigScope::k_ESteamNetworkingConfig_Global,
                value,
            )
        }
    }

    /// Unset a configuration value for different scopes. Internal use only.
    ///
    /// Returns true if the parameter was successfully unset
    pub(crate) unsafe fn unset_config_value_internal(
        &self,
        scope_handle: u32,
        scope: sys::ESteamNetworkingConfigScope,
        value: NetworkingConfigValue,
    ) -> bool {
        unsafe {
            sys::SteamAPI_ISteamNetworkingUtils_SetConfigValue(
                self.utils,
                value.into(),
                scope,
                scope_handle as isize,
                value.data_type().into(),
                std::ptr::null(),
            )
        }
    }

    /// Get a config value applied for the global instance.
    pub fn get_config_value(
        &self,
        value: NetworkingConfigValue,
    ) -> Result<NetworkingConfigData, ()> {
        unsafe {
            self.get_config_value_internal(
                0,
                sys::ESteamNetworkingConfigScope::k_ESteamNetworkingConfig_Global,
                value,
            )
        }
    }

    pub(crate) unsafe fn get_config_value_internal(
        &self,
        scope_handle: u32,
        scope: sys::ESteamNetworkingConfigScope,
        value: NetworkingConfigValue,
    ) -> Result<NetworkingConfigData, ()> {
        let mut buf_size = match value.data_type() {
            NetworkingConfigDataType::String => {
                // for strings, we need to first retrieve the size of the output string
                let mut buf_size: usize = 0;
                let r = sys::SteamAPI_ISteamNetworkingUtils_GetConfigValue(
                    self.utils,
                    value.into(),
                    scope,
                    scope_handle as isize,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    &mut buf_size as *mut _,
                );
                if r != sys::ESteamNetworkingGetConfigValueResult::k_ESteamNetworkingGetConfigValue_BufferTooSmall {
                    // at this stage, the API *should* return BufferTooSmall
                    return Err(());
                }
                buf_size
            }
            NetworkingConfigDataType::Int32 => std::mem::size_of::<i32>(),
            NetworkingConfigDataType::Int64 => std::mem::size_of::<i64>(),
            NetworkingConfigDataType::Float => std::mem::size_of::<f32>(),
            NetworkingConfigDataType::Callback => std::mem::size_of::<*mut ()>(),
        };
        // once we have buf size, we can transform bytes into our values
        let mut buf = vec![0; buf_size];
        unsafe {
            let r = sys::SteamAPI_ISteamNetworkingUtils_GetConfigValue(
                self.utils,
                value.into(),
                scope,
                scope_handle as isize,
                std::ptr::null_mut(),
                buf.as_mut_ptr() as *mut c_void,
                &mut buf_size as *mut _,
            );
            if (r as i32) < 0 {
                return Err(());
            }
        };
        NetworkingConfigData::from_buf(&*buf, value.data_type()).ok_or(())
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
/// The relay network status callback.
pub struct RelayNetworkStatusCallback {
    status: RelayNetworkStatus,
}

impl_callback!(cb: SteamRelayNetworkStatus_t => RelayNetworkStatusCallback {
    Self {
        status: cb.into(),
    }
});

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
