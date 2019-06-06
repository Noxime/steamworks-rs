
use super::*;

use std::ffi::{CStr, CString};
use std::mem;
use std::fmt;
use std::error;

pub struct UGC<Manager> {
    pub(crate) ugc: *mut sys::ISteamUGC,
    pub(crate) inner: Arc<Inner<Manager>>,
}

const CALLBACK_BASE_ID: i32 = 3400;
const CALLBACK_REMOTE_STORAGE_BASE_ID: i32 = 1300;

// TODO: should come from sys, but I don't think its generated.
#[allow(non_upper_case_globals)]
const UGCQueryHandleInvalid: u64 = 0xffffffffffffffff;

/// Workshop item types to search for
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum UGCType {
    Items,
    ItemsMtx,
    ItemsReadyToUse,
    Collections,
    Artwork,
    Videos,
    Screenshots,
    AllGuides,
    WebGuides,
    IntegratedGuides,
    UsableInGame,
    ControllerBindings,
    GameManagedItems,
    All,
}
impl Into<sys::EUGCMatchingUGCType> for UGCType {
    fn into(self) -> sys::EUGCMatchingUGCType {
        match self {
            UGCType::Items => sys::EUGCMatchingUGCType::EUGCMatchingUGCType_Items,
            UGCType::ItemsMtx => sys::EUGCMatchingUGCType::EUGCMatchingUGCType_Items_Mtx,
            UGCType::ItemsReadyToUse => sys::EUGCMatchingUGCType::EUGCMatchingUGCType_Items_ReadyToUse,
            UGCType::Collections => sys::EUGCMatchingUGCType::EUGCMatchingUGCType_Collections,
            UGCType::Artwork => sys::EUGCMatchingUGCType::EUGCMatchingUGCType_Artwork,
            UGCType::Videos => sys::EUGCMatchingUGCType::EUGCMatchingUGCType_Videos,
            UGCType::Screenshots => sys::EUGCMatchingUGCType::EUGCMatchingUGCType_Screenshots,
            UGCType::AllGuides => sys::EUGCMatchingUGCType::EUGCMatchingUGCType_AllGuides,
            UGCType::WebGuides => sys::EUGCMatchingUGCType::EUGCMatchingUGCType_WebGuides,
            UGCType::IntegratedGuides => sys::EUGCMatchingUGCType::EUGCMatchingUGCType_IntegratedGuides,
            UGCType::UsableInGame => sys::EUGCMatchingUGCType::EUGCMatchingUGCType_UsableInGame,
            UGCType::ControllerBindings => sys::EUGCMatchingUGCType::EUGCMatchingUGCType_ControllerBindings,
            UGCType::GameManagedItems => sys::EUGCMatchingUGCType::EUGCMatchingUGCType_GameManagedItems,
            UGCType::All => sys::EUGCMatchingUGCType::EUGCMatchingUGCType_All,
        }
    }
}

/// AppID filter for queries.
/// The "consumer" app is the app that the content is for.
/// The "creator" app is a separate editor to create the content in, if applicable.
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum AppIDs {
    CreatorAppId(AppId),
    ConsumerAppId(AppId),
    Both { creator: AppId, consumer: AppId },
}
impl AppIDs {
    pub fn creator_app_id(&self) -> Option<AppId> {
        match self {
            AppIDs::CreatorAppId(v) => Some(*v),
            AppIDs::ConsumerAppId(_) => None,
            AppIDs::Both { creator, .. } => Some(*creator),
        }
    }
    pub fn consumer_app_id(&self) -> Option<AppId> {
        match self {
            AppIDs::CreatorAppId(_) => None,
            AppIDs::ConsumerAppId(v) => Some(*v),
            AppIDs::Both { consumer, .. } => Some(*consumer),
        }
    }
}

