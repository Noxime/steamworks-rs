mod stat_callback;
pub mod stats;

pub use self::stat_callback::*;
use super::*;
#[cfg(test)]
use serial_test::serial;

/// Access to the steam user interface
pub struct UserStats {
    pub(crate) user_stats: *mut sys::ISteamUserStats,
    pub(crate) inner: Arc<Inner>,
}

impl UserStats {
    pub fn find_leaderboard<F>(&self, name: &str, cb: F)
    where
        F: FnOnce(Result<Option<Leaderboard>, SteamError>) + 'static + Send,
    {
        unsafe {
            let name = CString::new(name).unwrap();
            let api_call =
                sys::SteamAPI_ISteamUserStats_FindLeaderboard(self.user_stats, name.as_ptr());
            register_call_result::<sys::LeaderboardFindResult_t, _>(
                &self.inner,
                api_call,
                move |v, io_error| {
                    cb(if io_error {
                        Err(SteamError::IOFailure)
                    } else {
                        Ok(if v.m_bLeaderboardFound != 0 {
                            Some(Leaderboard(v.m_hSteamLeaderboard))
                        } else {
                            None
                        })
                    })
                },
            );
        }
    }

    pub fn find_or_create_leaderboard<F>(
        &self,
        name: &str,
        sort_method: LeaderboardSortMethod,
        display_type: LeaderboardDisplayType,
        cb: F,
    ) where
        F: FnOnce(Result<Option<Leaderboard>, SteamError>) + 'static + Send,
    {
        unsafe {
            let name = CString::new(name).unwrap();

            let sort_method = match sort_method {
                LeaderboardSortMethod::Ascending => {
                    sys::ELeaderboardSortMethod::k_ELeaderboardSortMethodAscending
                }
                LeaderboardSortMethod::Descending => {
                    sys::ELeaderboardSortMethod::k_ELeaderboardSortMethodDescending
                }
            };

            let display_type = match display_type {
                LeaderboardDisplayType::Numeric => {
                    sys::ELeaderboardDisplayType::k_ELeaderboardDisplayTypeNumeric
                }
                LeaderboardDisplayType::TimeSeconds => {
                    sys::ELeaderboardDisplayType::k_ELeaderboardDisplayTypeTimeSeconds
                }
                LeaderboardDisplayType::TimeMilliSeconds => {
                    sys::ELeaderboardDisplayType::k_ELeaderboardDisplayTypeTimeMilliSeconds
                }
            };

            let api_call = sys::SteamAPI_ISteamUserStats_FindOrCreateLeaderboard(
                self.user_stats,
                name.as_ptr(),
                sort_method,
                display_type,
            );
            register_call_result::<sys::LeaderboardFindResult_t, _>(
                &self.inner,
                api_call,
                move |v, io_error| {
                    cb(if io_error {
                        Err(SteamError::IOFailure)
                    } else {
                        Ok(if v.m_bLeaderboardFound != 0 {
                            Some(Leaderboard(v.m_hSteamLeaderboard))
                        } else {
                            None
                        })
                    })
                },
            );
        }
    }

    pub fn upload_leaderboard_score<F>(
        &self,
        leaderboard: &Leaderboard,
        method: UploadScoreMethod,
        score: i32,
        details: &[i32],
        cb: F,
    ) where
        F: FnOnce(Result<Option<LeaderboardScoreUploaded>, SteamError>) + 'static + Send,
    {
        unsafe {
            let method = match method {
                UploadScoreMethod::KeepBest => {
                    sys::ELeaderboardUploadScoreMethod::k_ELeaderboardUploadScoreMethodKeepBest
                }
                UploadScoreMethod::ForceUpdate => {
                    sys::ELeaderboardUploadScoreMethod::k_ELeaderboardUploadScoreMethodForceUpdate
                }
            };
            let api_call = sys::SteamAPI_ISteamUserStats_UploadLeaderboardScore(
                self.user_stats,
                leaderboard.0,
                method,
                score,
                details.as_ptr(),
                details.len() as _,
            );
            register_call_result::<sys::LeaderboardScoreUploaded_t, _>(
                &self.inner,
                api_call,
                move |v, io_error| {
                    cb(if io_error {
                        Err(SteamError::IOFailure)
                    } else {
                        Ok(if v.m_bSuccess != 0 {
                            Some(LeaderboardScoreUploaded {
                                score: v.m_nScore,
                                was_changed: v.m_bScoreChanged != 0,
                                global_rank_new: v.m_nGlobalRankNew as _,
                                global_rank_previous: v.m_nGlobalRankPrevious as _,
                            })
                        } else {
                            None
                        })
                    })
                },
            );
        }
    }

