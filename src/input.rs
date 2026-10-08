use std::path::PathBuf;

use sys::InputHandle_t;

use super::*;

// `EInputActionOrigin` cannot hold origins of devices newer than this SDK, so this takes
// and returns the raw value.
extern "C" {
    #[link_name = "SteamAPI_ISteamInput_TranslateActionOrigin"]
    fn translate_raw_action_origin(
        input: *mut sys::ISteamInput,
        destination_input_type: sys::ESteamInputType,
        source_origin: u32,
    ) -> u32;
}

const ACTION_ORIGIN_NONE: u32 = sys::EInputActionOrigin::k_EInputActionOrigin_None as u32;
const ACTION_ORIGIN_COUNT: u32 = sys::EInputActionOrigin::k_EInputActionOrigin_Count as u32;

/// Access to the steam input interface
pub struct Input {
    pub(crate) input: *mut sys::ISteamInput,
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

/// Size of the glyphs returned by [`Input::get_glyph_png_for_action_origin`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GlyphSize {
    /// 32x32 pixels
    Small,
    /// 128x128 pixels
    Medium,
    /// 256x256 pixels
    Large,
}

bitflags! {
    /// Style of the glyphs returned by [`Input::get_glyph_png_for_action_origin`] and
    /// [`Input::get_glyph_svg_for_action_origin`].
    ///
    /// [`GlyphStyle::KNOCKOUT`], [`GlyphStyle::LIGHT`] and [`GlyphStyle::DARK`] are mutually
    /// exclusive; the ABXY flags can be combined with any of them.
    #[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
    pub struct GlyphStyle: u32 {
        /// Origin glyphs have a light fill color, for use on dark backgrounds.
        const LIGHT = sys::ESteamInputGlyphStyle::ESteamInputGlyphStyle_Light as u32;
        /// Origin glyphs have a dark fill color, for use on light backgrounds.
        const DARK = sys::ESteamInputGlyphStyle::ESteamInputGlyphStyle_Dark as u32;
        /// ABXY buttons use a neutral color instead of their brand color.
        const NEUTRAL_COLOR_ABXY = sys::ESteamInputGlyphStyle::ESteamInputGlyphStyle_NeutralColorABXY as u32;
        /// ABXY buttons have a solid fill.
        const SOLID_ABXY = sys::ESteamInputGlyphStyle::ESteamInputGlyphStyle_SolidABXY as u32;
    }
}

impl GlyphStyle {
    /// Face buttons have colored labels and outlines on a knocked out background, the rest
    /// white detail and borders. Steam's default.
    pub const KNOCKOUT: Self = Self::empty();
}

