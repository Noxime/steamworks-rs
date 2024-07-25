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

    /// Returns the percentage of users who have unlocked the specified achievement.
    ///
    /// You must have called `request_global_achievement_percentages()` and it needs to return
    /// successfully via its callback prior to calling this.
    ///
    /// *Note: Always returns an error for AppId `480` (Spacewar)!
    /// Other AppIds work fine though.*
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use steamworks::*;
    /// # let (client, single) = steamworks::Client::init().unwrap();
    /// // Get the current unlock percentage for the 'WIN_THE_GAME' achievement
    /// client.user_stats().request_global_achievement_percentages(move|result| {
    ///     if !result.is_err() {
    ///         let user_stats = client.user_stats();
    ///         let achievement = user_stats.achievement("WIN_THE_GAME");
    ///         let ach_percent = achievement.get_achievement_achieved_percent().expect("Failed to get achievement percentage");
    ///
    ///         println!("{}",ach_percent);
    ///     } else {
    ///         println!("Error requesting global achievement percentages");
    ///     }
    /// });
    /// # Err(())
    /// ```
    pub fn get_achievement_achieved_percent(&self) -> Result<f32, ()> {
        unsafe {
            let mut percent = 0.0;
            let success = sys::SteamAPI_ISteamUserStats_GetAchievementAchievedPercent(
                self.parent.user_stats,
                self.name.as_ptr() as *const _,
                &mut percent as *mut _,
            );
            if success {
                Ok(percent)
            } else {
                Err(())
            }
        }
    }

    /// Get general attributes for an achievement. Currently provides: `Name`, `Description`,
    /// and `Hidden` status.
    ///
    /// This receives the value from a dictionary/map keyvalue store, so you must provide one
    /// of the following keys:
    ///
    /// - `"name"` to retrive the localized achievement name in UTF8
    /// - `"desc"` to retrive the localized achievement description in UTF8
    /// - `"hidden"` for retrieving if an achievement is hidden. Returns `"0"` when not hidden,
    /// `"1"` when hidden
    ///
    /// This localization is provided based on the games language if it's set, otherwise it
    /// checks if a localization is available for the users Steam UI Language. If that fails
    /// too, then it falls back to english.
    ///
    /// This function returns the value as a `string` upon success if all of the following
    /// conditions are met; otherwise, an empty string: `""`.
    ///
    /// - `request_current_stats()` has completed and successfully returned its callback.
    /// - The specified achievement exists in App Admin on the Steamworks website, and the
    /// changes are published.
    /// - The specified `pchKey` is valid.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use steamworks::*;
    /// # let (client, single) = steamworks::Client::init().unwrap();
    /// // Get the "description" string for the 'WIN_THE_GAME' achievement
    /// client.user_stats().achievement("WIN_THE_GAME").get_achievement_display_attribute("desc").unwrap();
    /// # Err(())
    /// ```
    pub fn get_achievement_display_attribute(&self, key: &str) -> Result<&str, ()> {
        unsafe {
            let key_c_str = CString::new(key).expect("Failed to create c_str from key parameter");
            let ptr = key_c_str.as_ptr() as *const i8;

            let str = sys::SteamAPI_ISteamUserStats_GetAchievementDisplayAttribute(
                self.parent.user_stats,
                self.name.as_ptr() as *const _,
                ptr,
            );

            let c_str = CStr::from_ptr(str);

            match c_str.to_str() {
                Ok(result) => Ok(result),
                Err(_) => Err(()),
            }
        }
    }

    /// Gets the icon for an achievement.
    ///
    /// The image is returned as a handle to be used with `ISteamUtils::GetImageRGBA` to get
    /// the actual image data.*
    ///
    /// **Note: This is handled within the function. Returns a `Vec<u8>` buffer on success,
    /// which can be converted into the image data and saved to disk (e.g. via external RGBA to image crate).*
    /// ** Note: This may return None if Steam has not retrieved the icon yet. In that case an `UserAchievementIconFetched` callback will be processed

    pub fn get_achievement_icon(&self) -> Option<Vec<u8>> {
        Some(self.internal_get_achievement_icon(true)?.0)
    }

    fn internal_get_achievement_icon(&self, avoid_big_icons: bool) -> Option<(Vec<u8>, u32, u32)> {
        unsafe {
            let utils: *mut sys::ISteamUtils = sys::SteamAPI_SteamUtils_v010();
            let img = sys::SteamAPI_ISteamUserStats_GetAchievementIcon(
                self.parent.user_stats,
                self.name.as_ptr() as *const _,
            );
            if img == 0 {
                return None;
            }
            let mut width = 0;
            let mut height = 0;
            if !sys::SteamAPI_ISteamUtils_GetImageSize(utils, img, &mut width, &mut height) {
                return None;
            }
            if avoid_big_icons && (width != 64 || height != 64) {
                return None;
            }
            let mut dest = vec![0; (width * height * 4).try_into().unwrap()];
            if !sys::SteamAPI_ISteamUtils_GetImageRGBA(
                utils,
                img,
                dest.as_mut_ptr(),
                (width * height * 4).try_into().unwrap(),
            ) {
                return None;
            }
            Some((dest, width, height))
        }
    }

    /// Gets the icon for an achievement.
    ///
    /// The image is returned as a handle to be used with `ISteamUtils::GetImageRGBA` to get
    /// the actual image data.*
    ///
    /// **Note: This is handled within the function. Returns a `ImageBuffer::<image::Rgba<u8>, Vec<u8>>` from the image crate on success**
    /// ** Note: This may return None if Steam has not retrieved the icon yet. In that case an `UserAchievementIconFetched` callback will be processed
    #[cfg(feature = "image")]
    pub fn get_achievement_icon_v2(&self) -> Option<image::ImageBuffer<image::Rgba<u8>, Vec<u8>>> {
        let (vec, width, height) = self.internal_get_achievement_icon(false)?;
        let img = image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_vec(width, height, vec)?;
        return Some(img);
    }
}
