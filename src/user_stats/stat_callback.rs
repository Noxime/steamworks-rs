use super::*;

/// Callback type after calling
/// [`request_current_stats()`](struct.UserStats.html#method.request_current_stats).
///
/// # Example
///
/// ```no_run
/// # use steamworks::*;
/// # let (client, single) = steamworks::Client::init().unwrap();
/// let callback_handle = client.register_callback(|val: UserStatsReceived| {
///     if val.result.is_err() {
///         // ...
///     }
/// });
/// ```
#[derive(Clone, Debug)]
pub struct UserStatsReceived {
    pub steam_id: SteamId,
    pub game_id: GameId,
    pub result: Result<(), SteamError>,
}

unsafe impl Callback for UserStatsReceived {
    const ID: i32 = CALLBACK_BASE_ID + 1;
    const SIZE: i32 = std::mem::size_of::<sys::UserStatsReceived_t>() as i32;

    unsafe fn from_raw(raw: *mut c_void) -> Self {
        let val = &mut *(raw as *mut sys::UserStatsReceived_t);
        Self {
            steam_id: SteamId(val.m_steamIDUser.m_steamid.m_unAll64Bits),
            game_id: GameId(val.m_nGameID),
            result: match val.m_eResult {
                sys::EResult::k_EResultOK => Ok(()),
                err => Err(err.into()),
            },
        }
    }
}

/// Callback triggered by [`store()`](stats/struct.StatsHelper.html#method.store).
///
/// # Example
///
/// ```no_run
/// # use steamworks::*;
/// # let (client, single) = steamworks::Client::init().unwrap();
/// let callback_handle = client.register_callback(|val: UserStatsStored| {
///     if val.result.is_err() {
///         // ...
///     }
/// });
/// ```
#[derive(Clone, Debug)]
pub struct UserStatsStored {
    pub game_id: GameId,
    pub result: Result<(), SteamError>,
}

unsafe impl Callback for UserStatsStored {
    const ID: i32 = CALLBACK_BASE_ID + 2;
    const SIZE: i32 = std::mem::size_of::<sys::UserStatsStored_t>() as i32;

    unsafe fn from_raw(raw: *mut c_void) -> Self {
        let val = &mut *(raw as *mut sys::UserStatsStored_t);
        Self {
            game_id: GameId(val.m_nGameID),
            result: match val.m_eResult {
                sys::EResult::k_EResultOK => Ok(()),
                err => Err(err.into()),
            },
        }
    }
}

/// Result of a request to store the achievements on the server, or an "indicate progress" call.
/// If both `current_progress` and `max_progress` are zero, that means the achievement has been
/// fully unlocked.
///
/// # Example
///
/// ```no_run
/// # use steamworks::*;
/// # let (client, single) = steamworks::Client::init().unwrap();
/// let callback_handle = client.register_callback(|val: UserAchievementStored| {
///     // ...
/// });
/// ```
#[derive(Clone, Debug)]
pub struct UserAchievementStored {
    pub game_id: GameId,
    pub achievement_name: String,
    /// Current progress towards the achievement.
    pub current_progress: u32,
    /// The total amount of progress required to unlock.
    pub max_progress: u32,
}

unsafe impl Callback for UserAchievementStored {
    const ID: i32 = CALLBACK_BASE_ID + 3;
    const SIZE: i32 = std::mem::size_of::<sys::UserAchievementStored_t>() as i32;

    unsafe fn from_raw(raw: *mut c_void) -> Self {
        let val = &mut *(raw as *mut sys::UserAchievementStored_t);
        let name = CStr::from_ptr(val.m_rgchAchievementName.as_ptr()).to_owned();
        Self {
            game_id: GameId(val.m_nGameID),
            achievement_name: name.into_string().unwrap(),
            current_progress: val.m_nCurProgress,
            max_progress: val.m_nMaxProgress,
        }
    }
}

/// Result of a request to retrieve the achievement icon if the icon was not available at the time of the function call.
/// # Example
///
/// ```no_run
/// # use steamworks::*;
/// # let (client, single) = steamworks::Client::init().unwrap();
/// let callback_handle = client.register_callback(|val: UserAchievementIconFetched| {
///     // ...
/// });
/// ```
#[derive(Clone, Debug)]
pub struct UserAchievementIconFetched {
    pub game_id: GameId,
    pub achievement_name: String,
    pub achieved: bool,
    pub icon_handle: i32,
}

unsafe impl Callback for UserAchievementIconFetched {
    const ID: i32 = CALLBACK_BASE_ID + 9;
    const SIZE: i32 = std::mem::size_of::<sys::UserAchievementStored_t>() as i32;

    unsafe fn from_raw(raw: *mut c_void) -> Self {
        let val = &mut *(raw as *mut sys::UserAchievementIconFetched_t);
        let name = CStr::from_ptr(val.m_rgchAchievementName.as_ptr()).to_owned();
        Self {
            game_id: GameId(val.m_nGameID.__bindgen_anon_1.m_ulGameID),
            achievement_name: name.into_string().unwrap(),
            achieved: val.m_bAchieved,
            icon_handle: val.m_nIconHandle,
        }
    }
}