impl Input {
    /// Init must be called when starting use of this interface.
    /// if explicitly_call_run_frame is called then you will need to manually call RunFrame
    /// each frame, otherwise Steam Input will updated when SteamAPI_RunCallbacks() is called
    pub fn init(&self, explicitly_call_run_frame: bool) -> bool {
        unsafe { sys::SteamAPI_ISteamInput_Init(self.input, explicitly_call_run_frame) }
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
    pub fn get_connected_controllers(&self) -> Vec<sys::InputHandle_t> {
        let mut handles = vec![0_u64; sys::STEAM_INPUT_MAX_COUNT as usize];
        let quantity = self.get_connected_controllers_slice(&mut handles);
        handles.truncate(quantity);
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
                self.input,
                handles.as_mut_ptr(),
            ) as usize;
        }
    }

    /// Allows to load a specific Action Manifest File localy
    pub fn set_input_action_manifest_file_path(&self, path: &str) -> bool {
        let path = CString::new(path).unwrap();
        unsafe {
            sys::SteamAPI_ISteamInput_SetInputActionManifestFilePath(self.input, path.as_ptr())
        }
    }

    /// Returns the associated ControllerActionSet handle for the specified controller,
    pub fn get_action_set_handle(&self, action_set_name: &str) -> sys::InputActionSetHandle_t {
        let name = CString::new(action_set_name).unwrap();
        unsafe { sys::SteamAPI_ISteamInput_GetActionSetHandle(self.input, name.as_ptr()) }
    }

    /// Returns the input type for a controler
    pub fn get_input_type_for_handle(&self, input_handle: sys::InputHandle_t) -> InputType {
        let input_type: sys::ESteamInputType =
            unsafe { sys::SteamAPI_ISteamInput_GetInputTypeForHandle(self.input, input_handle) };

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
            let glyph_path =
                sys::SteamAPI_ISteamInput_GetGlyphForActionOrigin_Legacy(self.input, action_origin);
            let glyph_path = CStr::from_ptr(glyph_path);
            glyph_path.to_string_lossy().into_owned()
        }
    }

    /// Returns a local path to a PNG of the glyph for an action origin, or `None` if Steam has
    /// no glyph for it
    pub fn get_glyph_png_for_action_origin(
        &self,
        action_origin: sys::EInputActionOrigin,
        size: GlyphSize,
        style: GlyphStyle,
    ) -> Option<PathBuf> {
        let size = match size {
            GlyphSize::Small => sys::ESteamInputGlyphSize::k_ESteamInputGlyphSize_Small,
            GlyphSize::Medium => sys::ESteamInputGlyphSize::k_ESteamInputGlyphSize_Medium,
            GlyphSize::Large => sys::ESteamInputGlyphSize::k_ESteamInputGlyphSize_Large,
        };
        unsafe {
            glyph_path(sys::SteamAPI_ISteamInput_GetGlyphPNGForActionOrigin(
                self.input,
                action_origin,
                size,
                style.bits(),
            ))
        }
    }

    /// Returns a local path to an SVG of the glyph for an action origin, or `None` if Steam has
    /// no glyph for it
    pub fn get_glyph_svg_for_action_origin(
        &self,
        action_origin: sys::EInputActionOrigin,
        style: GlyphStyle,
    ) -> Option<PathBuf> {
        unsafe {
            glyph_path(sys::SteamAPI_ISteamInput_GetGlyphSVGForActionOrigin(
                self.input,
                action_origin,
                style.bits(),
            ))
        }
    }

    /// Returns the name of an input action
    pub fn get_string_for_action_origin(&self, action_origin: sys::EInputActionOrigin) -> String {
        unsafe {
            let name_path =
                sys::SteamAPI_ISteamInput_GetStringForActionOrigin(self.input, action_origin);
            let name_path = CStr::from_ptr(name_path);
            name_path.to_string_lossy().into_owned()
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
            sys::SteamAPI_ISteamInput_ActivateActionSet(self.input, input_handle, action_set_handle)
        }
    }

    /// Get the handle of the specified Digital action.
    pub fn get_digital_action_handle(&self, action_name: &str) -> sys::InputDigitalActionHandle_t {
        let name = CString::new(action_name).unwrap();
        unsafe { sys::SteamAPI_ISteamInput_GetDigitalActionHandle(self.input, name.as_ptr()) }
    }

    /// Get the handle of the specified Analog action.
    pub fn get_analog_action_handle(&self, action_name: &str) -> sys::InputAnalogActionHandle_t {
        let name = CString::new(action_name).unwrap();
        unsafe { sys::SteamAPI_ISteamInput_GetAnalogActionHandle(self.input, name.as_ptr()) }
    }

    /// Returns the current state of the supplied digital game action.
    pub fn get_digital_action_data(
        &self,
        input_handle: sys::InputHandle_t,
        action_handle: sys::InputDigitalActionHandle_t,
    ) -> sys::InputDigitalActionData_t {
        unsafe {
            sys::SteamAPI_ISteamInput_GetDigitalActionData(self.input, input_handle, action_handle)
        }
    }

    /// Returns the current state of the supplied analog game action.
    pub fn get_analog_action_data(
        &self,
        input_handle: sys::InputHandle_t,
        action_handle: sys::InputAnalogActionHandle_t,
    ) -> sys::InputAnalogActionData_t {
        unsafe {
            sys::SteamAPI_ISteamInput_GetAnalogActionData(self.input, input_handle, action_handle)
        }
    }

    /// Get the origin(s) for a digital action within an action set.
    ///
    /// Origins of devices newer than this SDK are translated to their closest known equivalent.
    pub fn get_digital_action_origins(
        &self,
        input_handle: sys::InputHandle_t,
        action_set_handle: sys::InputActionSetHandle_t,
        digital_action_handle: sys::InputDigitalActionHandle_t,
    ) -> Vec<sys::EInputActionOrigin> {
        let mut origins = [ACTION_ORIGIN_NONE; sys::STEAM_INPUT_MAX_ORIGINS as usize];
        // Steam writes into `u32` storage, so origins unknown to this SDK never become an
        // `EInputActionOrigin`.
        let len = unsafe {
            sys::SteamAPI_ISteamInput_GetDigitalActionOrigins(
                self.input,
                input_handle,
                action_set_handle,
                digital_action_handle,
                origins.as_mut_ptr().cast(),
            )
        };
        self.known_action_origins(&origins, len)
    }

    /// Get the origin(s) for an analog action within an action set.
    ///
    /// Origins of devices newer than this SDK are translated to their closest known equivalent.
    pub fn get_analog_action_origins(
        &self,
        input_handle: sys::InputHandle_t,
        action_set_handle: sys::InputActionSetHandle_t,
        analog_action_handle: sys::InputAnalogActionHandle_t,
    ) -> Vec<sys::EInputActionOrigin> {
        let mut origins = [ACTION_ORIGIN_NONE; sys::STEAM_INPUT_MAX_ORIGINS as usize];
        // See `get_digital_action_origins`.
        let len = unsafe {
            sys::SteamAPI_ISteamInput_GetAnalogActionOrigins(
                self.input,
                input_handle,
                action_set_handle,
                analog_action_handle,
                origins.as_mut_ptr().cast(),
            )
        };
        self.known_action_origins(&origins, len)
    }

    fn known_action_origins(&self, origins: &[u32], len: i32) -> Vec<sys::EInputActionOrigin> {
        origins[..(len.max(0) as usize).min(origins.len())]
            .iter()
            .filter_map(|&origin| {
                let origin = if origin < ACTION_ORIGIN_COUNT {
                    origin
                } else {
                    unsafe {
                        translate_raw_action_origin(
                            self.input,
                            sys::ESteamInputType::k_ESteamInputType_Unknown,
                            origin,
                        )
                    }
                };
                // Every value below `k_EInputActionOrigin_Count` is a variant.
                (origin != ACTION_ORIGIN_NONE && origin < ACTION_ORIGIN_COUNT)
                    .then(|| unsafe { std::mem::transmute::<u32, sys::EInputActionOrigin>(origin) })
            })
            .collect()
    }

    pub fn get_motion_data(&self, input_handle: sys::InputHandle_t) -> sys::InputMotionData_t {
        unsafe { sys::SteamAPI_ISteamInput_GetMotionData(self.input, input_handle) }
    }

    /// Invokes the Steam overlay and brings up the binding screen.
    /// Returns true for success, false if overlay is disabled/unavailable.
    /// If the player is using Big Picture Mode the configuration will open in
    /// the overlay. In desktop mode a popup window version of Big Picture will
    /// be created and open the configuration.
    pub fn show_binding_panel(&self, input_handle: sys::InputHandle_t) -> bool {
        unsafe { sys::SteamAPI_ISteamInput_ShowBindingPanel(self.input, input_handle) }
    }

    /// Shutdown must be called when ending use of this interface.
    pub fn shutdown(&self) {
        unsafe {
            sys::SteamAPI_ISteamInput_Shutdown(self.input);
        }
    }
}

/// Copies a glyph path Steam returned, which on macOS mixes in Windows separators.
unsafe fn glyph_path(path: *const c_char) -> Option<PathBuf> {
    if path.is_null() {
        return None;
    }
    let path = CStr::from_ptr(path).to_str().ok()?;
    if path.is_empty() {
        return None;
    }
    Some(PathBuf::from(if cfg!(windows) {
        path.to_owned()
    } else {
        path.replace('\\', "/")
    }))
}
