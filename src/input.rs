use sys::InputHandle_t;

use super::*;

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

/// The size of a glyph image returned by [`Input::get_glyph_png_for_action_origin`].
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum InputGlyphSize {
    /// Small glyph.
    Small,
    /// Medium glyph.
    Medium,
    /// Large glyph.
    Large,
}

impl From<InputGlyphSize> for sys::ESteamInputGlyphSize {
    fn from(size: InputGlyphSize) -> Self {
        match size {
            InputGlyphSize::Small => sys::ESteamInputGlyphSize::k_ESteamInputGlyphSize_Small,
            InputGlyphSize::Medium => sys::ESteamInputGlyphSize::k_ESteamInputGlyphSize_Medium,
            InputGlyphSize::Large => sys::ESteamInputGlyphSize::k_ESteamInputGlyphSize_Large,
        }
    }
}

/// The base look of a glyph, see [`InputGlyphStyle`].
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default)]
pub enum InputGlyphBaseStyle {
    /// Glyphs with a transparent "knockout" cut-out of the button symbol.
    #[default]
    Knockout,
    /// Light glyphs, intended for dark backgrounds.
    Light,
    /// Dark glyphs, intended for light backgrounds.
    Dark,
}

/// The style of a glyph returned by [`Input::get_glyph_png_for_action_origin`] and
/// [`Input::get_glyph_svg_for_action_origin`].
///
/// A style is one [`InputGlyphBaseStyle`] plus two independent options for how
/// the A/B/X/Y face buttons are drawn. The [`Default`] style is
/// [`InputGlyphBaseStyle::Knockout`] with both options disabled (flags value `0`).
///
/// Note that this does not match the legacy [`Input::get_glyph_for_action_origin`],
/// which has been observed to return glyphs in the dark style. Use
/// `InputGlyphStyle::new(InputGlyphBaseStyle::Dark)` for a similar look.
///
/// ```
/// use steamworks::{InputGlyphBaseStyle, InputGlyphStyle};
///
/// let style = InputGlyphStyle::new(InputGlyphBaseStyle::Dark).solid_abxy(true);
/// assert_eq!(style.to_flags(), 2 | 32);
/// ```
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default)]
pub struct InputGlyphStyle {
    /// The base look of the glyph.
    pub base: InputGlyphBaseStyle,
    /// Use neutral colours for the A/B/X/Y buttons.
    pub neutral_color_abxy: bool,
    /// Use solid (filled) A/B/X/Y buttons.
    pub solid_abxy: bool,
}

impl InputGlyphStyle {
    /// Creates a style with the given base look and no extra options.
    pub fn new(base: InputGlyphBaseStyle) -> Self {
        Self {
            base,
            ..Self::default()
        }
    }

    /// Sets whether A/B/X/Y buttons use neutral colours.
    pub fn neutral_color_abxy(mut self, enabled: bool) -> Self {
        self.neutral_color_abxy = enabled;
        self
    }

    /// Sets whether A/B/X/Y buttons are drawn solid.
    pub fn solid_abxy(mut self, enabled: bool) -> Self {
        self.solid_abxy = enabled;
        self
    }

    /// Converts the style to the `ESteamInputGlyphStyle` flags value Steam expects.
    pub fn to_flags(self) -> u32 {
        let base = match self.base {
            InputGlyphBaseStyle::Knockout => {
                sys::ESteamInputGlyphStyle::ESteamInputGlyphStyle_Knockout
            }
            InputGlyphBaseStyle::Light => sys::ESteamInputGlyphStyle::ESteamInputGlyphStyle_Light,
            InputGlyphBaseStyle::Dark => sys::ESteamInputGlyphStyle::ESteamInputGlyphStyle_Dark,
        } as u32;
        let mut flags = base;
        if self.neutral_color_abxy {
            flags |= sys::ESteamInputGlyphStyle::ESteamInputGlyphStyle_NeutralColorABXY as u32;
        }
        if self.solid_abxy {
            flags |= sys::ESteamInputGlyphStyle::ESteamInputGlyphStyle_SolidABXY as u32;
        }
        flags
    }
}

impl From<InputGlyphStyle> for u32 {
    fn from(style: InputGlyphStyle) -> Self {
        style.to_flags()
    }
}

/// Replaces `\` with `/` in a path returned by Steam.
///
/// Steam builds glyph paths with mixed separators on non-Windows platforms (for
/// example `.../MacOS\controller_base\images\api\knockout/button.png`). There
/// the backslashes are ordinary filename characters, so the file does not exist
/// as returned.
fn replace_backslashes(path: &str) -> String {
    path.replace('\\', "/")
}

