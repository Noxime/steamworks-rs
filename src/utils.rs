use super::*;

use std::ffi::CStr;
use std::os::raw::c_char;
use std::panic;
use std::process::abort;
use std::sync::RwLock;

/// Access to the steam utils interface
pub struct Utils<Manager> {
    pub(crate) utils: *mut sys::ISteamUtils,
    pub(crate) _inner: Arc<Inner<Manager>>,
}

#[derive(Clone, Debug)]
pub struct GamepadTextInputDismissed {
    pub submitted_text_len: Option<u32>,
}

unsafe impl Callback for GamepadTextInputDismissed {
    const ID: i32 = 714;
    const SIZE: i32 = ::std::mem::size_of::<sys::GamepadTextInputDismissed_t>() as i32;

    unsafe fn from_raw(raw: *mut c_void) -> Self {
        let val = &mut *(raw as *mut sys::GamepadTextInputDismissed_t);
        GamepadTextInputDismissed {
            submitted_text_len: val.m_bSubmitted.then_some(val.m_unSubmittedText),
        }
    }
}

#[derive(Clone, Debug)]
pub struct FloatingGamepadTextInputDismissed;

unsafe impl Callback for FloatingGamepadTextInputDismissed {
    const ID: i32 = 738;
    const SIZE: i32 = ::std::mem::size_of::<sys::FloatingGamepadTextInputDismissed_t>() as i32;

    unsafe fn from_raw(_: *mut c_void) -> Self {
        FloatingGamepadTextInputDismissed
    }
}

pub enum NotificationPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

pub enum GamepadTextInputMode {
    Normal,
    Password,
}

impl From<GamepadTextInputMode> for sys::EGamepadTextInputMode {
    fn from(mode: GamepadTextInputMode) -> Self {
        match mode {
            GamepadTextInputMode::Normal => {
                sys::EGamepadTextInputMode::k_EGamepadTextInputModeNormal
            }
            GamepadTextInputMode::Password => {
                sys::EGamepadTextInputMode::k_EGamepadTextInputModePassword
            }
        }
    }
}

pub enum GamepadTextInputLineMode {
    SingleLine,
    MultipleLines,
}

impl From<GamepadTextInputLineMode> for sys::EGamepadTextInputLineMode {
    fn from(mode: GamepadTextInputLineMode) -> Self {
        match mode {
            GamepadTextInputLineMode::SingleLine => {
                sys::EGamepadTextInputLineMode::k_EGamepadTextInputLineModeSingleLine
            }
            GamepadTextInputLineMode::MultipleLines => {
                sys::EGamepadTextInputLineMode::k_EGamepadTextInputLineModeMultipleLines
            }
        }
    }
}

pub enum FloatingGamepadTextInputMode {
    SingleLine,
    MultipleLines,
    Email,
    Numeric,
}

impl From<FloatingGamepadTextInputMode> for sys::EFloatingGamepadTextInputMode {
    fn from(mode: FloatingGamepadTextInputMode) -> Self {
        match mode {
            FloatingGamepadTextInputMode::SingleLine => {
                sys::EFloatingGamepadTextInputMode::k_EFloatingGamepadTextInputModeModeSingleLine
            }
            FloatingGamepadTextInputMode::MultipleLines => {
                sys::EFloatingGamepadTextInputMode::k_EFloatingGamepadTextInputModeModeMultipleLines
            }
            FloatingGamepadTextInputMode::Email => {
                sys::EFloatingGamepadTextInputMode::k_EFloatingGamepadTextInputModeModeEmail
            }
            FloatingGamepadTextInputMode::Numeric => {
                sys::EFloatingGamepadTextInputMode::k_EFloatingGamepadTextInputModeModeNumeric
            }
        }
    }
}

lazy_static! {
    /// Global rust warning callback
    static ref WARNING_CALLBACK: RwLock<Option<Box<dyn Fn(i32, &CStr) + Send + Sync>>> = RwLock::new(None);
}

/// C function to pass as the real callback, which forwards to the `WARNING_CALLBACK` if any
unsafe extern "C" fn c_warning_callback(level: i32, msg: *const c_char) {
    let lock = WARNING_CALLBACK.read().expect("warning func lock poisoned");
    let cb = match lock.as_ref() {
        Some(cb) => cb,
        None => {
            return;
        }
    };

    let s = CStr::from_ptr(msg);

    let res = panic::catch_unwind(panic::AssertUnwindSafe(|| cb(level, s)));
    if let Err(err) = res {
        if let Some(err) = err.downcast_ref::<&str>() {
            println!("Steam warning callback panicked: {}", err);
        } else if let Some(err) = err.downcast_ref::<String>() {
            println!("Steam warning callback panicked: {}", err);
        } else {
            println!("Steam warning callback panicked");
        }
        abort();
    }
}

impl<Manager> Utils<Manager> {
    /// Returns the app ID of the current process
    pub fn app_id(&self) -> AppId {
        unsafe { AppId(sys::SteamAPI_ISteamUtils_GetAppID(self.utils)) }
    }

    /// Returns the country code of the current user based on their IP
    pub fn ip_country(&self) -> String {
        unsafe {
            let ipcountry = sys::SteamAPI_ISteamUtils_GetIPCountry(self.utils);
            let ipcountry = CStr::from_ptr(ipcountry);
            ipcountry.to_string_lossy().into_owned()
        }
    }

    /// Returns the language the steam client is currently
    /// running in.
    ///
    /// Generally you want `Apps::current_game_language` instead of this
    pub fn ui_language(&self) -> String {
        unsafe {
            let lang = sys::SteamAPI_ISteamUtils_GetSteamUILanguage(self.utils);
            let lang = CStr::from_ptr(lang);
            lang.to_string_lossy().into_owned()
        }
    }

