use super::*;

/// Access to the steam input interface
pub struct Input<Manager> {
    pub(crate) input: *mut sys::ISteamInput,
    pub(crate) _inner: Arc<Inner<Manager>>,
}

impl<Manager> Input<Manager> {
    /// Init must be called when starting use of this interface.
    /// if explicitly_call_run_frame is called then you will need to manually call RunFrame
    /// each frame, otherwise Steam Input will updated when SteamAPI_RunCallbacks() is called
    pub fn init(&self, explicitly_call_run_frame: bool) {
        unsafe {
            sys::SteamAPI_ISteamInput_Init(self.input, explicitly_call_run_frame);
        }
    }

    /// Synchronize API state with the latest Steam Input action data available. This
    /// is performed automatically by SteamAPI_RunCallbacks, but for the absolute lowest
    /// possible latency, you call this directly before reading controller state.
    /// Note: This must be called from somewhere before GetConnectedControllers will
    /// return any handles
    pub fn run_frame(&self) {
        unsafe { sys::SteamAPI_ISteamInput_RunFrame(self.input, false) }
    }

    /// Returns a list of the currently connected controllers
    pub fn get_connected_controllers(&self) -> Vec<sys::ControllerHandle_t> {
        unsafe {
            let handles = [0_u64; sys::STEAM_INPUT_MAX_COUNT as usize].as_mut_ptr();
            let quantity = sys::SteamAPI_ISteamInput_GetConnectedControllers(self.input, handles);
            if quantity == 0 {
                Vec::new()
            } else {
                std::slice::from_raw_parts(handles as *const _, quantity as usize).to_vec()
            }
        }
    }

    /// Returns the associated ControllerActionSet handle for the specified controller,
    pub fn get_action_set_handle(&self, action_set_name: &str) -> sys::InputActionSetHandle_t {
        let name = CString::new(action_set_name).unwrap();
        unsafe { sys::SteamAPI_ISteamInput_GetActionSetHandle(self.input, name.as_ptr()) }
    }

    /// Shutdown must be called when ending use of this interface.
    pub fn shutdown(&self) {
        unsafe {
            sys::SteamAPI_ISteamInput_Shutdown(self.input);
        }
    }
}
