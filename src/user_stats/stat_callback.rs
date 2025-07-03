use super::*;

/// Callback type after calling
/// [`request_current_stats()`](struct.UserStats.html#method.request_current_stats).
///
/// # Example
///
/// ```no_run
/// # use steamworks::*;
/// # let client = steamworks::Client::init().unwrap();
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

impl_callback!(cb: UserStatsReceived_t => UserStatsReceived {
    Self {
        steam_id: SteamId(cb.m_steamIDUser.m_steamid.m_unAll64Bits),
        game_id: GameId(cb.m_nGameID),
        result: crate::to_steam_result(cb.m_eResult),
    }
});

/// Callback triggered by [`store()`](stats/struct.StatsHelper.html#method.store).
///
/// # Example
///
/// ```no_run
/// # use steamworks::*;
/// # let client = steamworks::Client::init().unwrap();
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

impl_callback!(cb: UserStatsStored_t => UserStatsStored {
    Self {
        game_id: GameId(cb.m_nGameID),
        result: crate::to_steam_result(cb.m_eResult),
    }
});

/// Result of a request to store the achievements on the server, or an "indicate progress" call.
/// If both `current_progress` and `max_progress` are zero, that means the achievement has been
/// fully unlocked.
///
/// # Example
///
/// ```no_run
/// # use steamworks::*;
/// # let client = steamworks::Client::init().unwrap();
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

impl_callback!(cb: UserAchievementStored_t => UserAchievementStored {
    let name = CStr::from_ptr(cb.m_rgchAchievementName.as_ptr()).to_owned();
    Self {
        game_id: GameId(cb.m_nGameID),
        achievement_name: name.into_string().unwrap(),
        current_progress: cb.m_nCurProgress,
        max_progress: cb.m_nMaxProgress,
    }
});

/// Result of a request to retrieve the achievement icon if the icon was not available at the time of the function call.
/// # Example
///
/// ```no_run
/// # use steamworks::*;
/// # let client = steamworks::Client::init().unwrap();
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

impl_callback!(cb: UserAchievementIconFetched_t => UserAchievementIconFetched {
    let name = CStr::from_ptr(cb.m_rgchAchievementName.as_ptr()).to_owned();
    Self {
        game_id: GameId(cb.m_nGameID.__bindgen_anon_1.m_ulGameID),
        achievement_name: name.into_string().unwrap(),
        achieved: cb.m_bAchieved,
        icon_handle: cb.m_nIconHandle,
    }
});
