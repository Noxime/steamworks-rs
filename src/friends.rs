use super::*;
use std::net::Ipv4Addr;

const CALLBACK_BASE_ID: i32 = 300;

bitflags! {
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

bitflags! {
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[repr(C)]
    /// see [Steam API](https://partner.steamgames.com/doc/api/ISteamFriends#EUserRestriction)
    pub struct UserRestriction: u32 {
        /// No known chat/content restriction.
        const NONE                  = 0x0000;
        /// We don't know yet, the user is offline.
        const UNKNOWN               = 0x0001;
        /// User is not allowed to (or can't) send/recv any chat.
        const ANY_CHAT              = 0x0002;
        /// User is not allowed to (or can't) send/recv voice chat.
        const VOICE_CHAT            = 0x0004;
        /// User is not allowed to (or can't) send/recv group chat.
        const GROUP_CHAT            = 0x0008;
        /// User is too young according to rating in current region.
        const RESTRICTION_RATING    = 0x0010;
        /// User cannot send or recv game invites, for example if they are on mobile.
        const GAME_INVITES          = 0x0020;
        /// User cannot participate in trading, for example if they are on a console or mobile.
        const TRADING               = 0x0040;
    }
}

pub enum OverlayToStoreFlag {
    None = 0,
    AddToCart = 1,
    AddToCartAndShow = 2,
}

/// Access to the steam friends interface
pub struct Friends<Manager> {
    pub(crate) friends: *mut sys::ISteamFriends,
    pub(crate) inner: Arc<Inner<Manager>>,
}

impl<Manager> Friends<Manager> {
    /// Returns the (display) name of the current user
    pub fn name(&self) -> String {
        unsafe {
            let name = sys::SteamAPI_ISteamFriends_GetPersonaName(self.friends);
            let name = CStr::from_ptr(name);
            name.to_string_lossy().into_owned()
        }
    }

    /// Returns a small (32x32) avatar for the current user in RGBA format
    pub fn small_avatar(&self) -> Option<Vec<u8>> {
        unsafe {
            let user: *mut sys::ISteamUser = sys::SteamAPI_SteamUser_v023();
            let steam_id = sys::SteamAPI_ISteamUser_GetSteamID(user);
            let utils = sys::SteamAPI_SteamUtils_v010();
            let img = sys::SteamAPI_ISteamFriends_GetSmallFriendAvatar(self.friends, steam_id);
            if img == 0 {
                return None;
            }
            let mut width = 0;
            let mut height = 0;
            if !sys::SteamAPI_ISteamUtils_GetImageSize(utils, img, &mut width, &mut height) {
                return None;
            }
            assert_eq!(width, 32);
            assert_eq!(height, 32);
            let mut dest = vec![0; 32 * 32 * 4];
            if !sys::SteamAPI_ISteamUtils_GetImageRGBA(utils, img, dest.as_mut_ptr(), 32 * 32 * 4) {
                return None;
            }
            Some(dest)
        }
    }

    /// Returns a small (64x64) avatar for the current user in RGBA format
    pub fn medium_avatar(&self) -> Option<Vec<u8>> {
        unsafe {
            let user: *mut sys::ISteamUser = sys::SteamAPI_SteamUser_v023();
            let steam_id = sys::SteamAPI_ISteamUser_GetSteamID(user);
            let utils = sys::SteamAPI_SteamUtils_v010();
            let img = sys::SteamAPI_ISteamFriends_GetMediumFriendAvatar(self.friends, steam_id);
            if img == 0 {
                return None;
            }
            let mut width = 0;
            let mut height = 0;
            if !sys::SteamAPI_ISteamUtils_GetImageSize(utils, img, &mut width, &mut height) {
                return None;
            }
            assert_eq!(width, 64);
            assert_eq!(height, 64);
            let mut dest = vec![0; 64 * 64 * 4];
            if !sys::SteamAPI_ISteamUtils_GetImageRGBA(utils, img, dest.as_mut_ptr(), 64 * 64 * 4) {
                return None;
            }
            Some(dest)
        }
    }

    /// Returns a small (184x184) avatar for the current user in RGBA format
    pub fn large_avatar(&self) -> Option<Vec<u8>> {
        unsafe {
            let user: *mut sys::ISteamUser = sys::SteamAPI_SteamUser_v023();
            let steam_id = sys::SteamAPI_ISteamUser_GetSteamID(user);
            let utils = sys::SteamAPI_SteamUtils_v010();
            let img = sys::SteamAPI_ISteamFriends_GetLargeFriendAvatar(self.friends, steam_id);
            if img == 0 {
                return None;
            }
            let mut width = 0;
            let mut height = 0;
            if !sys::SteamAPI_ISteamUtils_GetImageSize(utils, img, &mut width, &mut height) {
                return None;
            }
            assert_eq!(width, 184);
            assert_eq!(height, 184);
            let mut dest = vec![0; 184 * 184 * 4];
            if !sys::SteamAPI_ISteamUtils_GetImageRGBA(utils, img, dest.as_mut_ptr(), 184 * 184 * 4)
            {
                return None;
            }
            Some(dest)
        }
    }

    pub fn get_friends(&self, flags: FriendFlags) -> Vec<Friend<Manager>> {
        unsafe {
            let count = sys::SteamAPI_ISteamFriends_GetFriendCount(self.friends, flags.bits() as _);
            if count == -1 {
                return Vec::new();
            }
            let mut friends = Vec::with_capacity(count as usize);
            for idx in 0..count {
                let friend = SteamId(sys::SteamAPI_ISteamFriends_GetFriendByIndex(
                    self.friends,
                    idx,
                    flags.bits() as _,
                ));
                friends.push(self.get_friend(friend));
            }

            friends
        }
    }

    /// Get a list of users from a specific source (e.g. gameserver)
    /// Despite the misleading name, it does not return only friends
    /// but all users from the source that are currently connected.
    /// This does however require some local client to be connected to the source.
    pub fn get_friends_from_source(&self, source: SteamId) -> Vec<Friend<Manager>> {
        unsafe {
            let count =
                sys::SteamAPI_ISteamFriends_GetFriendCountFromSource(self.friends, source.0);
            if count == -1 {
                return Vec::new();
            }
            let mut friends = Vec::with_capacity(count as usize);
            for idx in 0..count {
                let friend = SteamId(sys::SteamAPI_ISteamFriends_GetFriendFromSourceByIndex(
                    self.friends,
                    source.0,
                    idx,
                ));
                friends.push(self.get_friend(friend));
            }
            friends
        }
    }

    /// Returns recently played with players list
    pub fn get_coplay_friends(&self) -> Vec<Friend<Manager>> {
        unsafe {
            let count = sys::SteamAPI_ISteamFriends_GetCoplayFriendCount(self.friends);
            if count == -1 {
                return Vec::new();
            }
            let mut friends = Vec::with_capacity(count as usize);
            for idx in 0..count {
                let friend = SteamId(sys::SteamAPI_ISteamFriends_GetCoplayFriend(
                    self.friends,
                    idx,
                ));
                friends.push(self.get_friend(friend));
            }
            friends
        }
    }

    pub fn get_friend(&self, friend: SteamId) -> Friend<Manager> {
        Friend {
            id: friend,
            friends: self.friends,
            _inner: self.inner.clone(),
        }
    }

    pub fn request_user_information(&self, user: SteamId, name_only: bool) -> bool {
        unsafe {
            sys::SteamAPI_ISteamFriends_RequestUserInformation(self.friends, user.0, name_only)
        }
    }

    pub fn activate_game_overlay(&self, dialog: &str) {
        let dialog = CString::new(dialog).unwrap();
        unsafe {
            sys::SteamAPI_ISteamFriends_ActivateGameOverlay(
                self.friends,
                dialog.as_ptr() as *const _,
            );
        }
    }

    // I don't know why these are part of friends either
    pub fn activate_game_overlay_to_web_page(&self, url: &str) {
        unsafe {
            let url = CString::new(url).unwrap();
            sys::SteamAPI_ISteamFriends_ActivateGameOverlayToWebPage(
                self.friends,
                url.as_ptr() as *const _,
                sys::EActivateGameOverlayToWebPageMode::k_EActivateGameOverlayToWebPageMode_Default,
            );
        }
    }

    pub fn activate_game_overlay_to_store(
        &self,
        app_id: AppId,
        overlay_to_store_flag: OverlayToStoreFlag,
    ) {
        unsafe {
            let overlay_to_store_flag = match overlay_to_store_flag {
                OverlayToStoreFlag::None => sys::EOverlayToStoreFlag::k_EOverlayToStoreFlag_None,
                OverlayToStoreFlag::AddToCart => {
                    sys::EOverlayToStoreFlag::k_EOverlayToStoreFlag_AddToCart
                }
                OverlayToStoreFlag::AddToCartAndShow => {
                    sys::EOverlayToStoreFlag::k_EOverlayToStoreFlag_AddToCartAndShow
                }
            };
            sys::SteamAPI_ISteamFriends_ActivateGameOverlayToStore(
                self.friends,
                app_id.0,
                overlay_to_store_flag,
            );
        }
    }

    pub fn activate_game_overlay_to_user(&self, dialog: &str, user: SteamId) {
        let dialog = CString::new(dialog).unwrap();
        unsafe {
            sys::SteamAPI_ISteamFriends_ActivateGameOverlayToUser(
                self.friends,
                dialog.as_ptr() as *const _,
                user.0,
            );
        }
    }

    /// Opens up an invite dialog for the given lobby
    pub fn activate_invite_dialog(&self, lobby: LobbyId) {
        unsafe {
            sys::SteamAPI_ISteamFriends_ActivateGameOverlayInviteDialog(self.friends, lobby.0);
        }
    }

    /// Set rich presence for the user. Unsets the rich presence if `value` is None or empty.
    /// See [Steam API](https://partner.steamgames.com/doc/api/ISteamFriends#SetRichPresence)
    pub fn set_rich_presence(&self, key: &str, value: Option<&str>) -> bool {
        unsafe {
            // Unwraps are infallible because Rust strs cannot contain null bytes
            let key = CString::new(key).unwrap();
            let value = CString::new(value.unwrap_or_default()).unwrap();
            sys::SteamAPI_ISteamFriends_SetRichPresence(
                self.friends,
                key.as_ptr() as *const _,
                value.as_ptr() as *const _,
            )
        }
    }
    /// Clears all of the current user's Rich Presence key/values.
    pub fn clear_rich_presence(&self) {
        unsafe {
            sys::SteamAPI_ISteamFriends_ClearRichPresence(self.friends);
        }
    }

    /// Checks if current user is chat restricted.
    ///
    /// If they are restricted, then they can't send or receive any text/voice chat messages, can't see custom avatars.
    /// A chat restricted user can't add friends or join any groups.
    /// Restricted users can still be online and send/receive game invites.
    pub fn get_user_restrictions(&self) -> UserRestriction {
        unsafe {
            let restrictions = sys::SteamAPI_ISteamFriends_GetUserRestrictions(self.friends);
            UserRestriction::from_bits_truncate(restrictions)
        }
    }
}

/// Information about a friend's current state in a game
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FriendGame {
    /// The id of the game that the friend is
    /// playing
    pub game: GameId,
    /// The address of the server the player is in
    pub game_address: Ipv4Addr,
    /// The game port of the server the player is in
    pub game_port: u16,
    /// The query port of the server the player is in
    pub query_port: u16,
    /// Optional id of the lobby the player is in
    pub lobby: LobbyId,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PersonaStateChange {
    pub steam_id: SteamId,
    pub flags: PersonaChange,
}

unsafe impl Callback for PersonaStateChange {
    const ID: i32 = CALLBACK_BASE_ID + 4;
    const SIZE: i32 = ::std::mem::size_of::<sys::PersonaStateChange_t>() as i32;

    unsafe fn from_raw(raw: *mut c_void) -> Self {
        let val = &mut *(raw as *mut sys::PersonaStateChange_t);
        PersonaStateChange {
            steam_id: SteamId(val.m_ulSteamID),
            flags: PersonaChange::from_bits_truncate(val.m_nChangeFlags as i32),
        }
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GameOverlayActivated {
    pub active: bool,
}

unsafe impl Callback for GameOverlayActivated {
    const ID: i32 = CALLBACK_BASE_ID + 31;
    const SIZE: i32 = std::mem::size_of::<sys::GameOverlayActivated_t>() as i32;

    unsafe fn from_raw(raw: *mut c_void) -> Self {
        let val = &mut *(raw as *mut sys::GameOverlayActivated_t);
        Self {
            active: val.m_bActive == 1,
        }
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GameLobbyJoinRequested {
    pub lobby_steam_id: LobbyId,
    pub friend_steam_id: SteamId,
}

unsafe impl Callback for GameLobbyJoinRequested {
    const ID: i32 = CALLBACK_BASE_ID + 33;
    const SIZE: i32 = ::std::mem::size_of::<sys::GameLobbyJoinRequested_t>() as i32;

    unsafe fn from_raw(raw: *mut c_void) -> Self {
        let val = &mut *(raw as *mut sys::GameLobbyJoinRequested_t);
        GameLobbyJoinRequested {
            lobby_steam_id: LobbyId(val.m_steamIDLobby.m_steamid.m_unAll64Bits),
            friend_steam_id: SteamId(val.m_steamIDFriend.m_steamid.m_unAll64Bits),
        }
    }
}

pub struct Friend<Manager> {
    id: SteamId,
    friends: *mut sys::ISteamFriends,
    _inner: Arc<Inner<Manager>>,
}

impl<Manager> Debug for Friend<Manager> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Friend({:?})", self.id)
    }
}

impl<Manager> Friend<Manager> {
    pub fn id(&self) -> SteamId {
        self.id
    }

    pub fn name(&self) -> String {
        unsafe {
            let name = sys::SteamAPI_ISteamFriends_GetFriendPersonaName(self.friends, self.id.0);
            let name = CStr::from_ptr(name);
            name.to_string_lossy().into_owned()
        }
    }
    /// Gets the nickname that the current user has set for the specified user.
    pub fn nick_name(&self) -> Option<String> {
        unsafe {
            let name = sys::SteamAPI_ISteamFriends_GetPlayerNickname(self.friends, self.id.0);
            if name.is_null() {
                return None;
            }

            let name = CStr::from_ptr(name);
            if name.is_empty() {
                None
            } else {
                Some(name.to_string_lossy().into_owned())
            }
        }
    }

    pub fn state(&self) -> FriendState {
        unsafe {
            let state = sys::SteamAPI_ISteamFriends_GetFriendPersonaState(self.friends, self.id.0);
            match state {
                sys::EPersonaState::k_EPersonaStateOffline => FriendState::Offline,
                sys::EPersonaState::k_EPersonaStateOnline => FriendState::Online,
                sys::EPersonaState::k_EPersonaStateBusy => FriendState::Busy,
                sys::EPersonaState::k_EPersonaStateAway => FriendState::Away,
                sys::EPersonaState::k_EPersonaStateSnooze => FriendState::Snooze,
                sys::EPersonaState::k_EPersonaStateLookingToPlay => FriendState::LookingToPlay,
                sys::EPersonaState::k_EPersonaStateLookingToTrade => FriendState::LookingToTrade,
                _ => unreachable!(),
            }
        }
    }

    /// Returns information about the game the player is current playing if any
    pub fn game_played(&self) -> Option<FriendGame> {
        unsafe {
            let mut info: sys::FriendGameInfo_t = std::mem::zeroed();
            if sys::SteamAPI_ISteamFriends_GetFriendGamePlayed(self.friends, self.id.0, &mut info) {
                Some(FriendGame {
                    game: GameId(std::mem::transmute(info.m_gameID)),
                    game_address: info.m_unGameIP.into(),
                    game_port: info.m_usGamePort,
                    query_port: info.m_usQueryPort,
                    lobby: LobbyId(info.m_steamIDLobby.m_steamid.m_unAll64Bits),
                })
            } else {
                None
            }
        }
    }
    /// Gets the app ID of the game that user played with someone on their recently-played-with list.
    pub fn coplay_game_played(&self) -> AppId {
        unsafe {
            let app_id = sys::SteamAPI_ISteamFriends_GetFriendCoplayGame(self.friends, self.id.0);
            AppId(app_id)
        }
    }

    /// Gets the timestamp of when the user played with someone on their recently-played-with list.
    pub fn coplay_time(&self) -> i32 {
        unsafe { sys::SteamAPI_ISteamFriends_GetFriendCoplayTime(self.friends, self.id.0) }
    }

    /// Returns a small (32x32) avatar for the user in RGBA format
    pub fn small_avatar(&self) -> Option<Vec<u8>> {
        unsafe {
            let utils = sys::SteamAPI_SteamUtils_v010();
            let img = sys::SteamAPI_ISteamFriends_GetSmallFriendAvatar(self.friends, self.id.0);
            if img == 0 {
                return None;
            }
            let mut width = 0;
            let mut height = 0;
            if !sys::SteamAPI_ISteamUtils_GetImageSize(utils, img, &mut width, &mut height) {
                return None;
            }
            assert_eq!(width, 32);
            assert_eq!(height, 32);
            let mut dest = vec![0; 32 * 32 * 4];
            if !sys::SteamAPI_ISteamUtils_GetImageRGBA(utils, img, dest.as_mut_ptr(), 32 * 32 * 4) {
                return None;
            }
            Some(dest)
        }
    }

    /// Returns a medium (64x64) avatar for the user in RGBA format
    pub fn medium_avatar(&self) -> Option<Vec<u8>> {
        unsafe {
            let utils = sys::SteamAPI_SteamUtils_v010();
            let img = sys::SteamAPI_ISteamFriends_GetMediumFriendAvatar(self.friends, self.id.0);
            if img == 0 {
                return None;
            }
            let mut width = 0;
            let mut height = 0;
            if !sys::SteamAPI_ISteamUtils_GetImageSize(utils, img, &mut width, &mut height) {
                return None;
            }
            assert_eq!(width, 64);
            assert_eq!(height, 64);
            let mut dest = vec![0; 64 * 64 * 4];
            if !sys::SteamAPI_ISteamUtils_GetImageRGBA(utils, img, dest.as_mut_ptr(), 64 * 64 * 4) {
                return None;
            }
            Some(dest)
        }
    }

    /// Returns a large (184x184) avatar for the user in RGBA format
    pub fn large_avatar(&self) -> Option<Vec<u8>> {
        unsafe {
            let utils = sys::SteamAPI_SteamUtils_v010();
            let img = sys::SteamAPI_ISteamFriends_GetLargeFriendAvatar(self.friends, self.id.0);
            if img == 0 {
                return None;
            }
            let mut width = 0;
            let mut height = 0;
            if !sys::SteamAPI_ISteamUtils_GetImageSize(utils, img, &mut width, &mut height) {
                return None;
            }
            assert_eq!(width, 184);
            assert_eq!(height, 184);
            let mut dest = vec![0; 184 * 184 * 4];
            if !sys::SteamAPI_ISteamUtils_GetImageRGBA(utils, img, dest.as_mut_ptr(), 184 * 184 * 4)
            {
                return None;
            }
            Some(dest)
        }
    }

    /// Checks if the user meets the specified criteria. (Friends, blocked, users on the same server, etc)
    pub fn has_friend(&self, flags: FriendFlags) -> bool {
        unsafe { sys::SteamAPI_ISteamFriends_HasFriend(self.friends, self.id.0, flags.bits() as _) }
    }

    /// Invites a friend or clan member to the current game using a special invite string.
    /// If the target user accepts the invite then the ConnectString gets added to the command-line when launching the game.
    /// If the game is already running for that user, then they will receive a GameRichPresenceJoinRequested_t callback with the connect string.
    pub fn invite_user_to_game(&self, connect_string: &str) {
        unsafe {
            let connect_string = CString::new(connect_string).unwrap();
            sys::SteamAPI_ISteamFriends_InviteUserToGame(
                self.friends,
                self.id.0,
                connect_string.as_ptr() as *const _,
            );
        }
    }

    /// Mark a target user as 'played with'.
    /// NOTE: The current user must be in game with the other player for the association to work.
    pub fn set_played_with(&self) {
        unsafe {
            sys::SteamAPI_ISteamFriends_SetPlayedWith(self.friends, self.id.0);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FriendState {
    Offline,
    Online,
    Busy,
    Away,
    Snooze,
    LookingToTrade,
    LookingToPlay,
}