/// Query result sorting
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum UserListOrder {
    CreationOrderAsc,
    CreationOrderDesc,
    TitleAsc,
    LastUpdatedDesc,
    SubscriptionDateDesc,
    VoteScoreDesc,
    ForModeration,
}
impl Into<sys::EUserUGCListSortOrder> for UserListOrder {
    fn into(self) -> sys::EUserUGCListSortOrder {
        match self {
            UserListOrder::CreationOrderAsc => sys::EUserUGCListSortOrder::EUserUGCListSortOrder_CreationOrderAsc,
            UserListOrder::CreationOrderDesc => sys::EUserUGCListSortOrder::EUserUGCListSortOrder_CreationOrderDesc,
            UserListOrder::TitleAsc => sys::EUserUGCListSortOrder::EUserUGCListSortOrder_TitleAsc,
            UserListOrder::LastUpdatedDesc => sys::EUserUGCListSortOrder::EUserUGCListSortOrder_LastUpdatedDesc,
            UserListOrder::SubscriptionDateDesc => sys::EUserUGCListSortOrder::EUserUGCListSortOrder_SubscriptionDateDesc,
            UserListOrder::VoteScoreDesc => sys::EUserUGCListSortOrder::EUserUGCListSortOrder_VoteScoreDesc,
            UserListOrder::ForModeration => sys::EUserUGCListSortOrder::EUserUGCListSortOrder_ForModeration,
        }
    }
}

/// Available user-specific lists.
/// Certain ones are only available to the currently logged in user.
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum UserList {
    /// Files user has published
    Published,
    /// Files user has voted on
    VotedOn,
    /// Files user has voted up (current user only)
    VotedUp,
    /// Files user has voted down (current user only)
    VotedDown,
    /// Deprecated
    #[deprecated(note="Deprecated in Steam API")]
    WillVoteLater,
    /// Files user has favorited
    Favorited,
    /// Files user has subscribed to (current user only)
    Subscribed,
    /// Files user has spent in-game time with
    UsedOrPlayed,
    /// Files user is following updates for
    Followed,
}
impl Into<sys::EUserUGCList> for UserList {
    #[allow(deprecated)]
    fn into(self) -> sys::EUserUGCList {
        match self {
            UserList::Published => sys::EUserUGCList::EUserUGCList_Published,
            UserList::VotedOn => sys::EUserUGCList::EUserUGCList_VotedOn,
            UserList::VotedUp => sys::EUserUGCList::EUserUGCList_VotedUp,
            UserList::VotedDown => sys::EUserUGCList::EUserUGCList_VotedDown,
            UserList::WillVoteLater => sys::EUserUGCList::EUserUGCList_WillVoteLater,
            UserList::Favorited => sys::EUserUGCList::EUserUGCList_Favorited,
            UserList::Subscribed => sys::EUserUGCList::EUserUGCList_Subscribed,
            UserList::UsedOrPlayed => sys::EUserUGCList::EUserUGCList_UsedOrPlayed,
            UserList::Followed => sys::EUserUGCList::EUserUGCList_Followed,
        }
    }
}



impl <Manager> UGC<Manager> {
    /// Suspends or resumes all workshop downloads
    pub fn suspend_download(&self, suspend: bool) {
        unsafe {
            sys::SteamAPI_ISteamUGC_SuspendDownload(self.ugc, suspend);
        }
    }

    /// Subscribes to a workshop item
    pub fn subscribe_item<F>(&self, published_file_id: u64, mut cb: F)
        where F: FnMut(Result<(), SteamError>) + 'static + Send + Sync
    {
        unsafe {
            let api_call = sys::SteamAPI_ISteamUGC_SubscribeItem(self.ugc, published_file_id);
            register_call_result::<sys::RemoteStorageSubscribePublishedFileResult_t, _, _>(
                &self.inner, api_call, CALLBACK_REMOTE_STORAGE_BASE_ID + 13,
                move |v, io_error| {
                    cb(if io_error {
                        Err(SteamError::IOFailure)
                    } else if v.m_eResult != sys::EResult::EResultOK {
                        Err(v.m_eResult.into())
                    } else {
                        Ok(())
                    })
            });
        }
    }

