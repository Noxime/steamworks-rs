pub mod stats;
mod stat_callback;

pub use self::stat_callback::*;
use super::*;
#[cfg(test)]
use serial_test_derive::serial;

/// Access to the steam user interface
pub struct UserStats<Manager> {
    pub(crate) user_stats: *mut sys::ISteamUserStats,
    pub(crate) inner: Arc<Inner<Manager>>,
}

const CALLBACK_BASE_ID: i32 = 1100;

impl <Manager> UserStats<Manager> {

    pub fn find_leaderboard<F>(&self, name: &str, mut cb: F)
        where F: FnMut(Result<Option<Leaderboard>, SteamError>) + 'static + Send
    {
        unsafe {
            let name = CString::new(name).unwrap();
            let api_call = sys::SteamAPI_ISteamUserStats_FindLeaderboard(self.user_stats, name.as_ptr() as *const _);
            register_call_result::<sys::LeaderboardFindResult_t, _, _>(
                &self.inner, api_call, CALLBACK_BASE_ID + 4,
                move |v, io_error| {
                    cb(if io_error {
                        Err(SteamError::IOFailure)
                    } else {
                        Ok(if v.m_bLeaderboardFound != 0 {
                            Some(Leaderboard(v.m_hSteamLeaderboard.0))
                        } else {
                            None
                        })
                    })
            });
        }
    }

    pub fn find_or_create_leaderboard<F>(&self, name: &str, sort_method: LeaderboardSortMethod, display_type: LeaderboardDisplayType, mut cb: F)
        where F: FnMut(Result<Option<Leaderboard>, SteamError>) + 'static + Send
    {
        unsafe {
            let name = CString::new(name).unwrap();

            let sort_method = match sort_method {
                LeaderboardSortMethod::Ascending => sys::ELeaderboardSortMethod::ELeaderboardSortMethodAscending,
                LeaderboardSortMethod::Descending => sys::ELeaderboardSortMethod::ELeaderboardSortMethodDescending,
            };

            let display_type = match display_type {
	            LeaderboardDisplayType::Numeric => sys::ELeaderboardDisplayType::ELeaderboardDisplayTypeNumeric,
	            LeaderboardDisplayType::TimeSeconds => sys::ELeaderboardDisplayType::ELeaderboardDisplayTypeTimeSeconds,
	            LeaderboardDisplayType::TimeMilliSeconds => sys::ELeaderboardDisplayType::ELeaderboardDisplayTypeTimeMilliSeconds,
            };

            let api_call = sys::SteamAPI_ISteamUserStats_FindOrCreateLeaderboard(self.user_stats, name.as_ptr() as *const _, sort_method, display_type);
            register_call_result::<sys::LeaderboardFindResult_t, _, _>(
                &self.inner, api_call, CALLBACK_BASE_ID + 4,
                move |v, io_error| {
                    cb(if io_error {
                        Err(SteamError::IOFailure)
                    } else {
                        Ok(if v.m_bLeaderboardFound != 0 {
                            Some(Leaderboard(v.m_hSteamLeaderboard.0))
                        } else {
                            None
                        })
                    })
                }
            );
        }
    }

