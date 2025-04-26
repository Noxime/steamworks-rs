use std::ptr::NonNull;
use sys::InputHandle_t;

use super::*;

/// Access to the steam input interface
pub struct Input {
    pub(crate) input: NonNull<sys::ISteamInput>,
    pub(crate) _inner: Arc<Inner>,
}

pub enum InputType {
    Unknown,
    SteamController,
    XBox360Controller,
    XBoxOneController,
    GenericGamepad,
    PS4Controller,
    AppleMFiController,
    AndroidController,
    SwitchJoyConPair,
    SwitchJoyConSingle,
    SwitchProController,
    MobileTouch,
    PS3Controller,
    PS5Controller,
    SteamDeckController,
}

impl Input {
    /// Init must be called when starting use of this interface.
    /// if explicitly_call_run_frame is called then you will need to manually call RunFrame
    /// each frame, otherwise Steam Input will updated when SteamAPI_RunCallbacks() is called
    pub fn init(&self, explicitly_call_run_frame: bool) -> bool {
        unsafe { sys::SteamAPI_ISteamInput_Init(self.input.as_ptr(), explicitly_call_run_frame) }
    }

    /// Synchronize API state with the latest Steam Input action data available. This
    /// is performed automatically by SteamAPI_RunCallbacks, but for the absolute lowest
    /// possible latency, you call this directly before reading controller state.
    /// Note: This must be called from somewhere before GetConnectedControllers will
    /// return any handles
    pub fn run_frame(&self) {
        unsafe { sys::SteamAPI_ISteamInput_RunFrame(self.input.as_ptr(), false) }
    }

    /// Returns a list of the currently connected controllers
    pub fn get_connected_controllers(&self) -> Vec<sys::InputHandle_t> {
        let mut handles = vec![0_u64; sys::STEAM_INPUT_MAX_COUNT as usize];
        let quantity = self.get_connected_controllers_slice(&mut handles);
        handles.shrink_to(quantity);
        handles
    }

    /// Returns a list of the currently connected controllers without allocating, and the count
    pub fn get_connected_controllers_slice(
        &self,
        mut controllers: impl AsMut<[InputHandle_t]>,
    ) -> usize {
        let handles = controllers.as_mut();
        assert!(handles.len() >= sys::STEAM_INPUT_MAX_COUNT as usize);
        unsafe {
            return sys::SteamAPI_ISteamInput_GetConnectedControllers(
                self.input.as_ptr(),
                handles.as_mut_ptr(),
            ) as usize;
        }
    }

    /// Allows to load a specific Action Manifest File localy
    pub fn set_input_action_manifest_file_path(&self, path: &str) -> bool {
        let path = CString::new(path).unwrap();
        unsafe {
            sys::SteamAPI_ISteamInput_SetInputActionManifestFilePath(
                self.input.as_ptr(),
                path.as_ptr(),
            )
        }
    }

    /// Returns the associated ControllerActionSet handle for the specified controller,
    pub fn get_action_set_handle(&self, action_set_name: &str) -> sys::InputActionSetHandle_t {
        let name = CString::new(action_set_name).unwrap();
        unsafe { sys::SteamAPI_ISteamInput_GetActionSetHandle(self.input.as_ptr(), name.as_ptr()) }
    }

    /// Returns the input type for a controler
    pub fn get_input_type_for_handle(&self, input_handle: sys::InputHandle_t) -> InputType {
        let input_type: sys::ESteamInputType = unsafe {
            sys::SteamAPI_ISteamInput_GetInputTypeForHandle(self.input.as_ptr(), input_handle)
        };

        match input_type {
            sys::ESteamInputType::k_ESteamInputType_SteamController => InputType::SteamController,
            sys::ESteamInputType::k_ESteamInputType_GenericGamepad => InputType::GenericGamepad,
            sys::ESteamInputType::k_ESteamInputType_PS4Controller => InputType::PS4Controller,
            sys::ESteamInputType::k_ESteamInputType_SwitchJoyConPair => InputType::SwitchJoyConPair,
            sys::ESteamInputType::k_ESteamInputType_MobileTouch => InputType::MobileTouch,
            sys::ESteamInputType::k_ESteamInputType_PS3Controller => InputType::PS3Controller,
            sys::ESteamInputType::k_ESteamInputType_PS5Controller => InputType::PS5Controller,
            sys::ESteamInputType::k_ESteamInputType_XBox360Controller => {
                InputType::XBox360Controller
            }
            sys::ESteamInputType::k_ESteamInputType_XBoxOneController => {
                InputType::XBoxOneController
            }
            sys::ESteamInputType::k_ESteamInputType_AppleMFiController => {
                InputType::AppleMFiController
            }
            sys::ESteamInputType::k_ESteamInputType_AndroidController => {
                InputType::AndroidController
            }
            sys::ESteamInputType::k_ESteamInputType_SwitchJoyConSingle => {
                InputType::SwitchJoyConSingle
            }
            sys::ESteamInputType::k_ESteamInputType_SwitchProController => {
                InputType::SwitchProController
            }
            sys::ESteamInputType::k_ESteamInputType_SteamDeckController => {
                InputType::SteamDeckController
            }
            _ => InputType::Unknown,
        }
    }

