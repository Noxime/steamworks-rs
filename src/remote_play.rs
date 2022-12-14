use super::*;

pub struct RemotePlay<Manager> {
    pub(crate) rp: *mut sys::ISteamRemotePlay,
    pub(crate) inner: Arc<Inner<Manager>>,
}

impl<Manager> Clone for RemotePlay<Manager> {
    fn clone(&self) -> Self {
        RemotePlay {
            inner: self.inner.clone(),
            rp: self.rp,
        }
    }
}

impl<Manager> RemotePlay<Manager> {
    /// Return a list of all active Remote Play sessions
    pub fn sessions(&self) -> Vec<RemotePlaySession<Manager>> {
        unsafe {
            let count = sys::SteamAPI_ISteamRemotePlay_GetSessionCount(self.rp);
            let mut sessions = Vec::with_capacity(count as usize);

            for i in 0..count {
                let id = sys::SteamAPI_ISteamRemotePlay_GetSessionID(self.rp, i as i32);

                // Session might be invalid if it ended after GetSessionCount
                if id == 0 {
                    continue;
                }

                sessions.push(self.session(RemotePlaySessionId::from_raw(id)))
            }

            sessions
        }
    }

    /// Get a remote play session from a session ID. The session may or may not be valid or active
    pub fn session(&self, session: RemotePlaySessionId) -> RemotePlaySession<Manager> {
        RemotePlaySession {
            session,
            rp: self.rp,
            _inner: self.inner.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RemotePlaySessionId(pub(crate) u32);

impl RemotePlaySessionId {
    /// Creates a `RemotePlaySessionId` from a raw 32 bit value.
    ///
    /// May be useful for deserializing session ids from
    /// a network or save format.
    pub fn from_raw(id: u32) -> RemotePlaySessionId {
        RemotePlaySessionId(id)
    }

    /// Returns the raw 32 bit value of the lobby id
    ///
    /// May be useful for serializing session ids over a
    /// network or to a save format.
    pub fn raw(&self) -> u32 {
        self.0
    }
}

pub struct RemotePlaySession<Manager> {
    session: RemotePlaySessionId,
    pub(crate) rp: *mut sys::ISteamRemotePlay,
    pub(crate) _inner: Arc<Inner<Manager>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SteamDeviceFormFactor {
    Phone,
    Tablet,
    Computer,
    TV,
}

impl<Manager> RemotePlaySession<Manager> {
    /// Get the user associated with this Remote Play session. This is either the logged in user or a friend when Remote
    /// Playing Together.
    pub fn user(&self) -> SteamId {
        unsafe {
            SteamId(sys::SteamAPI_ISteamRemotePlay_GetSessionSteamID(
                self.rp,
                self.session.raw(),
            ))
        }
    }

    /// Gets the client device name for this session. Returns `None` if the session has expired
    pub fn client_name(&self) -> Option<String> {
        unsafe {
            let name =
                sys::SteamAPI_ISteamRemotePlay_GetSessionClientName(self.rp, self.session.raw());

            if name.is_null() {
                return None;
            }

            let name = CStr::from_ptr(name);
            Some(name.to_string_lossy().into_owned())
        }
    }

    /// Gets the client device form factor for this session. Returns `None` if the session has expired or if the form
    /// factor is unknown
    pub fn client_form_factor(&self) -> Option<SteamDeviceFormFactor> {
        unsafe {
            use SteamDeviceFormFactor::*;
            match sys::SteamAPI_ISteamRemotePlay_GetSessionClientFormFactor(
                self.rp,
                self.session.raw(),
            ) {
                sys::ESteamDeviceFormFactor::k_ESteamDeviceFormFactorPhone => Some(Phone),
                sys::ESteamDeviceFormFactor::k_ESteamDeviceFormFactorTablet => Some(Tablet),
                sys::ESteamDeviceFormFactor::k_ESteamDeviceFormFactorComputer => Some(Computer),
                sys::ESteamDeviceFormFactor::k_ESteamDeviceFormFactorTV => Some(TV),
                _ => None,
            }
        }
    }

    /// Gets the client device resolution for this session. Returns `None` if the session has expired
    pub fn client_resolution(&self) -> Option<(u32, u32)> {
        unsafe {
            let mut width = 0;
            let mut height = 0;

            sys::SteamAPI_ISteamRemotePlay_BGetSessionClientResolution(
                self.rp,
                self.session.raw(),
                &mut width,
                &mut height,
            )
            .then_some((width as u32, height as u32))
        }
    }

    /// Invites a friend to join the game using Remote Play Together
    pub fn invite(&self, friend: SteamId) -> bool {
        unsafe {
            sys::SteamAPI_ISteamRemotePlay_BSendRemotePlayTogetherInvite(self.rp, friend.raw())
        }
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
/// A remote play session was established
pub struct RemotePlayConnected {
    /// The remote play session ID we just connected to
    pub session: RemotePlaySessionId,
}

unsafe impl Callback for RemotePlayConnected {
    const ID: i32 = sys::SteamRemotePlaySessionConnected_t_k_iCallback as i32;
    const SIZE: i32 = ::std::mem::size_of::<sys::SteamRemotePlaySessionConnected_t>() as i32;

    unsafe fn from_raw(raw: *mut c_void) -> Self {
        let val = &mut *(raw as *mut sys::SteamRemotePlaySessionConnected_t);
        RemotePlayConnected {
            session: RemotePlaySessionId::from_raw(val.m_unSessionID),
        }
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
/// A remote play session was closed
pub struct RemotePlayDisconnected {
    /// The remote play session ID that just disconnected
    pub session: RemotePlaySessionId,
}

unsafe impl Callback for RemotePlayDisconnected {
    const ID: i32 = sys::SteamRemotePlaySessionDisconnected_t_k_iCallback as i32;
    const SIZE: i32 = ::std::mem::size_of::<sys::SteamRemotePlaySessionDisconnected_t>() as i32;

    unsafe fn from_raw(raw: *mut c_void) -> Self {
        let val = &mut *(raw as *mut sys::SteamRemotePlaySessionDisconnected_t);
        RemotePlayDisconnected {
            session: RemotePlaySessionId::from_raw(val.m_unSessionID),
        }
    }
}
