use super::*;
#[cfg(test)]
use serial_test::serial;

/// Access to the steam matchmaking interface
pub struct Matchmaking<Manager> {
    pub(crate) mm: *mut sys::ISteamMatchmaking,
    pub(crate) inner: Arc<Inner<Manager>>,
}

const CALLBACK_BASE_ID: i32 = 500;

/// The visibility of a lobby
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum LobbyType {
    Private,
    FriendsOnly,
    Public,
    Invisible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LobbyId(pub(crate) u64);

impl LobbyId {
    /// Creates a `LobbyId` from a raw 64 bit value.
    ///
    /// May be useful for deserializing lobby ids from
    /// a network or save format.
    pub fn from_raw(id: u64) -> LobbyId {
        LobbyId(id)
    }

    /// Returns the raw 64 bit value of the lobby id
    ///
    /// May be useful for serializing lobby ids over a
    /// network or to a save format.
    pub fn raw(&self) -> u64 {
        self.0
    }
}

impl<Manager> Matchmaking<Manager> {
    pub fn request_lobby_list<F>(&self, cb: F)
    where
        F: FnOnce(SResult<Vec<LobbyId>>) + 'static + Send,
    {
        unsafe {
            let api_call = sys::SteamAPI_ISteamMatchmaking_RequestLobbyList(self.mm);
            register_call_result::<sys::LobbyMatchList_t, _, _>(
                &self.inner,
                api_call,
                CALLBACK_BASE_ID + 10,
                move |v, io_error| {
                    cb(if io_error {
                        Err(SteamError::IOFailure)
                    } else {
                        let mut out = Vec::with_capacity(v.m_nLobbiesMatching as usize);
                        for idx in 0..v.m_nLobbiesMatching {
                            out.push(LobbyId(sys::SteamAPI_ISteamMatchmaking_GetLobbyByIndex(
                                sys::SteamAPI_SteamMatchmaking_v009(),
                                idx as _,
                            )));
                        }
                        Ok(out)
                    })
                },
            );
        }
    }

    /// Attempts to create a new matchmaking lobby
    ///
    /// The lobby with have the visibility of the of the passed
    /// `LobbyType` and a limit of `max_members` inside it.
    /// The `max_members` may not be higher than 250.
    ///
    /// # Triggers
    ///
    /// * `LobbyEnter`
    /// * `LobbyCreated`
    pub fn create_lobby<F>(&self, ty: LobbyType, max_members: u32, cb: F)
    where
        F: FnOnce(SResult<LobbyId>) + 'static + Send,
    {
        assert!(max_members <= 250); // Steam API limits
        unsafe {
            let ty = match ty {
                LobbyType::Private => sys::ELobbyType::k_ELobbyTypePrivate,
                LobbyType::FriendsOnly => sys::ELobbyType::k_ELobbyTypeFriendsOnly,
                LobbyType::Public => sys::ELobbyType::k_ELobbyTypePublic,
                LobbyType::Invisible => sys::ELobbyType::k_ELobbyTypeInvisible,
            };
            let api_call =
                sys::SteamAPI_ISteamMatchmaking_CreateLobby(self.mm, ty, max_members as _);
            register_call_result::<sys::LobbyCreated_t, _, _>(
                &self.inner,
                api_call,
                CALLBACK_BASE_ID + 13,
                move |v, io_error| {
                    cb(if io_error {
                        Err(SteamError::IOFailure)
                    } else if v.m_eResult != sys::EResult::k_EResultOK {
                        Err(v.m_eResult.into())
                    } else {
                        Ok(LobbyId(v.m_ulSteamIDLobby))
                    })
                },
            );
        }
    }

    /// Tries to join the lobby with the given ID
    pub fn join_lobby<F>(&self, lobby: LobbyId, cb: F)
    where
        F: FnOnce(Result<LobbyId, ()>) + 'static + Send,
    {
        unsafe {
            let api_call = sys::SteamAPI_ISteamMatchmaking_JoinLobby(self.mm, lobby.0);
            register_call_result::<sys::LobbyEnter_t, _, _>(
                &self.inner,
                api_call,
                CALLBACK_BASE_ID + 4,
                move |v, io_error| {
                    cb(if io_error || v.m_EChatRoomEnterResponse != 1 {
                        Err(())
                    } else {
                        Ok(LobbyId(v.m_ulSteamIDLobby))
                    })
                },
            );
        }
    }

    /// Returns the number of data keys in the lobby
    pub fn lobby_data_count(&self, lobby: LobbyId) -> u32 {
        unsafe { sys::SteamAPI_ISteamMatchmaking_GetLobbyDataCount(self.mm, lobby.0) as _ }
    }

    /// Returns the lobby metadata associated with the specified key from the
    /// specified lobby.
    pub fn lobby_data(&self, lobby: LobbyId, key: &str) -> Option<&str> {
        let key = CString::new(key).unwrap();
        let data = unsafe {
            let data = sys::SteamAPI_ISteamMatchmaking_GetLobbyData(self.mm, lobby.0, key.as_ptr());
            let data = CStr::from_ptr(data);

            data
        };

        let data = data.to_str().unwrap();

        match data.is_empty() {
            false => Some(data),
            true => None,
        }
    }

    /// Returns the lobby metadata associated with the specified index
    pub fn lobby_data_by_index(&self, lobby: LobbyId, idx: u32) -> Option<(String, String)> {
        let mut key = [0i8; sys::k_nMaxLobbyKeyLength as usize];
        let mut value = [0i8; sys::k_cubChatMetadataMax as usize];
        unsafe {
            let success = sys::SteamAPI_ISteamMatchmaking_GetLobbyDataByIndex(
                self.mm,
                lobby.0,
                idx as _,
                key.as_mut_ptr() as _,
                key.len() as _,
                value.as_mut_ptr() as _,
                value.len() as _,
            );
            match success {
                true => Some((
                    CStr::from_ptr(key.as_ptr()).to_string_lossy().into_owned(),
                    CStr::from_ptr(value.as_ptr())
                        .to_string_lossy()
                        .into_owned(),
                )),
                false => None,
            }
        }
    }

    /// Sets the lobby metadata associated with the specified key in the specified lobby.
    pub fn set_lobby_data(&self, lobby: LobbyId, key: &str, value: &str) -> bool {
        let key = CString::new(key).unwrap();
        let value = CString::new(value).unwrap();
        unsafe {
            sys::SteamAPI_ISteamMatchmaking_SetLobbyData(
                self.mm,
                lobby.0,
                key.as_ptr(),
                value.as_ptr(),
            )
        }
    }

    /// Deletes the lobby metadata associated with the specified key in the specified lobby.
    pub fn delete_lobby_data(&self, lobby: LobbyId, key: &str) -> bool {
        let key = CString::new(key).unwrap();
        unsafe { sys::SteamAPI_ISteamMatchmaking_DeleteLobbyData(self.mm, lobby.0, key.as_ptr()) }
    }

    /// Exits the passed lobby
    pub fn leave_lobby(&self, lobby: LobbyId) {
        unsafe {
            sys::SteamAPI_ISteamMatchmaking_LeaveLobby(self.mm, lobby.0);
        }
    }

    /// Returns the current limit on the number of players in a lobby.
    ///
    /// Returns `[None]` if no metadata is available for the specified lobby.
    pub fn lobby_member_limit(&self, lobby: LobbyId) -> Option<usize> {
        unsafe {
            let count = sys::SteamAPI_ISteamMatchmaking_GetLobbyMemberLimit(self.mm, lobby.0);
            match count {
                0 => None,
                _ => Some(count as usize),
            }
        }
    }

    /// Returns the steam id of the current owner of the passed lobby
    pub fn lobby_owner(&self, lobby: LobbyId) -> SteamId {
        unsafe {
            SteamId(sys::SteamAPI_ISteamMatchmaking_GetLobbyOwner(
                self.mm, lobby.0,
            ))
        }
    }

    /// Returns the number of players in a lobby.
    ///
    /// Useful if you are not currently in the lobby
    pub fn lobby_member_count(&self, lobby: LobbyId) -> usize {
        unsafe {
            let count = sys::SteamAPI_ISteamMatchmaking_GetNumLobbyMembers(self.mm, lobby.0);
            count as usize
        }
    }

    /// Returns a list of members currently in the lobby
    pub fn lobby_members(&self, lobby: LobbyId) -> Vec<SteamId> {
        unsafe {
            let count = sys::SteamAPI_ISteamMatchmaking_GetNumLobbyMembers(self.mm, lobby.0);
            let mut members = Vec::with_capacity(count as usize);
            for idx in 0..count {
                members.push(SteamId(
                    sys::SteamAPI_ISteamMatchmaking_GetLobbyMemberByIndex(self.mm, lobby.0, idx),
                ))
            }
            members
        }
    }

    /// Sets whether or not a lobby is joinable by other players. This always defaults to enabled
    /// for a new lobby.
    ///
    /// If joining is disabled, then no players can join, even if they are a friend or have been
    /// invited.
    ///
    /// Lobbies with joining disabled will not be returned from a lobby search.
    ///
    /// Returns true on success, false if the current user doesn't own the lobby.
    pub fn set_lobby_joinable(&self, lobby: LobbyId, joinable: bool) -> bool {
        unsafe { sys::SteamAPI_ISteamMatchmaking_SetLobbyJoinable(self.mm, lobby.0, joinable) }
    }

    /// Broadcasts a chat message (text or binary data) to all users in the lobby.
    ///
    /// # Parameters
    /// - `lobby`: The Steam ID of the lobby to send the chat message to.
    /// - `msg`: This can be text or binary data, up to 4 Kilobytes in size.
    ///
    /// # Description
    /// All users in the lobby (including the local user) will receive a `LobbyChatMsg_t` callback
    /// with the message.
    ///
    /// If you're sending binary data, you should prefix a header to the message so that you know
    /// to treat it as your custom data rather than a plain old text message.
    ///
    /// For communication that needs to be arbitrated (e.g., having a user pick from a set of characters),
    /// you can use the lobby owner as the decision maker. `GetLobbyOwner` returns the current lobby owner.
    /// There is guaranteed to always be one and only one lobby member who is the owner.
    /// So for the choose-a-character scenario, the user who is picking a character would send the binary
    /// message 'I want to be Zoe', the lobby owner would see that message, see if it was OK, and broadcast
    /// the appropriate result (user X is Zoe).
    ///
    /// These messages are sent via the Steam back-end, and so the bandwidth available is limited.
    /// For higher-volume traffic like voice or game data, you'll want to use the Steam Networking API.
    ///
    /// # Returns
    /// Returns `Ok(())` if the message was successfully sent. Returns an error of type `SteamError` if the
    /// message is too small or too large, or if no connection to Steam could be made.
    pub fn send_lobby_chat_message(&self, lobby: LobbyId, msg: &[u8]) -> Result<(), SteamError> {
        match unsafe {
            steamworks_sys::SteamAPI_ISteamMatchmaking_SendLobbyChatMsg(
                self.mm,
                lobby.0,
                msg.as_ptr() as *const c_void,
                msg.len() as i32,
            )
        } {
            true => Ok(()),
            false => Err(SteamError::IOFailure),
        }
    }
    /// Adds a string comparison filter to the lobby list request.
    ///
    /// This method adds a filter that compares a specific string attribute in lobbies
    /// with the provided value. Lobbies matching this criterion will be included in the result.
    ///
    /// # Arguments
    ///
    /// * `key`: The attribute key to compare.
    /// * `value`: The value to compare against.
    ///
    pub fn set_request_lobby_list_string_filter(&self, key: &str, value: &str) -> &Self {
        unsafe {
            sys::SteamAPI_ISteamMatchmaking_AddRequestLobbyListStringFilter(
                self.mm,
                key.as_ptr() as _,
                value.as_ptr() as _,
                sys::ELobbyComparison::k_ELobbyComparisonEqual,
            );
        }
        self
    }
    /// Adds a numerical comparison filter to the lobby list request.
    ///
    /// This method adds a filter that compares a specific numerical attribute in lobbies
    /// with the provided value. Lobbies matching this criterion will be included in the result.
    ///
    /// # Arguments
    ///
    /// * `key`: The attribute key to compare.
    /// * `value`: The value to compare against.
    ///
    pub fn set_request_lobby_list_numerical_filter(&self, key: &str, value: i32) -> &Self {
        unsafe {
            sys::SteamAPI_ISteamMatchmaking_AddRequestLobbyListNumericalFilter(
                self.mm,
                key.as_ptr() as _,
                value,
                sys::ELobbyComparison::k_ELobbyComparisonEqual,
            );
        }
        self
    }
    /// Adds a near value filter to the lobby list request.
    ///
    /// This method adds a filter that sorts the lobby results based on their closeness
    /// to a specific value. No actual filtering is performed; lobbies are sorted based on proximity.
    ///
    /// # Arguments
    ///
    /// * `key`: The attribute key to use for sorting.
    /// * `value`: The reference value for sorting.
    ///
    pub fn set_request_lobby_list_near_value_filter(&self, key: &str, value: i32) -> &Self {
        unsafe {
            sys::SteamAPI_ISteamMatchmaking_AddRequestLobbyListNearValueFilter(
                self.mm,
                key.as_ptr() as _,
                value,
            );
        }
        self
    }
    /// Adds a filter for available open slots to the lobby list request.
    ///
    /// This method adds a filter that includes lobbies having a specific number of open slots.
    ///
    /// # Arguments
    ///
    /// * `open_slots`: The number of open slots in a lobby to filter by.
    ///
    pub fn set_request_lobby_list_slots_available_filter(&self, open_slots: u8) -> &Self {
        unsafe {
            sys::SteamAPI_ISteamMatchmaking_AddRequestLobbyListFilterSlotsAvailable(
                self.mm,
                open_slots as i32,
            );
        }
        self
    }
    /// Adds a distance filter to the lobby list request.
    ///
    /// This method adds a filter that includes lobbies within a certain distance criterion.
    ///
    /// # Arguments
    ///
    /// * `distance`: The `DistanceFilter` indicating the distance criterion for the filter.
    ///
    pub fn set_request_lobby_list_distance_filter(&self, distance: DistanceFilter) -> &Self {
        unsafe {
            sys::SteamAPI_ISteamMatchmaking_AddRequestLobbyListDistanceFilter(
                self.mm,
                distance.into(),
            );
        }
        self
    }
    /// Adds a result count filter to the lobby list request.
    ///
    /// This method adds a filter to limit the number of lobby results returned by the request.
    ///
    /// # Arguments
    ///
    /// * `count`: The maximum number of lobby results to include in the response.
    ///
    pub fn set_request_lobby_list_result_count_filter(&self, count: u64) -> &Self {
        unsafe {
            sys::SteamAPI_ISteamMatchmaking_AddRequestLobbyListResultCountFilter(
                self.mm,
                count as i32,
            );
        }
        self
    }

    /// Sets filters for the lobbies to be returned from [`request_lobby_list`].
    ///
    /// This method is used to apply various filters to the lobby list retrieval process.
    /// Call this method before calling `request_lobby_list` to ensure that the specified filters
    /// are taken into account when fetching the list of available lobbies.
    ///
    /// # Arguments
    ///
    /// * `filter`: A [`LobbyListFilter`] struct containing the filter criteria to be applied.
    ///
    /// [`request_lobby_list`]: #method.request_lobby_list
    /// [`LobbyListFilter`]: struct.LobbyListFilter.html
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use steamworks::*;
    /// fn main() {
    ///     let (client, single) = Client::init().unwrap();
    ///     client.matchmaking().set_lobby_list_filter(
    ///         LobbyListFilter {
    ///            string: Some(("name", "My Lobby")),
    ///           ..Default::default()
    ///       }
    ///    ).request_lobby_list(|lobbies| {
    ///         println!("Lobbies: {:?}", lobbies);
    ///     });
    /// }
    /// ```
    pub fn set_lobby_list_filter(&self, filter: LobbyListFilter) -> &Self {
        if let Some((key, value)) = filter.string {
            self.set_request_lobby_list_string_filter(key, value);
        }
        if let Some((key, value)) = filter.number {
            self.set_request_lobby_list_numerical_filter(key, value);
        }
        if let Some((key, value)) = filter.near_value {
            self.set_request_lobby_list_near_value_filter(key, value);
        }
        if let Some(distance) = filter.distance {
            self.set_request_lobby_list_distance_filter(distance);
        }
        if let Some(open_slots) = filter.open_slots {
            self.set_request_lobby_list_slots_available_filter(open_slots);
        }
        if let Some(count) = filter.count {
            self.set_request_lobby_list_result_count_filter(count);
        }
        self
    }
}

/// Filters for the lobbies to be returned from `request_lobby_list`.
///
/// This struct is designed to be used as part of the filtering process
/// when calling the [`set_lobby_list_filter`] method.
///
/// # Fields
///
/// - `string`: A string comparison filter that matches lobby attributes with specific strings.
/// - `number`: A number comparison filter that matches lobby attributes with specific integer values.
/// - `near_value`: Specifies a value, and the results will be sorted closest to this value (no actual filtering).
/// - `open_slots`: Filters lobbies based on the number of open slots they have.
/// - `distance`: Filters lobbies based on a distance criterion.
/// - `count`: Specifies the maximum number of lobby results to be returned.
#[derive(Debug, Default, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LobbyListFilter<'a> {
    /// A string comparison filter that matches lobby attributes with specific strings.
    #[cfg_attr(feature = "serde", serde(borrow))]
    string: Option<(&'a str, &'a str)>,
    /// A number comparison filter that matches lobby attributes with specific integer values
    #[cfg_attr(feature = "serde", serde(borrow))]
    number: Option<(&'a str, i32)>,
    /// Specifies a value, and the results will be sorted closest to this value (no actual filtering)
    #[cfg_attr(feature = "serde", serde(borrow))]
    near_value: Option<(&'a str, i32)>,
    /// Filters lobbies based on the number of open slots they have
    open_slots: Option<u8>,
    /// Filters lobbies based on a distance criterion
    distance: Option<DistanceFilter>,
    /// Specifies the maximum number of lobby results to be returned
    count: Option<u64>,
}

impl<'a> LobbyListFilter<'a> {
    /// Sets the string comparison filter for the lobby list filter.
    ///
    /// # Arguments
    ///
    /// * `string`: A tuple containing the attribute name and the target string value to match.
    ///
    pub fn set_string(&mut self, string: Option<(&'a str, &'a str)>) {
        self.string = string;
    }

    /// Sets the number comparison filter for the lobby list filter.
    ///
    /// # Arguments
    ///
    /// * `number`: A tuple containing the attribute name and the target integer value to match.
    ///
    pub fn set_number(&mut self, number: Option<(&'a str, i32)>) {
        self.number = number;
    }

    /// Sets the near value filter for the lobby list filter.
    ///
    /// # Arguments
    ///
    /// * `near_value`: A tuple containing the attribute name and the reference integer value.
    ///                 Lobby results will be sorted based on their closeness to this value.
    ///
    pub fn set_near_value(&mut self, near_value: Option<(&'a str, i32)>) {
        self.near_value = near_value;
    }

    /// Sets the open slots filter for the lobby list filter.
    ///
    /// # Arguments
    ///
    /// * `open_slots`: The number of open slots to filter lobbies by.
    ///
    pub fn set_open_slots(&mut self, open_slots: Option<u8>) {
        self.open_slots = open_slots;
    }

    /// Sets the distance filter for the lobby list filter.
    ///
    /// # Arguments
    ///
    /// * `distance`: A distance filter that specifies a distance criterion for filtering lobbies.
    ///
    pub fn set_distance(&mut self, distance: Option<DistanceFilter>) {
        self.distance = distance;
    }

    /// Sets the maximum number of lobby results to be returned.
    ///
    /// # Arguments
    ///
    /// * `count`: The maximum number of lobby results to retrieve.
    ///
    pub fn set_count(&mut self, count: Option<u64>) {
        self.count = count;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum DistanceFilter {
    Close,
    #[default]
    Default,
    Far,
    Worldwide,
}

impl From<DistanceFilter> for sys::ELobbyDistanceFilter {
    fn from(filter: DistanceFilter) -> Self {
        match filter {
            DistanceFilter::Close => sys::ELobbyDistanceFilter::k_ELobbyDistanceFilterClose,
            DistanceFilter::Default => sys::ELobbyDistanceFilter::k_ELobbyDistanceFilterDefault,
            DistanceFilter::Far => sys::ELobbyDistanceFilter::k_ELobbyDistanceFilterFar,
            DistanceFilter::Worldwide => sys::ELobbyDistanceFilter::k_ELobbyDistanceFilterWorldwide,
        }
    }
}

/// Flags describing how a users lobby state has changed. This is provided from `LobbyChatUpdate`.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ChatMemberStateChange {
    /// This user has joined or is joining the lobby.
    Entered,

    /// This user has left or is leaving the lobby.
    Left,

    /// User disconnected without leaving the lobby first.
    Disconnected,

    /// The user has been kicked.
    Kicked,

    /// The user has been kicked and banned.
    Banned,
}

/// A lobby chat room state has changed, this is usually sent when a user has joined or left the lobby.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LobbyChatUpdate {
    /// The Steam ID of the lobby.
    pub lobby: LobbyId,
    /// The user who's status in the lobby just changed - can be recipient.
    pub user_changed: SteamId,
    /// Chat member who made the change. This can be different from `user_changed` if kicking, muting, etc. For example, if one user kicks another from the lobby, this will be set to the id of the user who initiated the kick.
    pub making_change: SteamId,

    /// "ChatMemberStateChange" values.
    pub member_state_change: ChatMemberStateChange,
}

unsafe impl Callback for LobbyChatUpdate {
    const ID: i32 = 506;
    const SIZE: i32 = ::std::mem::size_of::<sys::LobbyChatUpdate_t>() as i32;

    unsafe fn from_raw(raw: *mut c_void) -> Self {
        let val = &mut *(raw as *mut sys::LobbyChatUpdate_t);

        LobbyChatUpdate {
            lobby: LobbyId(val.m_ulSteamIDLobby),
            user_changed: SteamId(val.m_ulSteamIDUserChanged),
            making_change: SteamId(val.m_ulSteamIDUserChanged),
            member_state_change: match val.m_rgfChatMemberStateChange {
                x if x == sys::EChatMemberStateChange::k_EChatMemberStateChangeEntered as u32 => {
                    ChatMemberStateChange::Entered
                }
                x if x == sys::EChatMemberStateChange::k_EChatMemberStateChangeLeft as u32 => {
                    ChatMemberStateChange::Left
                }
                x if x
                    == sys::EChatMemberStateChange::k_EChatMemberStateChangeDisconnected as u32 =>
                {
                    ChatMemberStateChange::Disconnected
                }
                x if x == sys::EChatMemberStateChange::k_EChatMemberStateChangeKicked as u32 => {
                    ChatMemberStateChange::Kicked
                }
                x if x == sys::EChatMemberStateChange::k_EChatMemberStateChangeBanned as u32 => {
                    ChatMemberStateChange::Banned
                }
                _ => unreachable!(),
            },
        }
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LobbyDataUpdate {
    pub lobby: LobbyId,
    pub member: SteamId,
    pub success: bool,
}

unsafe impl Callback for LobbyDataUpdate {
    const ID: i32 = 505;
    const SIZE: i32 = ::std::mem::size_of::<sys::LobbyDataUpdate_t>() as i32;

    unsafe fn from_raw(raw: *mut c_void) -> Self {
        let val = &mut *(raw as *mut sys::LobbyDataUpdate_t);

        LobbyDataUpdate {
            lobby: LobbyId(val.m_ulSteamIDLobby),
            member: SteamId(val.m_ulSteamIDMember),
            success: val.m_bSuccess != 0,
        }
    }
}

#[test]
#[serial]
fn test_lobby() {
    let (client, single) = Client::init().unwrap();
    let mm = client.matchmaking();

    mm.request_lobby_list(|v| {
        println!("List: {:?}", v);
    });
    mm.create_lobby(LobbyType::Private, 4, |v| {
        println!("Create: {:?}", v);
    });

    for _ in 0..100 {
        single.run_callbacks();
        ::std::thread::sleep(::std::time::Duration::from_millis(100));
    }
}