    pub fn download_leaderboard_entries<F>(
        &self,
        leaderboard: &Leaderboard,
        request: LeaderboardDataRequest,
        start: usize,
        end: usize,
        max_details_len: usize,
        cb: F,
    ) where
        F: FnOnce(Result<Vec<LeaderboardEntry>, SteamError>) + 'static + Send,
    {
        unsafe {
            let request = match request {
                LeaderboardDataRequest::Global => {
                    sys::ELeaderboardDataRequest::k_ELeaderboardDataRequestGlobal
                }
                LeaderboardDataRequest::GlobalAroundUser => {
                    sys::ELeaderboardDataRequest::k_ELeaderboardDataRequestGlobalAroundUser
                }
                LeaderboardDataRequest::Friends => {
                    sys::ELeaderboardDataRequest::k_ELeaderboardDataRequestFriends
                }
            };
            let api_call = sys::SteamAPI_ISteamUserStats_DownloadLeaderboardEntries(
                self.user_stats,
                leaderboard.0,
                request,
                start as _,
                end as _,
            );
            let user_stats = self.user_stats as isize;
            register_call_result::<sys::LeaderboardScoresDownloaded_t, _>(
                &self.inner,
                api_call,
                move |v, io_error| {
                    cb(if io_error {
                        Err(SteamError::IOFailure)
                    } else {
                        let len = v.m_cEntryCount;
                        let mut entries = Vec::with_capacity(len as usize);
                        for idx in 0..len {
                            let mut entry: sys::LeaderboardEntry_t = std::mem::zeroed();
                            let mut details = Vec::with_capacity(max_details_len);

                            sys::SteamAPI_ISteamUserStats_GetDownloadedLeaderboardEntry(
                                user_stats as *mut _,
                                v.m_hSteamLeaderboardEntries,
                                idx,
                                &mut entry,
                                details.as_mut_ptr(),
                                max_details_len as _,
                            );
                            details.set_len(entry.m_cDetails as usize);

                            entries.push(LeaderboardEntry {
                                user: SteamId(entry.m_steamIDUser.m_steamid.m_unAll64Bits),
                                global_rank: entry.m_nGlobalRank,
                                score: entry.m_nScore,
                                details,
                            })
                        }
                        Ok(entries)
                    })
                },
            );
        }
    }

    /// Returns the display type of a leaderboard handle. Returns `None` if the leaderboard handle is invalid.
    pub fn get_leaderboard_display_type(
        &self,
        leaderboard: &Leaderboard,
    ) -> Option<LeaderboardDisplayType> {
        unsafe {
            match sys::SteamAPI_ISteamUserStats_GetLeaderboardDisplayType(
                self.user_stats,
                leaderboard.0,
            ) {
                sys::ELeaderboardDisplayType::k_ELeaderboardDisplayTypeNumeric => {
                    Some(LeaderboardDisplayType::Numeric)
                }
                sys::ELeaderboardDisplayType::k_ELeaderboardDisplayTypeTimeSeconds => {
                    Some(LeaderboardDisplayType::TimeSeconds)
                }
                sys::ELeaderboardDisplayType::k_ELeaderboardDisplayTypeTimeMilliSeconds => {
                    Some(LeaderboardDisplayType::TimeMilliSeconds)
                }
                _ => None,
            }
        }
    }