    /// Returns the current real time on the Steam servers
    /// in Unix epoch format (seconds since 1970/1/1 UTC).
    pub fn get_server_real_time(&self) -> u32 {
        unsafe { sys::SteamAPI_ISteamUtils_GetServerRealTime(self.utils) }
    }

    /// Sets the position on the screen where popups from the steam overlay
    /// should appear and display themselves in.
    pub fn set_overlay_notification_position(&self, position: NotificationPosition) {
        unsafe {
            let position = match position {
                NotificationPosition::TopLeft => sys::ENotificationPosition::k_EPositionTopLeft,
                NotificationPosition::TopRight => sys::ENotificationPosition::k_EPositionTopRight,
                NotificationPosition::BottomLeft => {
                    sys::ENotificationPosition::k_EPositionBottomLeft
                }
                NotificationPosition::BottomRight => {
                    sys::ENotificationPosition::k_EPositionBottomRight
                }
            };
            sys::SteamAPI_ISteamUtils_SetOverlayNotificationPosition(self.utils, position);
        }
    }

    /// Sets the Steam warning callback, which is called to emit warning messages.
    ///
    /// The passed-in function takes two arguments: a severity level (0 = info, 1 = warning) and
    /// the message itself.
    ///
    /// See [Steamwork's debugging page](https://partner.steamgames.com/doc/sdk/api/debugging) for more info.
    pub fn set_warning_callback<F>(&self, cb: F)
    where
        F: Fn(i32, &CStr) + Send + Sync + 'static,
    {
        let mut lock = WARNING_CALLBACK
            .write()
            .expect("warning func lock poisoned");
        *lock = Some(Box::new(cb));
        unsafe {
            sys::SteamAPI_ISteamUtils_SetWarningMessageHook(self.utils, Some(c_warning_callback));
        }
    }

    /// Gets the gamepad text input from the Big Picture overlay.
    ///
    /// This must be called within the `show_gamepad_text_input` callback. Returns `Some(String)` if user submitted the
    /// text, and `None` otherwise
    pub fn get_entered_gamepad_text_input(
        &self,
        dismissed_data: &GamepadTextInputDismissed,
    ) -> Option<String> {
        unsafe {
            let len = dismissed_data.submitted_text_len?;
            let mut buf = vec![0u8; len as usize];

            sys::SteamAPI_ISteamUtils_GetEnteredGamepadTextInput(
                self.utils,
                buf.as_mut_ptr() as *mut i8,
                len,
            )
            .then(|| String::from_utf8(buf).expect("Steamworks returned invalid UTF-8 string"))
        }
    }

    /// Checks if Steam is running on a Steam Deck device.
    pub fn is_steam_running_on_steam_deck(&self) -> bool {
        unsafe { sys::SteamAPI_ISteamUtils_IsSteamRunningOnSteamDeck(self.utils) }
    }

    /// Activates the Big Picture text input dialog which only supports gamepad input.
    pub fn show_gamepad_text_input<F>(
        &self,
        input_mode: GamepadTextInputMode,
        input_line_mode: GamepadTextInputLineMode,
        description: &str,
        max_characters: u32,
        existing_text: Option<&str>,
        dismissed_cb: F,
    ) -> bool
    where
        F: FnMut(GamepadTextInputDismissed) + 'static + Send,
    {
        unsafe {
            let description = CString::new(description).unwrap();
            let existing_text = existing_text.map(|s| CString::new(s).unwrap());
            register_callback(&self._inner, dismissed_cb);
            sys::SteamAPI_ISteamUtils_ShowGamepadTextInput(
                self.utils,
                input_mode.into(),
                input_line_mode.into(),
                description.as_ptr(),
                max_characters,
                existing_text
                    .as_ref()
                    .map(|s| s.as_ptr())
                    .unwrap_or(std::ptr::null()),
            )
        }
    }

    /// Opens a floating keyboard over the game content and sends OS keyboard keys directly to the game.
    ///
    /// The text field position is specified in pixels relative the origin of the game window and is used to
    /// position the floating keyboard in a way that doesn't cover the text field.
    ///
    /// Callback is triggered when user dismisses the text input
    pub fn show_floating_gamepad_text_input<F>(
        &self,
        keyboard_mode: FloatingGamepadTextInputMode,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        mut dismissed_cb: F,
    ) -> bool
    where
        F: FnMut() + 'static + Send, // TODO: Support FnOnce callbacks
    {
        unsafe {
            register_callback(&self._inner, move |_: FloatingGamepadTextInputDismissed| {
                dismissed_cb();
            });
            sys::SteamAPI_ISteamUtils_ShowFloatingGamepadTextInput(
                self.utils,
                keyboard_mode.into(),
                x,
                y,
                width,
                height,
            )
        }
    }
}

pub(crate) struct SteamParamStringArray(Vec<*mut i8>);
impl Drop for SteamParamStringArray {
    fn drop(&mut self) {
        for c_string in &self.0 {
            unsafe { drop(CString::from_raw(*c_string)) };
        }
    }
}
impl SteamParamStringArray {
    pub(crate) fn new<S: AsRef<str>>(vec: &[S]) -> SteamParamStringArray {
        SteamParamStringArray(
            vec.into_iter()
                .map(|s| {
                    CString::new(s.as_ref())
                        .expect("String passed could not be converted to a c string")
                        .into_raw()
                })
                .collect(),
        )
    }

    pub(crate) fn as_raw(&mut self) -> sys::SteamParamStringArray_t {
        sys::SteamParamStringArray_t {
            m_nNumStrings: self.0.len() as i32,
            m_ppStrings: self.0.as_mut_ptr() as *mut *const i8,
        }
    }
}