    /// Returns the glyph for an input action
    pub fn get_glyph_for_action_origin(&self, action_origin: sys::EInputActionOrigin) -> String {
        unsafe {
            let glyph_path = sys::SteamAPI_ISteamInput_GetGlyphForActionOrigin_Legacy(
                self.input.as_ptr(),
                action_origin,
            );
            lossy_string_from_cstr(glyph_path)
        }
    }

    /// Returns the name of an input action
    pub fn get_string_for_action_origin(&self, action_origin: sys::EInputActionOrigin) -> String {
        unsafe {
            let name_path = sys::SteamAPI_ISteamInput_GetStringForActionOrigin(
                self.input.as_ptr(),
                action_origin,
            );
            lossy_string_from_cstr(name_path)
        }
    }

    /// Reconfigure the controller to use the specified action set
    /// This is cheap, and can be safely called repeatedly.
    pub fn activate_action_set_handle(
        &self,
        input_handle: sys::InputHandle_t,
        action_set_handle: sys::InputActionSetHandle_t,
    ) {
        unsafe {
            sys::SteamAPI_ISteamInput_ActivateActionSet(
                self.input.as_ptr(),
                input_handle,
                action_set_handle,
            )
        }
    }

    /// Get the handle of the specified Digital action.
    pub fn get_digital_action_handle(&self, action_name: &str) -> sys::InputDigitalActionHandle_t {
        let name = CString::new(action_name).unwrap();
        unsafe {
            sys::SteamAPI_ISteamInput_GetDigitalActionHandle(self.input.as_ptr(), name.as_ptr())
        }
    }

    /// Get the handle of the specified Analog action.
    pub fn get_analog_action_handle(&self, action_name: &str) -> sys::InputAnalogActionHandle_t {
        let name = CString::new(action_name).unwrap();
        unsafe {
            sys::SteamAPI_ISteamInput_GetAnalogActionHandle(self.input.as_ptr(), name.as_ptr())
        }
    }

    /// Returns the current state of the supplied digital game action.
    pub fn get_digital_action_data(
        &self,
        input_handle: sys::InputHandle_t,
        action_handle: sys::InputDigitalActionHandle_t,
    ) -> sys::InputDigitalActionData_t {
        unsafe {
            sys::SteamAPI_ISteamInput_GetDigitalActionData(
                self.input.as_ptr(),
                input_handle,
                action_handle,
            )
        }
    }

    /// Returns the current state of the supplied analog game action.
    pub fn get_analog_action_data(
        &self,
        input_handle: sys::InputHandle_t,
        action_handle: sys::InputAnalogActionHandle_t,
    ) -> sys::InputAnalogActionData_t {
        unsafe {
            sys::SteamAPI_ISteamInput_GetAnalogActionData(
                self.input.as_ptr(),
                input_handle,
                action_handle,
            )
        }
    }

    /// Get the origin(s) for a digital action within an action set.
    pub fn get_digital_action_origins(
        &self,
        input_handle: sys::InputHandle_t,
        action_set_handle: sys::InputActionSetHandle_t,
        digital_action_handle: sys::InputDigitalActionHandle_t,
    ) -> Vec<sys::EInputActionOrigin> {
        let mut origins = Vec::with_capacity(sys::STEAM_INPUT_MAX_ORIGINS as usize);
        unsafe {
            let len = sys::SteamAPI_ISteamInput_GetDigitalActionOrigins(
                self.input.as_ptr(),
                input_handle,
                action_set_handle,
                digital_action_handle,
                origins.as_mut_ptr(),
            );
            origins.set_len(len as usize);
            origins
        }
    }

    /// Get the origin(s) for an analog action within an action set.
    pub fn get_analog_action_origins(
        &self,
        input_handle: sys::InputHandle_t,
        action_set_handle: sys::InputActionSetHandle_t,
        analog_action_handle: sys::InputAnalogActionHandle_t,
    ) -> Vec<sys::EInputActionOrigin> {
        let mut origins = Vec::with_capacity(sys::STEAM_INPUT_MAX_ORIGINS as usize);
        unsafe {
            let len = sys::SteamAPI_ISteamInput_GetAnalogActionOrigins(
                self.input.as_ptr(),
                input_handle,
                action_set_handle,
                analog_action_handle,
                origins.as_mut_ptr(),
            );
            origins.set_len(len as usize);
            origins
        }
    }

    pub fn get_motion_data(&self, input_handle: sys::InputHandle_t) -> sys::InputMotionData_t {
        unsafe { sys::SteamAPI_ISteamInput_GetMotionData(self.input.as_ptr(), input_handle) }
    }

    /// Invokes the Steam overlay and brings up the binding screen.
    /// Returns true for success, false if overlay is disabled/unavailable.
    /// If the player is using Big Picture Mode the configuration will open in
    /// the overlay. In desktop mode a popup window version of Big Picture will
    /// be created and open the configuration.
    pub fn show_binding_panel(&self, input_handle: sys::InputHandle_t) -> bool {
        unsafe { sys::SteamAPI_ISteamInput_ShowBindingPanel(self.input.as_ptr(), input_handle) }
    }

    /// Shutdown must be called when ending use of this interface.
    pub fn shutdown(&self) {
        unsafe {
            sys::SteamAPI_ISteamInput_Shutdown(self.input.as_ptr());
        }
    }
}