    /// Returns the sort method of a leaderboard handle. Returns `None` if the leaderboard handle is invalid.
    pub fn get_leaderboard_sort_method(
        &self,
        leaderboard: &Leaderboard,
    ) -> Option<LeaderboardSortMethod> {
        unsafe {
            match sys::SteamAPI_ISteamUserStats_GetLeaderboardSortMethod(
                self.user_stats,
                leaderboard.0,
            ) {
                sys::ELeaderboardSortMethod::k_ELeaderboardSortMethodAscending => {
                    Some(LeaderboardSortMethod::Ascending)
                }
                sys::ELeaderboardSortMethod::k_ELeaderboardSortMethodDescending => {
                    Some(LeaderboardSortMethod::Descending)
                }
                _ => None,
            }
        }
    }

    /// Returns the name of a leaderboard handle. Returns an empty string if the leaderboard handle is invalid.
    pub fn get_leaderboard_name(&self, leaderboard: &Leaderboard) -> String {
        unsafe {
            let name = CStr::from_ptr(sys::SteamAPI_ISteamUserStats_GetLeaderboardName(
                self.user_stats,
                leaderboard.0,
            ));
            name.to_string_lossy().into()
        }
    }

    /// Returns the total number of entries in a leaderboard. Returns 0 if the leaderboard handle is invalid.
    pub fn get_leaderboard_entry_count(&self, leaderboard: &Leaderboard) -> i32 {
        unsafe {
            sys::SteamAPI_ISteamUserStats_GetLeaderboardEntryCount(self.user_stats, leaderboard.0)
        }
    }

    /// Triggers a [`UserStatsReceived`](./struct.UserStatsReceived.html) callback.
    pub fn request_user_stats(&self, steam_user_id: u64) {
        unsafe {
            sys::SteamAPI_ISteamUserStats_RequestUserStats(self.user_stats, steam_user_id);
        }
    }

    /// Asynchronously fetch the data for the percentage of players who have received each achievement
    /// for the current game globally.
    ///
    /// You must have called `request_current_stats()` and it needs to return successfully via its
    /// callback prior to calling this!*
    ///
    /// **Note: Not sure if this is applicable, as the other achievement functions requiring
    /// `request_current_stats()` don't specifically need it to be called in order for them to complete
    /// successfully. Maybe it autoruns via `Client::init()/init_app()` somehow?*
    pub fn request_global_achievement_percentages<F>(&self, cb: F)
    where
        F: FnOnce(Result<GameId, SteamError>) + 'static + Send,
    {
        unsafe {
            let api_call =
                sys::SteamAPI_ISteamUserStats_RequestGlobalAchievementPercentages(self.user_stats);
            register_call_result::<sys::GlobalAchievementPercentagesReady_t, _>(
                &self.inner,
                api_call,
                move |v, io_error| {
                    cb(if io_error {
                        Err(SteamError::IOFailure)
                    } else {
                        Ok(GameId(v.m_nGameID))
                    })
                },
            );
        }
    }

    /// Asynchronously requests global stats data, which is available for stats marked as "aggregated".
    ///
    /// This call is asynchronous, with the results returned in [`GlobalStatsReceived`](crate::GlobalStatsReceived) callback.
    ///
    /// # Arguments
    ///
    /// * `history_days` - Specifies how many days of day-by-day history to retrieve in addition
    ///   to the overall totals. The limit is 60.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use steamworks::*;
    /// # let client = steamworks::Client::init().unwrap();
    /// let user_stats = client.user_stats();
    ///
    /// // Request global stats with 7 days of history
    /// user_stats.request_global_stats(7, |result| {
    ///     match result {
    ///         Ok(game_id) => {
    ///             println!("Global stats received for game: {:?}", game_id);
    ///         }
    ///         Err(e) => {
    ///             println!("Failed to get global stats: {:?}", e);
    ///         }
    ///     }
    /// });
    /// ```
    pub fn request_global_stats<F>(&self, history_days: i32, cb: F)
    where
        F: FnOnce(Result<GameId, SteamError>) + 'static + Send,
    {
        unsafe {
            let api_call =
                sys::SteamAPI_ISteamUserStats_RequestGlobalStats(self.user_stats, history_days);
            register_call_result::<sys::GlobalStatsReceived_t, _>(
                &self.inner,
                api_call,
                move |v, io_error| {
                    cb(if io_error {
                        Err(SteamError::IOFailure)
                    } else {
                        Ok(GameId(v.m_nGameID))
                    })
                },
            );
        }
    }