    pub fn unsubscribe_item<F>(&self, published_file_id: u64, mut cb: F)
        where F: FnMut(Result<(), SteamError>) + 'static + Send + Sync
    {
        unsafe {
            let api_call = sys::SteamAPI_ISteamUGC_UnsubscribeItem(self.ugc, published_file_id);
            register_call_result::<sys::RemoteStorageUnsubscribePublishedFileResult_t, _, _>(
                &self.inner, api_call, CALLBACK_REMOTE_STORAGE_BASE_ID + 15,
                move |v, io_error| {
                    cb(if io_error {
                        Err(SteamError::IOFailure)
                    } else if v.m_eResult != sys::EResult::EResultOK {
                        Err(v.m_eResult.into())
                    } else {
                        Ok(())
                    })
            });
        }
    }

    /// Gets the publisher file IDs of all currently subscribed items.
    pub fn subscribed_items(&self) -> Vec<u64> {
        unsafe {
            let count = sys::SteamAPI_ISteamUGC_GetNumSubscribedItems(self.ugc);
            let mut vec: Vec<u64> = vec![0; count as usize];
            let gotten_count = sys::SteamAPI_ISteamUGC_GetSubscribedItems(self.ugc, vec.as_mut_ptr(), count);
            debug_assert!(count == gotten_count);
            vec
        }
    }

    /// Queries a list of workshop itmes, related to a user in some way (Ex. user's subscriptions, favorites, upvoted, ...)
    pub fn query_user(&self,
        account: u32,
        list_type: UserList,
        item_type: UGCType,
        sort_order: UserListOrder,
        appids: AppIDs,
        page: u32
    ) -> Result<UserListQuery<Manager>, CreateQueryError> {
        unsafe {
            let res = sys::SteamAPI_ISteamUGC_CreateQueryUserUGCRequest(
                self.ugc,
                sys::AccountID_t(account),
                list_type.into(),
                item_type.into(),
                sort_order.into(),
                sys::AppId_t(appids.creator_app_id().unwrap_or(AppId(0)).0),
                sys::AppId_t(appids.consumer_app_id().unwrap_or(AppId(0)).0),
                page,
            );
            if res.0 == UGCQueryHandleInvalid {
                return Err(CreateQueryError);
            }

            Ok(UserListQuery {
                ugc: self.ugc,
                inner: Arc::clone(&self.inner),
                handle: res,
            })
        }
    }
}



/// Query object from `query_user`, to allow for more filtering.
pub struct UserListQuery<Manager> {
    ugc: *mut sys::ISteamUGC,
    inner: Arc<Inner<Manager>>,
    handle: sys::UGCQueryHandle_t,
}
impl <Manager> Drop for UserListQuery<Manager> {
    fn drop(&mut self) {
        unsafe {
            sys::SteamAPI_ISteamUGC_ReleaseQueryUGCRequest(self.ugc, self.handle);
        }
    }
}
impl <Manager> UserListQuery<Manager> {
    /// Excludes items with a specific tag.
    ///
    /// Panics if `tag` could not be converted to a `CString`.
    pub fn exclude_tag(self, tag: &str) -> Self {
        let cstr = CString::new(tag).expect("String passed to exclude_tag could not be converted to a c string");
        let ok = unsafe {
            sys::SteamAPI_ISteamUGC_AddExcludedTag(self.ugc, self.handle, cstr.as_ptr())
        };
        debug_assert!(ok);
        self
    }

    /// Only include items with a specific tag.
    ///
    /// Panics if `tag` could not be converted to a `CString`.
    pub fn require_tag(self, tag: &str) -> Self {
        let cstr = CString::new(tag).expect("String passed to require_tag could not be converted to a c string");
        let ok = unsafe {
            sys::SteamAPI_ISteamUGC_AddRequiredTag(self.ugc, self.handle, cstr.as_ptr())
        };
        debug_assert!(ok);
        self
    }

    /// Sets how to match tags added by `require_tag`. If `true`, then any tag may match. If `false`, all required tags must match.
    pub fn any_required(self, any: bool) -> Self {
        let ok = unsafe {
            sys::SteamAPI_ISteamUGC_SetMatchAnyTag(self.ugc, self.handle, any)
        };
        debug_assert!(ok);
        self
    }

    /// Sets the language to return the title and description in for the items on a pending UGC Query.
    ///
    /// Defaults to "english"
    pub fn language(self, language: &str) -> Self {
        let cstr = CString::new(language).expect("String passed to language could not be converted to a c string");
        let ok = unsafe {
            sys::SteamAPI_ISteamUGC_SetLanguage(self.ugc, self.handle, cstr.as_ptr())
        };
        debug_assert!(ok);
        self
    }

