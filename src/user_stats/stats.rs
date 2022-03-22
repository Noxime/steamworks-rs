use super::*;

/// Achievement API.
///
/// Methods require
/// [`request_current_stats()`](../struct.UserStats.html#method.request_current_stats)
/// to have been called and a successful [`UserStatsReceived`](../struct.UserStatsReceived.html)
/// callback processed.
///
/// # Example
///
/// ```no_run
/// # use steamworks::*;
/// # let (client, single) = steamworks::Client::init().unwrap();
/// // Unlock the 'WIN_THE_GAME' achievement
/// client.user_stats().achievement("WIN_THE_GAME").set()?;
/// # Err(())
/// ```
pub struct AchievementHelper<'parent, M> {
    pub(crate) name: CString,
    pub(crate) parent: &'parent UserStats<M>,
}

impl<M> AchievementHelper<'_, M> {
    /// Gets the unlock status of the Achievement.
    ///
    /// This call only modifies Steam's in-memory state so it is quite cheap. To send the unlock
    /// status to the server and to trigger the Steam overlay notification you must call
    /// [`store_stats()`](../struct.UserStats.html#method.store_stats).
    ///
    /// Fails if this achievement's 'API Name' is unknown, or unsuccessful
    /// [`UserStatsReceived`](../struct.UserStatsReceived.html).
    pub fn get(&self) -> Result<bool, ()> {
        unsafe {
            let mut achieved = false;
            let success = sys::SteamAPI_ISteamUserStats_GetAchievement(
                self.parent.user_stats,
                self.name.as_ptr() as *const _,
                &mut achieved as *mut _,
            );
            if success {
                Ok(achieved)
            } else {
                Err(())
            }
        }
    }

    /// Unlocks an achievement.
    ///
    /// This call only modifies Steam's in-memory state so it is quite cheap. To send the unlock
    /// status to the server and to trigger the Steam overlay notification you must call
    /// [`store_stats()`](../struct.UserStats.html#method.store_stats).
    ///
    /// Fails if this achievement's 'API Name' is unknown, or unsuccessful
    /// [`UserStatsReceived`](../struct.UserStatsReceived.html).
    pub fn set(&self) -> Result<(), ()> {
        let success = unsafe {
            sys::SteamAPI_ISteamUserStats_SetAchievement(
                self.parent.user_stats,
                self.name.as_ptr() as *const _,
            )
        };
        if success {
            Ok(())
        } else {
            Err(())
        }
    }

    /// Resets the unlock status of an achievement.
    ///
    /// This call only modifies Steam's in-memory state so it is quite cheap. To send the unlock
    /// status to the server and to trigger the Steam overlay notification you must call
    /// [`store_stats()`](../struct.UserStats.html#method.store_stats).
    ///
    /// Fails if this achievement's 'API Name' is unknown, or unsuccessful
    /// [`UserStatsReceived`](../struct.UserStatsReceived.html).
    pub fn clear(&self) -> Result<(), ()> {
        let success = unsafe {
            sys::SteamAPI_ISteamUserStats_ClearAchievement(
                self.parent.user_stats,
                self.name.as_ptr() as *const _,
            )
        };
        if success {
            Ok(())
        } else {
            Err(())
        }
    }
}