    /// Gets the lifetime total for an aggregated stat as an `i64`.
    ///
    /// The specified stat must exist and be marked as "aggregated" in the Steamworks App Admin.
    ///
    /// Requires [`request_global_stats()`](Self::request_global_stats) to have been called
    /// and a successful [`GlobalStatsReceived`](crate::GlobalStatsReceived) callback processed.
    ///
    /// # Arguments
    ///
    /// * `name` - The 'API Name' of the stat. Must not be longer than `k_cchStatNameMax`.
    ///
    /// # Returns
    ///
    /// Returns `Ok(i64)` with the stat value if successful, or `Err(())` if the stat doesn't exist
    /// or hasn't been received yet.
    pub fn get_global_stat_i64(&self, name: &str) -> Result<i64, ()> {
        let name = CString::new(name).map_err(|_| ())?;
        let mut value: i64 = 0;
        let success = unsafe {
            sys::SteamAPI_ISteamUserStats_GetGlobalStatInt64(self.user_stats, name.as_ptr(), &mut value)
        };
        if success {
            Ok(value)
        } else {
            Err(())
        }
    }

    /// Gets the lifetime total for an aggregated stat as an `f64`.
    ///
    /// The specified stat must exist and be marked as "aggregated" in the Steamworks App Admin.
    ///
    /// Requires [`request_global_stats()`](Self::request_global_stats) to have been called
    /// and a successful [`GlobalStatsReceived`](crate::GlobalStatsReceived) callback processed.
    ///
    /// # Arguments
    ///
    /// * `name` - The 'API Name' of the stat. Must not be longer than `k_cchStatNameMax`.
    ///
    /// # Returns
    ///
    /// Returns `Ok(f64)` with the stat value if successful, or `Err(())` if the stat doesn't exist
    /// or hasn't been received yet.
    pub fn get_global_stat_f64(&self, name: &str) -> Result<f64, ()> {
        let name = CString::new(name).map_err(|_| ())?;
        let mut value: f64 = 0.0;
        let success = unsafe {
            sys::SteamAPI_ISteamUserStats_GetGlobalStatDouble(self.user_stats, name.as_ptr(), &mut value)
        };
        if success {
            Ok(value)
        } else {
            Err(())
        }
    }

    /// Gets history for an aggregated stat as `i64` values.
    ///
    /// The data will be filled with daily values, starting with today.
    /// So when called, `data[0]` will be today, `data[1]` will be yesterday, and `data[2]` will be
    /// two days ago, etc.
    ///
    /// The specified stat must exist and be marked as "aggregated" in the Steamworks App Admin.
    ///
    /// Requires [`request_global_stats()`](Self::request_global_stats) to have been called
    /// and a successful [`GlobalStatsReceived`](crate::GlobalStatsReceived) callback processed.
    ///
    /// # Arguments
    ///
    /// * `name` - The 'API Name' of the stat. Must not be longer than `k_cchStatNameMax`.
    /// * `max_days` - The maximum number of days of history to retrieve. This should match
    ///   or be less than the `history_days` value passed to `request_global_stats()`.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Vec<i64>)` containing the daily values (from today backwards) if successful,
    /// or `Err(())` if the stat doesn't exist or hasn't been received yet.
    pub fn get_global_stat_history_i64(&self, name: &str, max_days: usize) -> Result<Vec<i64>, ()> {
        let name = CString::new(name).map_err(|_| ())?;
        let mut data = vec![0i64; max_days];
        let count = unsafe {
            sys::SteamAPI_ISteamUserStats_GetGlobalStatHistoryInt64(
                self.user_stats,
                name.as_ptr(),
                data.as_mut_ptr(),
                (max_days * std::mem::size_of::<i64>()) as u32,
            )
        };
        if count >= 0 {
            data.truncate(count as usize);
            Ok(data)
        } else {
            Err(())
        }
    }