    /// Sets whether results will be returned from the cache for the specific period of time on a pending UGC Query.
    ///
    /// Age is in seconds.
    pub fn allow_cached_response(self, max_age_s: u32) -> Self {
        let ok = unsafe {
            sys::SteamAPI_ISteamUGC_SetAllowCachedResponse(self.ugc, self.handle, max_age_s)
        };
        debug_assert!(ok);
        self
    }

    /// Include the full description in results
    pub fn include_long_desc(self, include: bool) -> Self {
        let ok = unsafe {
            sys::SteamAPI_ISteamUGC_SetReturnLongDescription(self.ugc, self.handle, include)
        };
        debug_assert!(ok);
        self
    }

    /// Include children in results
    pub fn include_children(self, include: bool) -> Self {
        let ok = unsafe {
            sys::SteamAPI_ISteamUGC_SetReturnChildren(self.ugc, self.handle, include)
        };
        debug_assert!(ok);
        self
    }

    /// Include metadata in results
    pub fn include_metadata(self, include: bool) -> Self {
        let ok = unsafe {
            sys::SteamAPI_ISteamUGC_SetReturnMetadata(self.ugc, self.handle, include)
        };
        debug_assert!(ok);
        self
    }

    /// Include additional previews in results
    pub fn include_additional_previews(self, include: bool) -> Self {
        let ok = unsafe {
            sys::SteamAPI_ISteamUGC_SetReturnAdditionalPreviews(self.ugc, self.handle, include)
        };
        debug_assert!(ok);
        self
    }

    /// Runs the query
    pub fn fetch<F>(self, mut cb: F)
        where F: FnMut(Result<QueryResults,SteamError>) + 'static + Send
    {
        let ugc = self.ugc;
        let inner = Arc::clone(&self.inner);
        let handle = self.handle;
        mem::forget(self); // Don't run destructor since we need the handle still

        unsafe {
            let api_call = sys::SteamAPI_ISteamUGC_SendQueryUGCRequest(ugc, handle);
            register_call_result::<sys::SteamUGCQueryCompleted_t, _, _>(
                &inner, api_call, CALLBACK_BASE_ID + 1,
                move |v, io_error| {
                    if io_error {
                        cb(Err(SteamError::IOFailure));
                        return;
                    } else if v.m_eResult != sys::EResult::EResultOK {
                        cb(Err(v.m_eResult.into()));
                        return;
                    }

                    let result = QueryResults {
                        ugc: sys::steam_rust_get_ugc(),
                        handle,
                        num_results_returned: v.m_unNumResultsReturned,
                        num_results_total: v.m_unTotalMatchingResults,
                        was_cached: v.m_bCachedData,
                    };
                    cb(Ok(result));
            });
        }
    }

    /// Runs the query, only fetching the total number of results.
    pub fn fetch_total<F>(self, mut cb: F)
        where F: FnMut(Result<u32, SteamError>) + 'static + Send
    {
        unsafe {
            let ok = sys::SteamAPI_ISteamUGC_SetReturnTotalOnly(self.ugc, self.handle, true);
            debug_assert!(ok);
        }

        self.fetch(move |res| cb(res.map(|qr| qr.total_results())))
    }

    /// Runs the query, only fetchind the IDs.
    pub fn fetch_ids<F>(self, mut cb: F)
        where F: FnMut(Result<Vec<u64>, SteamError>) + 'static + Send
    {
        unsafe {
            let ok = sys::SteamAPI_ISteamUGC_SetReturnOnlyIDs(self.ugc, self.handle, true);
            debug_assert!(ok);
        }

        self.fetch(move |res| cb(res.map(|qr| qr.iter().map(|v| v.published_file_id).collect::<Vec<_>>())))
    }
}