/// Copies a file path owned by Steam, mapping null and empty strings to `None`.
///
/// On non-Windows targets `\` is rewritten as `/`, see [`replace_backslashes`];
/// Windows paths are left untouched. Only use this for paths, since it would
/// mangle other strings such as display names.
///
/// # Safety
///
/// `ptr` must be null or point to a valid nul-terminated C string.
unsafe fn owned_path_from_steam(ptr: *const std::os::raw::c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    let value = CStr::from_ptr(ptr).to_string_lossy().into_owned();
    if value.is_empty() {
        None
    } else if cfg!(windows) {
        Some(value)
    } else {
        Some(replace_backslashes(&value))
    }
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
    ///
    /// Returns an empty string if Steam has no glyph for the origin (for example
    /// `k_EInputActionOrigin_None`). See [`Input::get_glyph_png_for_action_origin`]
    /// for a variant that lets you choose the size and style.
    pub fn get_glyph_for_action_origin(&self, action_origin: sys::EInputActionOrigin) -> String {
        unsafe {
            let glyph_path =
                sys::SteamAPI_ISteamInput_GetGlyphForActionOrigin_Legacy(self.input, action_origin);
            owned_path_from_steam(glyph_path).unwrap_or_default()
        }
    }

    /// Returns the path to a PNG glyph for an input action origin, in the requested
    /// size and style, or `None` if Steam has no glyph for it.
    ///
    /// [`Input::init`] must be called before using this.
    ///
    /// Unlike [`Input::get_glyph_for_action_origin`], which wraps the legacy API
    /// and always uses Steam's default style, this lets you choose the size and
    /// [`InputGlyphStyle`].
    pub fn get_glyph_png_for_action_origin(
        &self,
        action_origin: sys::EInputActionOrigin,
        size: InputGlyphSize,
        style: InputGlyphStyle,
    ) -> Option<String> {
        unsafe {
            let glyph_path = sys::SteamAPI_ISteamInput_GetGlyphPNGForActionOrigin(
                self.input,
                action_origin,
                size.into(),
                style.to_flags(),
            );
            owned_path_from_steam(glyph_path)
        }
    }

    /// Returns the path to an SVG glyph for an input action origin in the requested
    /// style, or `None` if Steam has no glyph for it.
    ///
    /// [`Input::init`] must be called before using this.
    ///
    /// Unlike [`Input::get_glyph_for_action_origin`], which wraps the legacy API
    /// and always uses Steam's default style, this lets you choose the
    /// [`InputGlyphStyle`].
    pub fn get_glyph_svg_for_action_origin(
        &self,
        action_origin: sys::EInputActionOrigin,
        style: InputGlyphStyle,
    ) -> Option<String> {
        unsafe {
            let glyph_path = sys::SteamAPI_ISteamInput_GetGlyphSVGForActionOrigin(
                self.input,
                action_origin,
                style.to_flags(),
            );
            owned_path_from_steam(glyph_path)
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
    pub fn get_digital_action_origins(
        &self,
        input_handle: sys::InputHandle_t,
        action_set_handle: sys::InputActionSetHandle_t,
        digital_action_handle: sys::InputDigitalActionHandle_t,
    ) -> Vec<sys::EInputActionOrigin> {
        unsafe {
            let mut origins = Vec::with_capacity(sys::STEAM_INPUT_MAX_ORIGINS as usize);
            let len = sys::SteamAPI_ISteamInput_GetDigitalActionOrigins(
                self.input,
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
        unsafe {
            let mut origins = Vec::with_capacity(sys::STEAM_INPUT_MAX_ORIGINS as usize);
            let len = sys::SteamAPI_ISteamInput_GetAnalogActionOrigins(
                self.input,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glyph_style_flags() {
        use InputGlyphBaseStyle::*;

        assert_eq!(InputGlyphStyle::default().to_flags(), 0);
        assert_eq!(InputGlyphStyle::new(Knockout).to_flags(), 0);
        assert_eq!(InputGlyphStyle::new(Light).to_flags(), 1);
        assert_eq!(InputGlyphStyle::new(Dark).to_flags(), 2);

        for (base, base_flag) in [(Knockout, 0), (Light, 1), (Dark, 2)] {
            let style = InputGlyphStyle::new(base);
            assert_eq!(style.neutral_color_abxy(true).to_flags(), base_flag | 16);
            assert_eq!(style.solid_abxy(true).to_flags(), base_flag | 32);
            assert_eq!(
                style.neutral_color_abxy(true).solid_abxy(true).to_flags(),
                base_flag | 16 | 32
            );
            assert_eq!(
                style
                    .neutral_color_abxy(true)
                    .neutral_color_abxy(false)
                    .to_flags(),
                base_flag
            );
        }
        assert_eq!(u32::from(InputGlyphStyle::new(Dark).solid_abxy(true)), 34);
    }

    #[test]
    fn glyph_size_conversion() {
        assert_eq!(
            sys::ESteamInputGlyphSize::from(InputGlyphSize::Small) as u32,
            0
        );
        assert_eq!(
            sys::ESteamInputGlyphSize::from(InputGlyphSize::Medium) as u32,
            1
        );
        assert_eq!(
            sys::ESteamInputGlyphSize::from(InputGlyphSize::Large) as u32,
            2
        );
    }

    #[test]
    fn backslash_replacement() {
        assert_eq!(
            replace_backslashes(
                r"/MacOS\controller_base\images\api\knockout/shared_color_button_a_sm.png"
            ),
            "/MacOS/controller_base/images/api/knockout/shared_color_button_a_sm.png"
        );
        assert_eq!(replace_backslashes("a/b/c.png"), "a/b/c.png");
        assert_eq!(replace_backslashes(""), "");
    }

    #[test]
    fn steam_path_conversion() {
        unsafe {
            assert_eq!(owned_path_from_steam(std::ptr::null()), None);
            assert_eq!(owned_path_from_steam(c"".as_ptr()), None);
            assert_eq!(
                owned_path_from_steam(c"glyph.png".as_ptr()),
                Some("glyph.png".to_owned())
            );
        }
    }
}