    /// Gets history for an aggregated stat as `f64` values.
    ///
    /// The data will be filled with daily values, starting with today.
    /// So when called, `data[0]` will be today, `data[1]` will be yesterday, and `data[2]` will be
    /// two days ago, etc.
    ///
    /// The specified stat must exist and be marked as "aggregated" in the Steamworks App Admin.
    ///
    /// Requires [`request_global_stats()`](Self::request_global_stats) to have been called
    /// and a successful [`GlobalStatsReceived`](crate::GlobalStatsReceived) callback processed.
    ///
    /// # Arguments
    ///
    /// * `name` - The 'API Name' of the stat. Must not be longer than `k_cchStatNameMax`.
    /// * `max_days` - The maximum number of days of history to retrieve. This should match
    ///   or be less than the `history_days` value passed to `request_global_stats()`.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Vec<f64>)` containing the daily values (from today backwards) if successful,
    /// or `Err(())` if the stat doesn't exist or hasn't been received yet.
    pub fn get_global_stat_history_f64(&self, name: &str, max_days: usize) -> Result<Vec<f64>, ()> {
        let name = CString::new(name).map_err(|_| ())?;
        let mut data = vec![0f64; max_days];
        let count = unsafe {
            sys::SteamAPI_ISteamUserStats_GetGlobalStatHistoryDouble(
                self.user_stats,
                name.as_ptr(),
                data.as_mut_ptr(),
                (max_days * std::mem::size_of::<f64>()) as u32,
            )
        };
        if count >= 0 {
            data.truncate(count as usize);
            Ok(data)
        } else {
            Err(())
        }
    }

    /// Send the changed stats and achievements data to the server for permanent storage.
    ///
    /// * Triggers a [`UserStatsStored`](../struct.UserStatsStored.html) callback if successful.
    /// * Triggers a [`UserAchievementStored`](../struct.UserAchievementStored.html) callback
    ///   if achievements have been unlocked.
    ///
    /// Requires [`request_current_stats()`](#method.request_current_stats) to have been called
    /// and a successful [`UserStatsReceived`](./struct.UserStatsReceived.html) callback processed.
    pub fn store_stats(&self) -> Result<(), ()> {
        let success = unsafe { sys::SteamAPI_ISteamUserStats_StoreStats(self.user_stats) };
        if success {
            Ok(())
        } else {
            Err(())
        }
    }

    /// Resets the current users stats and, optionally achievements.
    pub fn reset_all_stats(&self, achievements_too: bool) -> Result<(), ()> {
        let success = unsafe {
            sys::SteamAPI_ISteamUserStats_ResetAllStats(self.user_stats, achievements_too)
        };
        if success {
            Ok(())
        } else {
            Err(())
        }
    }

    /// Gets the value of a given stat for the current user
    ///
    /// The specified stat must exist and match the type set on the Steamworks App Admin website.
    ///
    /// Requires [`request_current_stats()`](#method.request_current_stats) to have been called
    /// and a successful [`UserStatsReceived`](./struct.UserStatsReceived.html) callback processed.
    pub fn get_stat_i32(&self, name: &str) -> Result<i32, ()> {
        let name = CString::new(name).unwrap();

        let mut value: i32 = 0;
        let success = unsafe {
            sys::SteamAPI_ISteamUserStats_GetStatInt32(self.user_stats, name.as_ptr(), &mut value)
        };
        if success {
            Ok(value)
        } else {
            Err(())
        }
    }

    /// Sets / updates the value of a given stat for the current user
    ///
    /// This call only changes the value in-memory and is very cheap. To commit the stats you
    /// must call [`store_stats()`](#method.store_stats)
    ///
    /// The specified stat must exist and match the type set on the Steamworks App Admin website.
    ///
    /// Requires [`request_current_stats()`](#method.request_current_stats) to have been called
    /// and a successful [`UserStatsReceived`](./struct.UserStatsReceived.html) callback processed.
    pub fn set_stat_i32(&self, name: &str, stat: i32) -> Result<(), ()> {
        let name = CString::new(name).unwrap();

        let success = unsafe {
            sys::SteamAPI_ISteamUserStats_SetStatInt32(self.user_stats, name.as_ptr(), stat)
        };
        if success {
            Ok(())
        } else {
            Err(())
        }
    }

