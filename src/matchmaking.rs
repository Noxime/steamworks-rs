use super::*;
#[cfg(test)]
use serial_test_derive::serial;

/// Access to the steam matchmaking interface
pub struct Matchmaking<Manager> {
    pub(crate) mm: *mut sys::ISteamMatchmaking,
    pub(crate) inner: Arc<Inner<Manager>>,
}

const CALLBACK_BASE_ID: i32 = 500;

/// The visibility of a lobby
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum LobbyType {
    Private,
    FriendsOnly,
    Public,
    Invisible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LobbyId(pub(crate) u64);

impl LobbyId {
    /// Creates a `LobbyId` from a raw 64 bit value.
    ///
    /// May be useful for deserializing lobby ids from
    /// a network or save format.
    pub fn from_raw(id: u64) -> LobbyId {
        LobbyId(id)
    }

    /// Returns the raw 64 bit value of the lobby id
    ///
    /// May be useful for serializing lobby ids over a
    /// network or to a save format.
    pub fn raw(&self) -> u64 {
        self.0
    }
}

impl<Manager> Matchmaking<Manager> {
    pub fn request_lobby_list<F>(&self, cb: F)
    where
        F: FnOnce(SResult<Vec<LobbyId>>) + 'static + Send,
    {
        unsafe {
            let api_call = sys::SteamAPI_ISteamMatchmaking_RequestLobbyList(self.mm);
            register_call_result::<sys::LobbyMatchList_t, _, _>(
                &self.inner,
                api_call,
                CALLBACK_BASE_ID + 10,
                move |v, io_error| {
                    cb(if io_error {
                        Err(SteamError::IOFailure)
                    } else {
                        let mut out = Vec::with_capacity(v.m_nLobbiesMatching as usize);
                        for idx in 0..v.m_nLobbiesMatching {
                            out.push(LobbyId(sys::SteamAPI_ISteamMatchmaking_GetLobbyByIndex(
                                sys::SteamAPI_SteamMatchmaking_v009(),
                                idx as _,
                            )));
                        }
                        Ok(out)
                    })
                },
            );
        }
    }

    /// Attempts to create a new matchmaking lobby
    ///
    /// The lobby with have the visibility of the of the passed
    /// `LobbyType` and a limit of `max_members` inside it.
    /// The `max_members` may not be higher than 250.
    ///
    /// # Triggers
    ///
    /// * `LobbyEnter`
    /// * `LobbyCreated`
    pub fn create_lobby<F>(&self, ty: LobbyType, max_members: u32, cb: F)
    where
        F: FnOnce(SResult<LobbyId>) + 'static + Send,
    {
        assert!(max_members <= 250); // Steam API limits
        unsafe {
            let ty = match ty {
                LobbyType::Private => sys::ELobbyType::k_ELobbyTypePrivate,
                LobbyType::FriendsOnly => sys::ELobbyType::k_ELobbyTypeFriendsOnly,
                LobbyType::Public => sys::ELobbyType::k_ELobbyTypePublic,
                LobbyType::Invisible => sys::ELobbyType::k_ELobbyTypeInvisible,
            };
            let api_call =
                sys::SteamAPI_ISteamMatchmaking_CreateLobby(self.mm, ty, max_members as _);
            register_call_result::<sys::LobbyCreated_t, _, _>(
                &self.inner,
                api_call,
                CALLBACK_BASE_ID + 13,
                move |v, io_error| {
                    cb(if io_error {
                        Err(SteamError::IOFailure)
                    } else if v.m_eResult != sys::EResult::k_EResultOK {
                        Err(v.m_eResult.into())
                    } else {
                        Ok(LobbyId(v.m_ulSteamIDLobby))
                    })
                },
            );
        }
    }

    /// Tries to join the lobby with the given ID
    pub fn join_lobby<F>(&self, lobby: LobbyId, cb: F)
    where
        F: FnOnce(Result<LobbyId, ()>) + 'static + Send,
    {
        unsafe {
            let api_call = sys::SteamAPI_ISteamMatchmaking_JoinLobby(self.mm, lobby.0);
            register_call_result::<sys::LobbyEnter_t, _, _>(
                &self.inner,
                api_call,
                CALLBACK_BASE_ID + 4,
                move |v, io_error| {
                    cb(if io_error {
                        Err(())
                    } else if v.m_EChatRoomEnterResponse != 1 {
                        Err(())
                    } else {
                        Ok(LobbyId(v.m_ulSteamIDLobby))
                    })
                },
            );
        }
    }

    /// Returns the lobby metadata associated with the specified key from the
    /// specified lobby.
    pub fn lobby_data(&self, lobby: LobbyId, key: &str) -> Option<&str> {
        let key = CString::new(key).unwrap();
        let data = unsafe {
            let data = sys::SteamAPI_ISteamMatchmaking_GetLobbyData(self.mm, lobby.0, key.as_ptr());
            let data = CStr::from_ptr(data);

            data
        };

        let data = data.to_str().unwrap();

        match data.is_empty() {
            false => Some(data),
            true => None,
        }
    }

    /// Exits the passed lobby
    pub fn leave_lobby(&self, lobby: LobbyId) {
        unsafe {
            sys::SteamAPI_ISteamMatchmaking_LeaveLobby(self.mm, lobby.0);
        }
    }

    /// Returns the current limit on the number of players in a lobby.
    ///
    /// Returns `[None]` if no metadata is available for the specified lobby.
    pub fn lobby_member_limit(&self, lobby: LobbyId) -> Option<usize> {
        unsafe {
            let count = sys::SteamAPI_ISteamMatchmaking_GetLobbyMemberLimit(self.mm, lobby.0);
            match count {
                0 => None,
                _ => Some(count as usize),
            }
        }
    }

    /// Returns the steam id of the current owner of the passed lobby
    pub fn lobby_owner(&self, lobby: LobbyId) -> SteamId {
        unsafe {
            SteamId(sys::SteamAPI_ISteamMatchmaking_GetLobbyOwner(
                self.mm, lobby.0,
            ))
        }
    }

    /// Returns the number of players in a lobby.
    ///
    /// Useful if you are not currently in the lobby
    pub fn lobby_member_count(&self, lobby: LobbyId) -> usize {
        unsafe {
            let count = sys::SteamAPI_ISteamMatchmaking_GetNumLobbyMembers(self.mm, lobby.0);
            count as usize
        }
    }

    /// Returns a list of members currently in the lobby
    pub fn lobby_members(&self, lobby: LobbyId) -> Vec<SteamId> {
        unsafe {
            let count = sys::SteamAPI_ISteamMatchmaking_GetNumLobbyMembers(self.mm, lobby.0);
            let mut members = Vec::with_capacity(count as usize);
            for idx in 0..count {
                members.push(SteamId(
                    sys::SteamAPI_ISteamMatchmaking_GetLobbyMemberByIndex(self.mm, lobby.0, idx),
                ))
            }
            members
        }
    }

    /// Sets whether or not a lobby is joinable by other players. This always defaults to enabled
    /// for a new lobby.
    ///
    /// If joining is disabled, then no players can join, even if they are a friend or have been
    /// invited.
    ///
    /// Lobbies with joining disabled will not be returned from a lobby search.
    ///
    /// Returns true on success, false if the current user doesn't own the lobby.
    pub fn set_lobby_joinable(&self, lobby: LobbyId, joinable: bool) -> bool {
        unsafe { sys::SteamAPI_ISteamMatchmaking_SetLobbyJoinable(self.mm, lobby.0, joinable) }
    }
}

#[test]
#[serial]
fn test_lobby() {
    let (client, single) = Client::init().unwrap();
    let mm = client.matchmaking();

    mm.request_lobby_list(|v| {
        println!("List: {:?}", v);
    });
    mm.create_lobby(LobbyType::Private, 4, |v| {
        println!("Create: {:?}", v);
    });

    for _ in 0..100 {
        single.run_callbacks();
        ::std::thread::sleep(::std::time::Duration::from_millis(100));
    }
}
