
use super::*;

/// Access to the steam matchmaking interface
pub struct Matchmaking<Manager> {
    pub(crate) mm: *mut sys::ISteamMatchmaking,
    pub(crate) inner: Arc<Inner<Manager>>,
}

const CALLBACK_BASE_ID: i32 = 500;

/// The visibility of a lobby
pub enum LobbyType {
    Private,
    FriendsOnly,
    Public ,
    Invisible,
}

#[derive(Debug)]
pub struct LobbyId(pub u64);

impl <Manager> Matchmaking<Manager> {

    pub fn request_lobby_list<F>(&self, mut cb: F)
        where F: FnMut(Result<Vec<LobbyId>, SteamError>) + 'static + Send + Sync
    {
        unsafe {
            let api_call = sys::SteamAPI_ISteamMatchmaking_RequestLobbyList(self.mm);
            register_call_result::<sys::LobbyMatchList_t, _, _>(
                &self.inner, api_call, CALLBACK_BASE_ID + 10,
                move |v, io_error| {
                    cb(if io_error {
                        Err(SteamError::IOFailure)
                    } else {
                        let mut out = Vec::with_capacity(v.get_m_nLobbiesMatching() as usize);
                        for idx in 0 .. v.get_m_nLobbiesMatching() {
                            out.push(LobbyId(sys::SteamAPI_ISteamMatchmaking_GetLobbyByIndex(sys::steam_rust_get_matchmaking(), idx as _)));
                        }
                        Ok(out)
                    })
            });
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
    pub fn create_lobby<F>(&self, ty: LobbyType, max_members: u32, mut cb: F)
        where F: FnMut(Result<LobbyId, SteamError>) + 'static + Send
    {
        assert!(max_members <= 250); // Steam API limits
        unsafe {
            let ty = match ty {
                LobbyType::Private => sys::ELobbyType_k_ELobbyTypePrivate,
                LobbyType::FriendsOnly => sys::ELobbyType_k_ELobbyTypeFriendsOnly,
                LobbyType::Public => sys::ELobbyType_k_ELobbyTypePublic,
                LobbyType::Invisible => sys::ELobbyType_k_ELobbyTypeInvisible,
            };
            let api_call = sys::SteamAPI_ISteamMatchmaking_CreateLobby(self.mm, ty, max_members as _);
            register_call_result::<sys::LobbyCreated_t, _, _>(
                &self.inner, api_call, CALLBACK_BASE_ID + 13,
                move |v, io_error| {
                    cb(if io_error {
                        Err(SteamError::IOFailure)
                    } else if v.get_m_eResult() != sys::EResult_k_EResultOK {
                        Err(v.get_m_eResult().into())
                    } else {
                        Ok(LobbyId(v.get_m_ulSteamIDLobby()))
                    })
            });
        }
    }

    /// Tries to join the lobby with the given ID
    pub fn join_lobby<F>(&self, lobby: LobbyId, mut cb: F)
        where F: FnMut(Result<LobbyId, ()>) + 'static + Send
    {
        unsafe {
            let api_call = sys::SteamAPI_ISteamMatchmaking_JoinLobby(self.mm, lobby.0);
            register_call_result::<sys::LobbyEnter_t, _, _>(
                &self.inner, api_call, CALLBACK_BASE_ID + 4,
                move |v, io_error| {
                    cb(if io_error {
                        Err(())
                    } else if v.get_m_EChatRoomEnterResponse() != 1 {
                        Err(())
                    } else {
                        Ok(LobbyId(v.get_m_ulSteamIDLobby()))
                    })
            });
        }
    }

    /// Exits the passed lobby
    pub fn leave_lobby(&self, lobby: LobbyId) {
        unsafe {
            sys::SteamAPI_ISteamMatchmaking_LeaveLobby(self.mm, lobby.0);
        }
    }
}

#[test]
fn test_lobby() {
    let (client, single) = Client::init().unwrap();
    let mm = client.matchmaking();

    mm.request_lobby_list(|v| {
        println!("List: {:?}", v);
    });
    mm.create_lobby(LobbyType::Private, 4, |v| {
        println!("Create: {:?}", v);
    });

    for _ in 0 .. 100 {
        single.run_callbacks();
        ::std::thread::sleep(::std::time::Duration::from_millis(100));
    }
}