    /// Gets the value of a given stat for the current user
    ///
    /// The specified stat must exist and match the type set on the Steamworks App Admin website.
    ///
    /// Requires [`request_current_stats()`](#method.request_current_stats) to have been called
    /// and a successful [`UserStatsReceived`](./struct.UserStatsReceived.html) callback processed.
    pub fn get_stat_f32(&self, name: &str) -> Result<f32, ()> {
        let name = CString::new(name).unwrap();

        let mut value: f32 = 0.0;
        let success = unsafe {
            sys::SteamAPI_ISteamUserStats_GetStatFloat(self.user_stats, name.as_ptr(), &mut value)
        };
        if success {
            Ok(value)
        } else {
            Err(())
        }
    }

    /// Sets / updates the value of a given stat for the current user
    ///
    /// This call only changes the value in-memory and is very cheap. To commit the stats you
    /// must call [`store_stats()`](#method.store_stats)
    ///
    /// The specified stat must exist and match the type set on the Steamworks App Admin website.
    ///
    /// Requires [`request_current_stats()`](#method.request_current_stats) to have been called
    /// and a successful [`UserStatsReceived`](./struct.UserStatsReceived.html) callback processed.
    pub fn set_stat_f32(&self, name: &str, stat: f32) -> Result<(), ()> {
        let name = CString::new(name).unwrap();

        let success = unsafe {
            sys::SteamAPI_ISteamUserStats_SetStatFloat(self.user_stats, name.as_ptr(), stat)
        };
        if success {
            Ok(())
        } else {
            Err(())
        }
    }

