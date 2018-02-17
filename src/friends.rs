
use super::*;

bitflags! {
    #[repr(C)]
    pub struct FriendFlags: u16 {
        const NONE                  = 0x0000;
        const BLOCKED               = 0x0001;
        const FRIENDSHIP_REQUESTED  = 0x0002;
        const IMMEDIATE             = 0x0004;
        const CLAN_MEMBER           = 0x0008;
        const ON_GAME_SERVER        = 0x0010;
        // Unused
        // Unused
        const REQUESTING_FRIENDSHIP = 0x0080;
        const REQUESTING_INFO       = 0x0100;
        const IGNORED               = 0x0200;
        const IGNORED_FRIEND        = 0x0400;
        // Unused
        const CHAT_MEMBER           = 0x1000;
        const ALL                   = 0xFFFF;
    }
}

bitflags! {
    #[repr(C)]
    pub struct PersonaChange: i32 {
        const NAME                = 0x0001;
        const STATUS              = 0x0002;
        const COME_ONLINE         = 0x0004;
        const GONE_OFFLINE        = 0x0008;
        const GAME_PLAYED         = 0x0010;
        const GAME_SERVER         = 0x0020;
        const AVATAR              = 0x0040;
        const JOINED_SOURCE       = 0x0080;
        const LEFT_SOURCE         = 0x0100;
        const RELATIONSHIP_CHANGE = 0x0200;
        const NAME_FIRST_SET      = 0x0400;
        const FACEBOOK_INFO       = 0x0800;
        const NICKNAME            = 0x1000;
        const STEAM_LEVEL         = 0x2000;
    }
}
pub struct Friends {
    pub(crate) friends: *mut sys::ISteamFriends,
    pub(crate) _client: Arc<ClientInner>,
}

impl Friends {
    pub fn get_friends(&self, flags: FriendFlags) -> Vec<Friend> {
        unsafe {
            let count = sys::SteamAPI_ISteamFriends_GetFriendCount(self.friends, flags.bits() as _);
            let mut friends = Vec::with_capacity(count as usize);
            for idx in 0 .. count {
                let friend = SteamId(sys::SteamAPI_ISteamFriends_GetFriendByIndex(self.friends, idx, flags.bits() as _));
                friends.push(Friend {
                    id: friend,
                    friends: self.friends,
                    _client: self._client.clone(),
                });
            }

            friends
        }
    }

    pub fn request_user_information(&self, user: SteamId, name_only: bool) {
        unsafe {
            sys::SteamAPI_ISteamFriends_RequestUserInformation(self.friends, user.0, name_only as u8);
        }
    }

    // I don't know why this is part of friends either
    pub fn activate_game_overlay_to_web_page(&self, url: &str) {
        unsafe {
            let url = CString::new(url).unwrap();
            sys::SteamAPI_ISteamFriends_ActivateGameOverlayToWebPage(self.friends, url.as_ptr() as *const _);
        }
    }
}

#[derive(Debug)]
pub struct PersonaStateChange {
    pub steam_id: SteamId,
    pub flags: PersonaChange,
}

unsafe impl Callback for PersonaStateChange {
    fn id() -> i32 {
        304
    }
    fn size() -> i32 {
        ::std::mem::size_of::<sys::PersonaStateChange_t>() as i32
    }

    unsafe fn from_raw(raw: *mut libc::c_void) -> Self {
        let val = &mut *(raw as *mut sys::PersonaStateChange_t);
        PersonaStateChange {
            steam_id: SteamId(val.steam_id),
            flags: PersonaChange::from_bits_truncate(val.flags as i32),
        }
    }
}

pub struct Friend {
    id: SteamId,
    friends: *mut sys::ISteamFriends,
    _client: Arc<ClientInner>,
}

impl Debug for Friend {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Friend({:?})", self.id)
    }
}

impl Friend {
    pub fn id(&self) -> SteamId {
        self.id
    }

    pub fn name(&self) -> Cow<str> {
        unsafe {
            let name = sys::SteamAPI_ISteamFriends_GetFriendPersonaName(self.friends, self.id.0);
            let name = CStr::from_ptr(name);
            name.to_string_lossy()
        }
    }

    pub fn state(&self) -> FriendState {
        unsafe {
            let state = sys::SteamAPI_ISteamFriends_GetFriendPersonaState(self.friends, self.id.0);
            match state {
                sys::PersonaState::Offline => FriendState::Offline,
                sys::PersonaState::Online => FriendState::Online,
                sys::PersonaState::Busy => FriendState::Busy,
                sys::PersonaState::Away => FriendState::Away,
                sys::PersonaState::Snooze => FriendState::Snooze,
                sys::PersonaState::LookingToPlay => FriendState::LookingToPlay,
                sys::PersonaState::LookingToTrade => FriendState::LookingToTrade,
                sys::PersonaState::Max => unreachable!(),
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum FriendState {
    Offline,
    Online,
    Busy,
    Away,
    Snooze,
    LookingToTrade,
    LookingToPlay,
}