/// Query results
pub struct QueryResults {
    ugc: *mut sys::ISteamUGC,
    handle: sys::UGCQueryHandle_t,
    num_results_returned: u32,
    num_results_total: u32,
    was_cached: bool,
}
impl Drop for QueryResults {
    fn drop(&mut self) {
        unsafe {
            sys::SteamAPI_ISteamUGC_ReleaseQueryUGCRequest(self.ugc, self.handle);
        }
    }
}
impl QueryResults {
    /// Were these results retreived from a cache?
    pub fn was_cached(&self) -> bool {
        self.was_cached
    }

    /// Gets the total number of results in this query, not just the current page
    pub fn total_results(&self) -> u32 {
        self.num_results_total
    }

    /// Gets the number of results in this page.
    pub fn returned_results(&self) -> u32 {
        self.num_results_returned
    }

    /// Gets a result.
    ///
    /// Returns None if index was out of bounds.
    pub fn get(&self, index: u32) -> Option<QueryResult> {
        if index >= self.num_results_returned {
            return None;
        }

        unsafe {
            let mut raw_details: sys::SteamUGCDetails_t = mem::zeroed();
            let ok = sys::SteamAPI_ISteamUGC_GetQueryUGCResult(self.ugc, self.handle, index, &mut raw_details);
            debug_assert!(ok);

            // TODO: is this always true? we don't get this from an async call...
            debug_assert!(raw_details.m_eResult == sys::EResult::EResultOK);

            let tags = CStr::from_ptr(raw_details.m_rgchTags.as_ptr())
                .to_string_lossy()
                .split(',')
                .map(|s| String::from(s))
                .collect::<Vec<_>>();

            Some(QueryResult {
                published_file_id: raw_details.m_nPublishedFileId.0,
                creator_app_id: if raw_details.m_nCreatorAppID.0 != 0 { Some(AppId(raw_details.m_nCreatorAppID.0)) } else { None },
                consumer_app_id: if raw_details.m_nConsumerAppID.0 != 0 { Some(AppId(raw_details.m_nConsumerAppID.0)) } else { None },
                title: CStr::from_ptr(raw_details.m_rgchTitle.as_ptr())
                    .to_string_lossy()
                    .into_owned(),
                description: CStr::from_ptr(raw_details.m_rgchDescription.as_ptr())
                    .to_string_lossy()
                    .into_owned(),
                owner: SteamId(raw_details.m_ulSteamIDOwner),
                time_created: raw_details.m_rtimeCreated,
                time_updated: raw_details.m_rtimeUpdated,
                banned: raw_details.m_bBanned,
                accepted_for_use: raw_details.m_bAcceptedForUse,
                url: CStr::from_ptr(raw_details.m_rgchURL.as_ptr())
                    .to_string_lossy()
                    .into_owned(),
                num_upvotes: raw_details.m_unVotesUp,
                num_downvotes: raw_details.m_unVotesDown,
                score: raw_details.m_flScore,
                num_children: raw_details.m_unNumChildren,
                tags,
                tags_truncated: raw_details.m_bTagsTruncated,
            })
        }
    }

    /// Returns an iterator that runs over all the fetched results
    pub fn iter<'a>(&'a self) -> impl Iterator<Item=QueryResult> + 'a {
        (0..self.returned_results())
            .map(move |i| self.get(i).unwrap())
    }
}

/// Query result
#[derive(Debug,Clone)]
pub struct QueryResult {
    pub published_file_id: u64,
    pub creator_app_id: Option<AppId>,
    pub consumer_app_id: Option<AppId>,
    pub title: String,
    pub description: String,
    pub owner: SteamId,
    /// Time created in unix epoch seconds format
    pub time_created: u32,
    /// Time updated in unix epoch seconds format
    pub time_updated: u32,
    pub banned: bool,
    pub accepted_for_use: bool,
    pub tags: Vec<String>,
    pub tags_truncated: bool,

    pub url: String,
    pub num_upvotes: u32,
    pub num_downvotes: u32,
    /// The bayesian average for up votes / total votes, between [0,1].
    pub score: f32,
    pub num_children: u32,

    // TODO: Add missing fields as needed
}

#[derive(Debug,Clone,Copy)]
pub struct CreateQueryError;
impl fmt::Display for CreateQueryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Could not create workshop query")
    }
}
impl error::Error for CreateQueryError {}
