use std::fmt::Display;

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

    /// Gets the data from a lobby chat message after receiving a `LobbyChatMsg_t` callback.
    ///
    /// # Parameters
    /// - `lobby`: The Steam ID of the lobby to get the chat message from.
    /// - `chat_id`: The index of the chat entry in the lobby.
    /// - `buffer`: Return the message data by copying it into this buffer. The buffer should be up
    /// to 4 Kilobytes.
    ///
    /// # Returns
    /// Returns `usize` The number of bytes copied into buffer
    pub fn get_lobby_chat_entry<'a>(
        &self,
        lobby: LobbyId,
        chat_id: i32,
        buffer: &'a mut [u8],
    ) -> &'a [u8] {
        let mut steam_user = sys::CSteamID {
            m_steamid: sys::CSteamID_SteamID_t { m_unAll64Bits: 0 },
        };
        let mut chat_type = steamworks_sys::EChatEntryType::k_EChatEntryTypeInvalid;
        unsafe {
            let len = sys::SteamAPI_ISteamMatchmaking_GetLobbyChatEntry(
                self.mm,
                lobby.0,
                chat_id,
                &mut steam_user,
                buffer.as_mut_ptr() as *mut _,
                buffer.len() as _,
                &mut chat_type,
            );
            return &buffer[0..len as usize];
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
    pub fn add_request_lobby_list_string_filter(
        &self,
        StringFilter(LobbyKey(key), value, kind): StringFilter,
    ) -> &Self {
        unsafe {
            sys::SteamAPI_ISteamMatchmaking_AddRequestLobbyListStringFilter(
                self.mm,
                key.as_ptr() as _,
                value.as_ptr() as _,
                kind.into(),
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
    pub fn add_request_lobby_list_numerical_filter(
        &self,
        NumberFilter(LobbyKey(key), value, comparison): NumberFilter,
    ) -> &Self {
        unsafe {
            sys::SteamAPI_ISteamMatchmaking_AddRequestLobbyListNumericalFilter(
                self.mm,
                key.as_ptr() as _,
                value,
                comparison.into(),
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
    pub fn add_request_lobby_list_near_value_filter(
        &self,
        NearFilter(LobbyKey(key), value): NearFilter,
    ) -> &Self {
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
    ///             string: Some(vec![
    ///                 StringFilter(
    ///                     LobbyKey::new("name"), "My Lobby", StringFilterKind::Include
    ///                 ),
    ///                 StringFilter(
    ///                     LobbyKey::new("gamemode"), "ffa", StringFilterKind::Include
    ///                 ),
    ///             ]),
    ///             number: Some(vec![
    ///                 NumberFilter("elo", 1500, ComparisonFilter::GreaterThan),
    ///                 NumberFilter("elo", 2000, ComparisonFilter::LessThan)
    ///             ]),
    ///             ..Default::default()
    ///         }
    ///     ).request_lobby_list(|lobbies| {
    ///         println!("Lobbies: {:?}", lobbies);
    ///     });
    /// }
    /// ```
    pub fn set_lobby_list_filter(&self, filter: LobbyListFilter<'_>) -> &Self {
        filter.string.into_iter().flatten().for_each(|str_filter| {
            self.add_request_lobby_list_string_filter(str_filter);
        });
        filter.number.into_iter().flatten().for_each(|num_filter| {
            self.add_request_lobby_list_numerical_filter(num_filter);
        });
        filter
            .near_value
            .into_iter()
            .flatten()
            .for_each(|near_filter| {
                self.add_request_lobby_list_near_value_filter(near_filter);
            });
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
#[derive(Debug, Clone, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LobbyListFilter<'a> {
    /// A string comparison filter that matches lobby attributes with specific strings.
    //#[cfg_attr(feature = "serde", serde(borrow))]
    pub string: Option<StringFilters<'a>>,
    /// A number comparison filter that matches lobby attributes with specific integer values
    #[cfg_attr(feature = "serde", serde(borrow))]
    pub number: Option<NumberFilters<'a>>,
    /// Specifies a value, and the results will be sorted closest to this value (no actual filtering)
    #[cfg_attr(feature = "serde", serde(borrow))]
    pub near_value: Option<NearFilters<'a>>,
    /// Filters lobbies based on the number of open slots they have
    pub open_slots: Option<u8>,
    /// Filters lobbies based on a distance criterion
    pub distance: Option<DistanceFilter>,
    /// Specifies the maximum number of lobby results to be returned
    pub count: Option<u64>,
}

/// A wrapper for a lobby key string.
///
/// This struct provides a wrapper for a lobby key string. It is used to validate
/// constructed keys and to ensure that they do not exceed the maximum allowed length.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LobbyKey<'a>(pub(crate) &'a str);

impl<'a> std::ops::Deref for LobbyKey<'a> {
    type Target = &'a str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Error)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LobbyKeyTooLongError;

impl Display for LobbyKeyTooLongError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Lobby key is greater than {} characters",
            sys::k_nMaxLobbyKeyLength
        )
    }
}

impl<'a> LobbyKey<'a> {
    /// Attempts to create a new `LobbyKey` from a provided string key.
    ///
    /// # Arguments
    ///
    /// * `key`: The string key to create a `LobbyKey` from.
    ///
    /// # Errors
    ///
    /// This function will return an error of type [`LobbyKeyTooLongError`] if the provided key's length
    /// exceeds k_nMaxLobbyKeyLength (255 characters).
    pub fn try_new(key: &'a str) -> Result<Self, LobbyKeyTooLongError> {
        if key.len() > sys::k_nMaxLobbyKeyLength as usize {
            Err(LobbyKeyTooLongError)
        } else {
            Ok(LobbyKey(key))
        }
    }
    /// Creates a new `LobbyKey` from a provided string key.
    ///
    /// # Arguments
    ///
    /// * `key`: The string key to create a `LobbyKey` from.
    ///
    /// # Panics
    ///
    /// This function will panic if the provided key's length exceeds 255 characters.
    /// ```
    pub fn new(key: &'a str) -> Self {
        Self::try_new(key).unwrap()
    }
}

pub type StringFilters<'a> = Vec<StringFilter<'a>>;
pub type NumberFilters<'a> = Vec<NumberFilter<'a>>;
pub type NearFilters<'a> = Vec<NearFilter<'a>>;

/// A filter used for string based key value comparisons.
///
/// # Fields
///
/// * `0`: The attribute key for comparison.
/// * `1`: The target string value for matching.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StringFilter<'a>(
    #[cfg_attr(feature = "serde", serde(borrow))] pub LobbyKey<'a>,
    pub &'a str,
    pub StringFilterKind,
);

#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum StringFilterKind {
    #[default]
    Include,
    Exclude,
}

impl From<StringFilterKind> for sys::ELobbyComparison {
    fn from(filter: StringFilterKind) -> Self {
        match filter {
            StringFilterKind::Include => sys::ELobbyComparison::k_ELobbyComparisonEqual,
            StringFilterKind::Exclude => sys::ELobbyComparison::k_ELobbyComparisonNotEqual,
        }
    }
}

/// A filter used for numerical attribute comparison in lobby filtering.
///
/// # Fields
///
/// * `key`: The attribute key for comparison.
/// * `value`: The target numerical value for matching.
/// * `comparison`: The comparison mode indicating how the numerical values should be compared.
///
/// # Example
///
/// ```no_run
/// let elo_filter = NumberFilter {
///     key: "lobby_elo",
///     value: 1500,
///     comparison: ComparisonFilter::GreaterThan,
/// };
/// ```
///
#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NumberFilter<'a>(
    #[cfg_attr(feature = "serde", serde(borrow))] pub LobbyKey<'a>,
    pub i32,
    pub ComparisonFilter,
);

/// A filter used for near-value sorting in lobby filtering.
///
/// This struct enables sorting the lobby results based on their closeness to a reference value.
/// It includes two fields: the attribute key to use for sorting and the reference numerical value.
///
/// This filter does not perform actual filtering but rather sorts the results based on proximity.
///
/// # Fields
///
/// * `0`: The attribute key to use for sorting.
/// * `1`: The reference numerical value used for sorting proximity.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NearFilter<'a>(
    #[cfg_attr(feature = "serde", serde(borrow))] pub LobbyKey<'a>,
    pub i32,
);

impl<'a> LobbyListFilter<'a> {
    /// Sets the string comparison filter for the lobby list filter.
    ///
    /// # Arguments
    ///
    /// * `string`: A tuple containing the attribute name and the target string value to match.
    ///
    pub fn set_string(mut self, string: Option<StringFilters<'a>>) -> Self {
        self.string = string;
        self
    }

    /// Sets the number comparison filter for the lobby list filter.
    ///
    /// # Arguments
    ///
    /// * `number`: A tuple containing the attribute name and the target integer value to match.
    ///
    pub fn set_number(mut self, number: Option<NumberFilters<'a>>) -> Self {
        self.number = number;
        self
    }

    /// Sets the near value filter for the lobby list filter.
    ///
    /// # Arguments
    ///
    /// * `near_value`: A tuple containing the attribute name and the reference integer value.
    ///                 Lobby results will be sorted based on their closeness to this value.
    ///
    pub fn set_near_value(mut self, near_value: Option<NearFilters<'a>>) -> Self {
        self.near_value = near_value;
        self
    }

    /// Sets the open slots filter for the lobby list filter.
    ///
    /// # Arguments
    ///
    /// * `open_slots`: The number of open slots to filter lobbies by.
    ///
    pub fn set_open_slots(mut self, open_slots: Option<u8>) -> Self {
        self.open_slots = open_slots;
        self
    }

    /// Sets the distance filter for the lobby list filter.
    ///
    /// # Arguments
    ///
    /// * `distance`: A distance filter that specifies a distance criterion for filtering lobbies.
    ///
    pub fn set_distance(mut self, distance: Option<DistanceFilter>) -> Self {
        self.distance = distance;
        self
    }

    /// Sets the maximum number of lobby results to be returned.
    ///
    /// # Arguments
    ///
    /// * `count`: The maximum number of lobby results to retrieve.
    ///
    pub fn set_count(mut self, count: Option<u64>) -> Self {
        self.count = count;
        self
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

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ComparisonFilter {
    #[default]
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanEqualTo,
    LessThan,
    LessThanEqualTo,
}

impl From<ComparisonFilter> for sys::ELobbyComparison {
    fn from(filter: ComparisonFilter) -> Self {
        match filter {
            ComparisonFilter::Equal => sys::ELobbyComparison::k_ELobbyComparisonEqual,
            ComparisonFilter::NotEqual => sys::ELobbyComparison::k_ELobbyComparisonNotEqual,
            ComparisonFilter::GreaterThan => sys::ELobbyComparison::k_ELobbyComparisonGreaterThan,
            ComparisonFilter::GreaterThanEqualTo => {
                sys::ELobbyComparison::k_ELobbyComparisonEqualToOrGreaterThan
            }
            ComparisonFilter::LessThan => sys::ELobbyComparison::k_ELobbyComparisonLessThan,
            ComparisonFilter::LessThanEqualTo => {
                sys::ELobbyComparison::k_ELobbyComparisonEqualToOrLessThan
            }
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

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ChatEntryType {
    Invalid,
    ChatMsg,
    Typing,
    InviteGame,
    Emote,
    LeftConversation,
    Entered,
    WasKicked,
    WasBanned,
    Disconnected,
    HistoricalChat,
    LinkBlocked,
}

impl From<u8> for ChatEntryType {
    fn from(value: u8) -> Self {
        match value {
            x if x == sys::EChatEntryType::k_EChatEntryTypeInvalid as u8 => ChatEntryType::Invalid,
            x if x == sys::EChatEntryType::k_EChatEntryTypeChatMsg as u8 => ChatEntryType::ChatMsg,
            x if x == sys::EChatEntryType::k_EChatEntryTypeTyping as u8 => ChatEntryType::Typing,
            x if x == sys::EChatEntryType::k_EChatEntryTypeInviteGame as u8 => {
                ChatEntryType::InviteGame
            }
            x if x == sys::EChatEntryType::k_EChatEntryTypeEmote as u8 => ChatEntryType::Emote,
            x if x == sys::EChatEntryType::k_EChatEntryTypeLeftConversation as u8 => {
                ChatEntryType::LeftConversation
            }
            x if x == sys::EChatEntryType::k_EChatEntryTypeEntered as u8 => ChatEntryType::Entered,
            x if x == sys::EChatEntryType::k_EChatEntryTypeWasKicked as u8 => {
                ChatEntryType::WasKicked
            }
            x if x == sys::EChatEntryType::k_EChatEntryTypeWasBanned as u8 => {
                ChatEntryType::WasBanned
            }
            x if x == sys::EChatEntryType::k_EChatEntryTypeDisconnected as u8 => {
                ChatEntryType::Disconnected
            }
            x if x == sys::EChatEntryType::k_EChatEntryTypeHistoricalChat as u8 => {
                ChatEntryType::HistoricalChat
            }
            x if x == sys::EChatEntryType::k_EChatEntryTypeLinkBlocked as u8 => {
                ChatEntryType::LinkBlocked
            }
            _ => ChatEntryType::Invalid,
        }
    }
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

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LobbyChatMsg {
    pub lobby: LobbyId,
    pub user: SteamId,
    pub chat_entry_type: ChatEntryType,
    pub chat_id: i32,
}

unsafe impl Callback for LobbyChatMsg {
    const ID: i32 = 507;
    const SIZE: i32 = ::std::mem::size_of::<sys::LobbyChatMsg_t>() as i32;

    unsafe fn from_raw(raw: *mut c_void) -> Self {
        let val = &mut *(raw as *mut sys::LobbyChatMsg_t);

        println!("Raw: {:?}", val);

        LobbyChatMsg {
            lobby: LobbyId(val.m_ulSteamIDLobby),
            user: SteamId(val.m_ulSteamIDUser),
            chat_entry_type: val.m_eChatEntryType.into(),
            chat_id: val.m_iChatID as i32,
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

    mm.set_lobby_list_filter(LobbyListFilter {
        string: Some(vec![StringFilter(
            LobbyKey::new("name"),
            "My Lobby",
            StringFilterKind::Include,
        )]),
        ..Default::default()
    });

    for _ in 0..100 {
        single.run_callbacks();
        ::std::thread::sleep(::std::time::Duration::from_millis(100));
    }
}