    pub fn upload_leaderboard_score<F>(&self, leaderboard: &Leaderboard, method: UploadScoreMethod, score: i32, details: &[i32], mut cb: F)
        where F: FnMut(Result<Option<LeaderboardScoreUploaded>, SteamError>) + 'static + Send
    {
        unsafe {
            let method = match method {
                UploadScoreMethod::KeepBest => sys::ELeaderboardUploadScoreMethod::ELeaderboardUploadScoreMethodKeepBest,
                UploadScoreMethod::ForceUpdate => sys::ELeaderboardUploadScoreMethod::ELeaderboardUploadScoreMethodForceUpdate,
            };
            let api_call = sys::SteamAPI_ISteamUserStats_UploadLeaderboardScore(self.user_stats, sys::SteamLeaderboard_t(leaderboard.0), method, score, details.as_ptr(), details.len() as _);
            register_call_result::<sys::LeaderboardScoreUploaded_t , _, _>(
                &self.inner, api_call, CALLBACK_BASE_ID + 6,
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
            });
        }
    }

    pub fn download_leaderboard_entries<F>(
        &self,
        leaderboard: &Leaderboard,
        request: LeaderboardDataRequest, start: usize, end: usize,
        max_details_len: usize,
        mut cb: F
    )
        where F: FnMut(Result<Vec<LeaderboardEntry>, SteamError>) + 'static + Send
    {
        unsafe {
            let request = match request {
                LeaderboardDataRequest::Global => sys::ELeaderboardDataRequest::ELeaderboardDataRequestGlobal,
                LeaderboardDataRequest::GlobalAroundUser => sys::ELeaderboardDataRequest::ELeaderboardDataRequestGlobalAroundUser,
                LeaderboardDataRequest::Friends => sys::ELeaderboardDataRequest::ELeaderboardDataRequestFriends,
            };
            let api_call = sys::SteamAPI_ISteamUserStats_DownloadLeaderboardEntries(self.user_stats, sys::SteamLeaderboard_t(leaderboard.0), request, start as _, end as _);
            let user_stats = self.user_stats as isize;
            register_call_result::<sys::LeaderboardScoresDownloaded_t , _, _>(
                &self.inner, api_call, CALLBACK_BASE_ID + 5,
                move |v, io_error| {
                    cb(if io_error {
                        Err(SteamError::IOFailure)
                    } else {
                        let len = v.m_cEntryCount;
                        let mut entries = Vec::with_capacity(len as usize);
                        for idx in 0 .. len {
                            let mut entry: sys::LeaderboardEntry_t = std::mem::uninitialized();
                            let mut details = Vec::with_capacity(max_details_len);

                            sys::SteamAPI_ISteamUserStats_GetDownloadedLeaderboardEntry(user_stats as *mut _, v.m_hSteamLeaderboardEntries, idx, &mut entry, details.as_mut_ptr(), max_details_len as _);
                            details.set_len(entry.m_cDetails as usize);

                            entries.push(LeaderboardEntry {
                                user: SteamId(entry.m_steamIDUser.0),
                                global_rank: entry.m_nGlobalRank,
                                score: entry.m_nScore,
                                details,
                            })
                        }
                        Ok(entries)
                    })
            });
        }
    }

    /// Triggers a [`UserStatsReceived`](./struct.UserStatsReceived.html) callback.
    pub fn request_current_stats(&self) {
        unsafe { sys::SteamAPI_ISteamUserStats_RequestCurrentStats(self.user_stats); }
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
        if success { Ok(()) } else { Err(()) }
    }

    /// Access achievement API for a given achievement 'API Name'.
    ///
    /// Requires [`request_current_stats()`](#method.request_current_stats) to have been called
    /// and a successful [`UserStatsReceived`](./struct.UserStatsReceived.html) callback processed.
    #[inline]
    #[must_use]
    pub fn achievement(&self, name: &str) -> stats::AchievementHelper<'_, Manager> {
        stats::AchievementHelper { name: CString::new(name).unwrap(), parent: self }
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

#[test]
#[serial]
fn test() {
    let (client, single) = Client::init().unwrap();

    let stats = client.user_stats();

    stats.find_leaderboard("steamworks_test", |lb| {
        println!("Got: {:?}", lb);
    });
    let c2 = client.clone();
    stats.find_or_create_leaderboard("steamworks_test_created", LeaderboardSortMethod::Descending, LeaderboardDisplayType::TimeMilliSeconds, move |lb| {
        println!("Got: {:?}", lb);

        if let Some(lb) = lb.ok().and_then(|v| v) {
            c2.user_stats().upload_leaderboard_score(&lb, UploadScoreMethod::ForceUpdate, 1337, &[1, 2, 3, 4], |v| {
                println!("Upload: {:?}", v);
            });
            c2.user_stats().download_leaderboard_entries(&lb, LeaderboardDataRequest::Global, 0, 200, 10, |v| {
                println!("Download: {:?}", v);
            });
        }
    });

    for _ in 0 .. 50 {
        single.run_callbacks();
        ::std::thread::sleep(::std::time::Duration::from_millis(100));
    }
}
