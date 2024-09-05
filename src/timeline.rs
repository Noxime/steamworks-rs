use super::*;
use std::time::Duration;

pub struct Timeline<Manager> {
    pub(crate) timeline: *mut sys::ISteamTimeline,
    /// Whether the client's steam API is not recent enough.
    pub(crate) disabled: bool,
    pub(crate) _inner: Arc<Inner<Manager>>,
}

pub enum TimelineGameMode {
    /// The player is fully loaded into the game and playing.
    Playing,
    /// The player is in a multiplayer lobby.
    Staging,
    /// The player is in the game's main menu or a pause menu.
    Menus,
    /// The player is waiting for a loading screen.
    LoadingScreen,
}

impl From<TimelineGameMode> for sys::ETimelineGameMode {
    fn from(mode: TimelineGameMode) -> Self {
        match mode {
            TimelineGameMode::Playing => sys::ETimelineGameMode::k_ETimelineGameMode_Playing,
            TimelineGameMode::Staging => sys::ETimelineGameMode::k_ETimelineGameMode_Staging,
            TimelineGameMode::Menus => sys::ETimelineGameMode::k_ETimelineGameMode_Menus,
            TimelineGameMode::LoadingScreen => {
                sys::ETimelineGameMode::k_ETimelineGameMode_LoadingScreen
            }
        }
    }
}

pub enum TimelineEventClipPriority {
    /// This event is not appropriate as a clip.
    None,
    /// The user may want to make a clip around this event.
    Standard,
    /// The player will be likely to want a clip around event,
    /// and those clips should be promoted more prominently than clips with the [TimelineEventClipPriority::Standard] priority.
    Featured,
}

impl From<TimelineEventClipPriority> for sys::ETimelineEventClipPriority {
    fn from(priority: TimelineEventClipPriority) -> Self {
        match priority {
            TimelineEventClipPriority::None => {
                sys::ETimelineEventClipPriority::k_ETimelineEventClipPriority_None
            }
            TimelineEventClipPriority::Standard => {
                sys::ETimelineEventClipPriority::k_ETimelineEventClipPriority_Standard
            }
            TimelineEventClipPriority::Featured => {
                sys::ETimelineEventClipPriority::k_ETimelineEventClipPriority_Featured
            }
        }
    }
}

impl<Manager> Timeline<Manager> {
    /// Changes the color of the timeline bar.
    pub fn set_timeline_game_mode(&self, mode: TimelineGameMode) {
        if self.disabled {
            return;
        }

        unsafe {
            sys::SteamAPI_ISteamTimeline_SetTimelineGameMode(self.timeline, mode.into());
        }
    }

    /// Sets a description for the current game state in the timeline.
    /// These help the user to find specific moments in the timeline when saving clips.
    /// Setting a new state description replaces any previous description.
    pub fn set_timeline_state_description(&self, description: &str, duration: Duration) {
        if self.disabled {
            return;
        }

        let description = CString::new(description).unwrap();

        unsafe {
            sys::SteamAPI_ISteamTimeline_SetTimelineStateDescription(
                self.timeline,
                description.as_ptr(),
                duration.as_secs_f32(),
            )
        }
    }

    /// Clears the previous set game state in the timeline.
    pub fn clear_timeline_state_description(&self, duration: Duration) {
        if self.disabled {
            return;
        }

        unsafe {
            sys::SteamAPI_ISteamTimeline_ClearTimelineStateDescription(
                self.timeline,
                duration.as_secs_f32(),
            )
        }
    }

    /// Use this to mark an event on the Timeline.
    /// The event can be instantaneous or take some amount of time to complete,
    /// depending on the value passed in `duration`.
    pub fn add_timeline_event(
        &self,
        icon: &str,
        title: &str,
        description: &str,
        priority: u32,
        start_offset_seconds: f32,
        duration: Duration,
        clip_priority: TimelineEventClipPriority,
    ) {
        if self.disabled {
            return;
        }

        let icon = CString::new(icon).unwrap();
        let title = CString::new(title).unwrap();
        let description = CString::new(description).unwrap();
        let duration = duration.as_secs_f32();

        unsafe {
            sys::SteamAPI_ISteamTimeline_AddTimelineEvent(
                self.timeline,
                icon.as_ptr(),
                title.as_ptr(),
                description.as_ptr(),
                priority,
                start_offset_seconds,
                duration,
                clip_priority.into(),
            )
        }
    }
}