    /// Access achievement API for a given achievement 'API Name'.
    ///
    /// Requires [`request_current_stats()`](#method.request_current_stats) to have been called
    /// and a successful [`UserStatsReceived`](./struct.UserStatsReceived.html) callback processed.
    #[inline]
    #[must_use]
    pub fn achievement(&self, name: &str) -> stats::AchievementHelper<'_> {
        stats::AchievementHelper {
            name: CString::new(name).unwrap(),
            parent: self,
        }
    }

    /// Get the number of achievements defined in the App Admin panel of the Steamworks website.
    ///
    /// This is used for iterating through all of the achievements with GetAchievementName.
    ///
    /// Returns 0 if the current App ID has no achievements.
    ///
    /// *Note: Returns an error for AppId `480` (Spacewar)!*
    pub fn get_num_achievements(&self) -> Result<u32, ()> {
        unsafe {
            let num = sys::SteamAPI_ISteamUserStats_GetNumAchievements(self.user_stats);
            if num != 0 {
                Ok(num)
            } else {
                Err(())
            }
        }
    }

    /// Returns an array of all achievement names for the current AppId.
    ///
    /// Returns an empty string for an achievement name if `iAchievement` is not a valid index,
    /// and the current AppId must have achievements.
    pub fn get_achievement_names(&self) -> Option<Vec<String>> {
        let num = self
            .get_num_achievements()
            .expect("Failed to get number of achievements");
        let mut names = Vec::new();

        for i in 0..num {
            unsafe {
                let name = sys::SteamAPI_ISteamUserStats_GetAchievementName(self.user_stats, i);

                let c_str = CStr::from_ptr(name).to_string_lossy().into_owned();

                names.push(c_str);
            }
        }
        Some(names)
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LeaderboardEntry {
    pub user: SteamId,
    pub global_rank: i32,
    pub score: i32,
    pub details: Vec<i32>,
}

pub enum LeaderboardDataRequest {
    Global,
    GlobalAroundUser,
    Friends,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LeaderboardScoreUploaded {
    pub score: i32,
    pub was_changed: bool,
    pub global_rank_new: i32,
    pub global_rank_previous: i32,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum UploadScoreMethod {
    KeepBest,
    ForceUpdate,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum LeaderboardSortMethod {
    Ascending,
    Descending,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum LeaderboardDisplayType {
    Numeric,
    TimeSeconds,
    TimeMilliSeconds,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Leaderboard(u64);

impl Leaderboard {
    /// Returns the raw 64 bit value of the leaderboard id
    ///
    /// Useful for serializing leaderboard ids over a
    /// network or to a save format.
    pub fn raw(&self) -> u64 {
        self.0
    }
}

#[test]
#[ignore]
#[serial]
fn test() {
    let client = Client::init().unwrap();

    let stats = client.user_stats();

    stats.find_leaderboard("steamworks_test", |lb| {
        println!("Got: {:?}", lb);
    });
    let c2 = client.clone();
    stats.find_or_create_leaderboard(
        "steamworks_test_created",
        LeaderboardSortMethod::Descending,
        LeaderboardDisplayType::TimeMilliSeconds,
        move |lb| {
            println!("Got: {:?}", lb);

            if let Some(lb) = lb.ok().and_then(|v| v) {
                c2.user_stats().upload_leaderboard_score(
                    &lb,
                    UploadScoreMethod::ForceUpdate,
                    1337,
                    &[1, 2, 3, 4],
                    |v| {
                        println!("Upload: {:?}", v);
                    },
                );
                c2.user_stats().download_leaderboard_entries(
                    &lb,
                    LeaderboardDataRequest::Global,
                    0,
                    200,
                    10,
                    |v| {
                        println!("Download: {:?}", v);
                    },
                );
            }
        },
    );

    for _ in 0..50 {
        client.run_callbacks();
        ::std::thread::sleep(::std::time::Duration::from_millis(100));
    }
}

#[test]
#[ignore]
#[serial]
fn test_global_stats() {
    let client = Client::init().unwrap();
    let stats = client.user_stats();

    // Get stat name from environment variable, default to "test_stat"
    let stat_name = std::env::var("TEST_GLOBAL_STAT_NAME")
        .unwrap_or_else(|_| "test_stat".to_string());
    println!("Using global stat name: {}", stat_name);

    // Test request_global_stats with 7 days of history
    let c2 = client.clone();
    let stat_name_clone = stat_name.clone();
    stats.request_global_stats(7, move |result| {
        match result {
            Ok(game_id) => {
                println!("Global stats received for game: {:?}", game_id);

                // Test get_global_stat_i64
                match c2.user_stats().get_global_stat_i64(&stat_name_clone) {
                    Ok(value) => println!("Global stat (i64): {}", value),
                    Err(_) => println!("Failed to get global stat (i64) - stat may not exist or not be aggregated"),
                }

                // Test get_global_stat_f64
                match c2.user_stats().get_global_stat_f64(&stat_name_clone) {
                    Ok(value) => println!("Global stat (f64): {}", value),
                    Err(_) => println!("Failed to get global stat (f64) - stat may not exist or not be aggregated"),
                }

                // Test get_global_stat_history_i64
                match c2.user_stats().get_global_stat_history_i64(&stat_name_clone, 7) {
                    Ok(history) => println!("Global stat history (i64): {:?}", history),
                    Err(_) => println!("Failed to get global stat history (i64)"),
                }

                // Test get_global_stat_history_f64
                match c2.user_stats().get_global_stat_history_f64(&stat_name_clone, 7) {
                    Ok(history) => println!("Global stat history (f64): {:?}", history),
                    Err(_) => println!("Failed to get global stat history (f64)"),
                }
            }
            Err(e) => {
                println!("Failed to get global stats: {:?}", e);
            }
        }
    });

    // Run callbacks to process the async result
    for _ in 0..50 {
        client.run_callbacks();
        ::std::thread::sleep(::std::time::Duration::from_millis(100));
    }
}

#[test]
fn test_global_stat_cstring_error() {
    // Test that get_global_stat methods properly handle invalid CString input
    // This test doesn't require Steam to be running

    // Create a string with null byte which is invalid for CString
    let invalid_name = "test\0stat";

    // We can't actually test the UserStats methods without initializing Steam,
    // but we can verify the CString conversion behavior
    assert!(CString::new(invalid_name).is_err());
}
