
use super::*;

/// Access to the steam matchmaking interface
pub struct Matchmaking<Manager> {
    pub(crate) mm: *mut sys::ISteamMatchmaking,
    pub(crate) inner: Arc<Inner<Manager>>,
}

const CALLBACK_BASE_ID: i32 = 500;

pub enum LobbyType {
    Private,
    FriendsOnly,
    Public ,
    Invisible,
}

#[derive(Debug)]
pub struct LobbyId(u64);

impl <Manager> Matchmaking<Manager> {

    pub fn request_lobby_list<F>(&self, mut cb: F)
        where F: FnMut(Result<Vec<LobbyId>, SteamError>) + 'static + Send + Sync
    {
        unsafe {
            let api_call = sys::SteamAPI_ISteamMatchmaking_RequestLobbyList(self.mm);
            register_call_result::<sys::LobbyMatchList, _, _>(
                &self.inner, api_call, CALLBACK_BASE_ID + 10,
                move |v, io_error| {
                   cb(if io_error {
                      Err(SteamError::IOFailure)
                   } else {
                       let mut out = Vec::with_capacity(v.lobbies_matching as usize);
                       for idx in 0 .. v.lobbies_matching {
                           out.push(LobbyId(sys::SteamAPI_ISteamMatchmaking_GetLobbyByIndex(sys::steam_rust_get_matchmaking(), idx as _)));
                       }
                       Ok(out)
                   })
            });
        }
    }

    pub fn create_lobby<F>(&self, ty: LobbyType, max_members: u32, mut cb: F)
        where F: FnMut(Result<LobbyId, SteamError>) + 'static + Send + Sync
    {
        unsafe {
            let ty = match ty {
                LobbyType::Private => sys::LobbyType::Private,
                LobbyType::FriendsOnly => sys::LobbyType::FriendsOnly,
                LobbyType::Public => sys::LobbyType::Public,
                LobbyType::Invisible => sys::LobbyType::Invisible,
            };
            let api_call = sys::SteamAPI_ISteamMatchmaking_CreateLobby(self.mm, ty, max_members as _);
            register_call_result::<sys::LobbyCreated, _, _>(
                &self.inner, api_call, CALLBACK_BASE_ID + 13,
                move |v, io_error| {
                    cb(if io_error {
                        Err(SteamError::IOFailure)
                    } else if v.result != sys::SResult::Ok {
                        Err(v.result.into())
                    } else {
                        Ok(LobbyId(v.lobby_steam_id))
                    })
            });
        }
    }
}

#[test]
fn test_lobby() {
    let client = Client::init().unwrap();
    let mm = client.matchmaking();

    mm.request_lobby_list(|v| {
        println!("List: {:?}", v);
    });
    mm.create_lobby(LobbyType::Private, 4, |v| {
        println!("Create: {:?}", v);
    });

    for _ in 0 .. 100 {
        client.run_callbacks();
        ::std::thread::sleep(::std::time::Duration::from_millis(100));
    }
}