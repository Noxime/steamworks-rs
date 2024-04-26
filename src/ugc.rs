use super::*;

use std::error;
use std::ffi::{CStr, CString};
use std::fmt;
use std::marker;
use std::mem;
use std::os::raw::c_char;
use std::path::Path;

pub const RESULTS_PER_PAGE: u32 = sys::kNumUGCResultsPerPage as u32;

pub struct UGC<Manager> {
    pub(crate) ugc: *mut sys::ISteamUGC,
    pub(crate) inner: Arc<Inner<Manager>>,
}

const CALLBACK_BASE_ID: i32 = 3400;
const CALLBACK_REMOTE_STORAGE_BASE_ID: i32 = 1300;

// TODO: should come from sys, but I don't think its generated.
#[allow(non_upper_case_globals)]
const UGCQueryHandleInvalid: u64 = 0xffffffffffffffff;

/// Worshop item ID
#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PublishedFileId(pub u64);
impl From<u64> for PublishedFileId {
    fn from(id: u64) -> Self {
        PublishedFileId(id)
    }
}

/// Workshop item types to search for
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
            UGCType::Items => sys::EUGCMatchingUGCType::k_EUGCMatchingUGCType_Items,
            UGCType::ItemsMtx => sys::EUGCMatchingUGCType::k_EUGCMatchingUGCType_Items_Mtx,
            UGCType::ItemsReadyToUse => {
                sys::EUGCMatchingUGCType::k_EUGCMatchingUGCType_Items_ReadyToUse
            }
            UGCType::Collections => sys::EUGCMatchingUGCType::k_EUGCMatchingUGCType_Collections,
            UGCType::Artwork => sys::EUGCMatchingUGCType::k_EUGCMatchingUGCType_Artwork,
            UGCType::Videos => sys::EUGCMatchingUGCType::k_EUGCMatchingUGCType_Videos,
            UGCType::Screenshots => sys::EUGCMatchingUGCType::k_EUGCMatchingUGCType_Screenshots,
            UGCType::AllGuides => sys::EUGCMatchingUGCType::k_EUGCMatchingUGCType_AllGuides,
            UGCType::WebGuides => sys::EUGCMatchingUGCType::k_EUGCMatchingUGCType_WebGuides,
            UGCType::IntegratedGuides => {
                sys::EUGCMatchingUGCType::k_EUGCMatchingUGCType_IntegratedGuides
            }
            UGCType::UsableInGame => sys::EUGCMatchingUGCType::k_EUGCMatchingUGCType_UsableInGame,
            UGCType::ControllerBindings => {
                sys::EUGCMatchingUGCType::k_EUGCMatchingUGCType_ControllerBindings
            }
            UGCType::GameManagedItems => {
                sys::EUGCMatchingUGCType::k_EUGCMatchingUGCType_GameManagedItems
            }
            UGCType::All => sys::EUGCMatchingUGCType::k_EUGCMatchingUGCType_All,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UGCQueryType {
    RankedByVote,
    RankedByPublicationDate,
    AcceptedForGameRankedByAcceptanceDate,
    RankedByTrend,
    FavoritedByFriendsRankedByPublicationDate,
    CreatedByFriendsRankedByPublicationDate,
    RankedByNumTimesReported,
    CreatedByFollowedUsersRankedByPublicationDate,
    NotYetRated,
    RankedByTotalVotesAsc,
    RankedByVotesUp,
    RankedByTextSearch,
    RankedByTotalUniqueSubscriptions,
    RankedByPlaytimeTrend,
    RankedByTotalPlaytime,
    RankedByAveragePlaytimeTrend,
    RankedByLifetimeAveragePlaytime,
    RankedByPlaytimeSessionsTrend,
    RankedByLifetimePlaytimeSessions,
    RankedByLastUpdatedDate,
}
impl Into<sys::EUGCQuery> for UGCQueryType {
    fn into(self) -> sys::EUGCQuery {
        match self {
            UGCQueryType::RankedByVote => sys::EUGCQuery::k_EUGCQuery_RankedByVote,
            UGCQueryType::RankedByPublicationDate => {
                sys::EUGCQuery::k_EUGCQuery_RankedByPublicationDate
            }
            UGCQueryType::AcceptedForGameRankedByAcceptanceDate => {
                sys::EUGCQuery::k_EUGCQuery_AcceptedForGameRankedByAcceptanceDate
            }
            UGCQueryType::RankedByTrend => sys::EUGCQuery::k_EUGCQuery_RankedByTrend,
            UGCQueryType::FavoritedByFriendsRankedByPublicationDate => {
                sys::EUGCQuery::k_EUGCQuery_FavoritedByFriendsRankedByPublicationDate
            }
            UGCQueryType::CreatedByFriendsRankedByPublicationDate => {
                sys::EUGCQuery::k_EUGCQuery_CreatedByFriendsRankedByPublicationDate
            }
            UGCQueryType::RankedByNumTimesReported => {
                sys::EUGCQuery::k_EUGCQuery_RankedByNumTimesReported
            }
            UGCQueryType::CreatedByFollowedUsersRankedByPublicationDate => {
                sys::EUGCQuery::k_EUGCQuery_CreatedByFollowedUsersRankedByPublicationDate
            }
            UGCQueryType::NotYetRated => sys::EUGCQuery::k_EUGCQuery_NotYetRated,
            UGCQueryType::RankedByTotalVotesAsc => {
                sys::EUGCQuery::k_EUGCQuery_RankedByTotalVotesAsc
            }
            UGCQueryType::RankedByVotesUp => sys::EUGCQuery::k_EUGCQuery_RankedByVotesUp,
            UGCQueryType::RankedByTextSearch => sys::EUGCQuery::k_EUGCQuery_RankedByTextSearch,
            UGCQueryType::RankedByTotalUniqueSubscriptions => {
                sys::EUGCQuery::k_EUGCQuery_RankedByTotalUniqueSubscriptions
            }
            UGCQueryType::RankedByPlaytimeTrend => {
                sys::EUGCQuery::k_EUGCQuery_RankedByPlaytimeTrend
            }
            UGCQueryType::RankedByTotalPlaytime => {
                sys::EUGCQuery::k_EUGCQuery_RankedByTotalPlaytime
            }
            UGCQueryType::RankedByAveragePlaytimeTrend => {
                sys::EUGCQuery::k_EUGCQuery_RankedByAveragePlaytimeTrend
            }
            UGCQueryType::RankedByLifetimeAveragePlaytime => {
                sys::EUGCQuery::k_EUGCQuery_RankedByLifetimeAveragePlaytime
            }
            UGCQueryType::RankedByPlaytimeSessionsTrend => {
                sys::EUGCQuery::k_EUGCQuery_RankedByPlaytimeSessionsTrend
            }
            UGCQueryType::RankedByLifetimePlaytimeSessions => {
                sys::EUGCQuery::k_EUGCQuery_RankedByLifetimePlaytimeSessions
            }
            UGCQueryType::RankedByLastUpdatedDate => {
                sys::EUGCQuery::k_EUGCQuery_RankedByLastUpdatedDate
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Community,
    Microtransaction,
    Collection,
    Art,
    Video,
    Screenshot,
    Game,
    Software,
    Concept,
    WebGuide,
    IntegratedGuide,
    Merch,
    ControllerBinding,
    SteamworksAccessInvite,
    SteamVideo,
    GameManagedItem,
}

impl Into<sys::EWorkshopFileType> for FileType {
    fn into(self) -> sys::EWorkshopFileType {
        match self {
            FileType::Community => sys::EWorkshopFileType::k_EWorkshopFileTypeCommunity,
            FileType::Microtransaction => {
                sys::EWorkshopFileType::k_EWorkshopFileTypeMicrotransaction
            }
            FileType::Collection => sys::EWorkshopFileType::k_EWorkshopFileTypeCollection,
            FileType::Art => sys::EWorkshopFileType::k_EWorkshopFileTypeArt,
            FileType::Video => sys::EWorkshopFileType::k_EWorkshopFileTypeVideo,
            FileType::Screenshot => sys::EWorkshopFileType::k_EWorkshopFileTypeScreenshot,
            FileType::Game => sys::EWorkshopFileType::k_EWorkshopFileTypeGame,
            FileType::Software => sys::EWorkshopFileType::k_EWorkshopFileTypeSoftware,
            FileType::Concept => sys::EWorkshopFileType::k_EWorkshopFileTypeConcept,
            FileType::WebGuide => sys::EWorkshopFileType::k_EWorkshopFileTypeWebGuide,
            FileType::IntegratedGuide => sys::EWorkshopFileType::k_EWorkshopFileTypeIntegratedGuide,
            FileType::Merch => sys::EWorkshopFileType::k_EWorkshopFileTypeMerch,
            FileType::ControllerBinding => {
                sys::EWorkshopFileType::k_EWorkshopFileTypeControllerBinding
            }
            FileType::SteamworksAccessInvite => {
                sys::EWorkshopFileType::k_EWorkshopFileTypeSteamworksAccessInvite
            }
            FileType::SteamVideo => sys::EWorkshopFileType::k_EWorkshopFileTypeSteamVideo,
            FileType::GameManagedItem => sys::EWorkshopFileType::k_EWorkshopFileTypeGameManagedItem,
        }
    }
}
impl From<sys::EWorkshopFileType> for FileType {
    fn from(file_type: sys::EWorkshopFileType) -> FileType {
        match file_type {
            sys::EWorkshopFileType::k_EWorkshopFileTypeCommunity => FileType::Community,
            sys::EWorkshopFileType::k_EWorkshopFileTypeMicrotransaction => {
                FileType::Microtransaction
            }
            sys::EWorkshopFileType::k_EWorkshopFileTypeCollection => FileType::Collection,
            sys::EWorkshopFileType::k_EWorkshopFileTypeArt => FileType::Art,
            sys::EWorkshopFileType::k_EWorkshopFileTypeVideo => FileType::Video,
            sys::EWorkshopFileType::k_EWorkshopFileTypeScreenshot => FileType::Screenshot,
            sys::EWorkshopFileType::k_EWorkshopFileTypeGame => FileType::Game,
            sys::EWorkshopFileType::k_EWorkshopFileTypeSoftware => FileType::Software,
            sys::EWorkshopFileType::k_EWorkshopFileTypeConcept => FileType::Concept,
            sys::EWorkshopFileType::k_EWorkshopFileTypeWebGuide => FileType::WebGuide,
            sys::EWorkshopFileType::k_EWorkshopFileTypeIntegratedGuide => FileType::IntegratedGuide,
            sys::EWorkshopFileType::k_EWorkshopFileTypeMerch => FileType::Merch,
            sys::EWorkshopFileType::k_EWorkshopFileTypeControllerBinding => {
                FileType::ControllerBinding
            }
            sys::EWorkshopFileType::k_EWorkshopFileTypeSteamworksAccessInvite => {
                FileType::SteamworksAccessInvite
            }
            sys::EWorkshopFileType::k_EWorkshopFileTypeSteamVideo => FileType::SteamVideo,
            sys::EWorkshopFileType::k_EWorkshopFileTypeGameManagedItem => FileType::GameManagedItem,
            _ => unreachable!(),
        }
    }
}

/// AppID filter for queries.
/// The "consumer" app is the app that the content is for.
/// The "creator" app is a separate editor to create the content in, if applicable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
            UserListOrder::CreationOrderAsc => {
                sys::EUserUGCListSortOrder::k_EUserUGCListSortOrder_CreationOrderAsc
            }
            UserListOrder::CreationOrderDesc => {
                sys::EUserUGCListSortOrder::k_EUserUGCListSortOrder_CreationOrderDesc
            }
            UserListOrder::TitleAsc => sys::EUserUGCListSortOrder::k_EUserUGCListSortOrder_TitleAsc,
            UserListOrder::LastUpdatedDesc => {
                sys::EUserUGCListSortOrder::k_EUserUGCListSortOrder_LastUpdatedDesc
            }
            UserListOrder::SubscriptionDateDesc => {
                sys::EUserUGCListSortOrder::k_EUserUGCListSortOrder_SubscriptionDateDesc
            }
            UserListOrder::VoteScoreDesc => {
                sys::EUserUGCListSortOrder::k_EUserUGCListSortOrder_VoteScoreDesc
            }
            UserListOrder::ForModeration => {
                sys::EUserUGCListSortOrder::k_EUserUGCListSortOrder_ForModeration
            }
        }
    }
}

/// Available user-specific lists.
/// Certain ones are only available to the currently logged in user.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    #[deprecated(note = "Deprecated in Steam API")]
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
            UserList::Published => sys::EUserUGCList::k_EUserUGCList_Published,
            UserList::VotedOn => sys::EUserUGCList::k_EUserUGCList_VotedOn,
            UserList::VotedUp => sys::EUserUGCList::k_EUserUGCList_VotedUp,
            UserList::VotedDown => sys::EUserUGCList::k_EUserUGCList_VotedDown,
            UserList::WillVoteLater => sys::EUserUGCList::k_EUserUGCList_WillVoteLater,
            UserList::Favorited => sys::EUserUGCList::k_EUserUGCList_Favorited,
            UserList::Subscribed => sys::EUserUGCList::k_EUserUGCList_Subscribed,
            UserList::UsedOrPlayed => sys::EUserUGCList::k_EUserUGCList_UsedOrPlayed,
            UserList::Followed => sys::EUserUGCList::k_EUserUGCList_Followed,
        }
    }
}

/// Available published item statistic types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UGCStatisticType {
    /// The number of subscriptions.
    Subscriptions,
    /// The number of favorites.
    Favorites,
    /// The number of followers.
    Followers,
    /// The number of unique subscriptions.
    UniqueSubscriptions,
    /// The number of unique favorites.
    UniqueFavorites,
    /// The number of unique followers.
    UniqueFollowers,
    /// The number of unique views the item has on its Steam Workshop page.
    UniqueWebsiteViews,
    /// The number of times the item has been reported.
    Reports,
    /// The total number of seconds this item has been used across all players.
    SecondsPlayed,
    /// The total number of play sessions this item has been used in.
    PlaytimeSessions,
    /// The number of comments on the items steam has on its Steam Workshop page.
    Comments,
    /// The number of seconds this item has been used over the given time period.
    SecondsPlayedDuringTimePeriod,
    /// The number of sessions this item has been used in over the given time period.
    PlaytimeSessionsDuringTimePeriod,
}
impl Into<sys::EItemStatistic> for UGCStatisticType {
    fn into(self) -> sys::EItemStatistic {
        match self {
            UGCStatisticType::Subscriptions => {
                sys::EItemStatistic::k_EItemStatistic_NumSubscriptions
            }
            UGCStatisticType::Favorites => sys::EItemStatistic::k_EItemStatistic_NumFavorites,
            UGCStatisticType::Followers => sys::EItemStatistic::k_EItemStatistic_NumFollowers,
            UGCStatisticType::UniqueSubscriptions => {
                sys::EItemStatistic::k_EItemStatistic_NumUniqueSubscriptions
            }
            UGCStatisticType::UniqueFavorites => {
                sys::EItemStatistic::k_EItemStatistic_NumUniqueFavorites
            }
            UGCStatisticType::UniqueFollowers => {
                sys::EItemStatistic::k_EItemStatistic_NumUniqueFollowers
            }
            UGCStatisticType::UniqueWebsiteViews => {
                sys::EItemStatistic::k_EItemStatistic_NumUniqueWebsiteViews
            }
            UGCStatisticType::Reports => sys::EItemStatistic::k_EItemStatistic_ReportScore,
            UGCStatisticType::SecondsPlayed => {
                sys::EItemStatistic::k_EItemStatistic_NumSecondsPlayed
            }
            UGCStatisticType::PlaytimeSessions => {
                sys::EItemStatistic::k_EItemStatistic_NumPlaytimeSessions
            }
            UGCStatisticType::Comments => sys::EItemStatistic::k_EItemStatistic_NumComments,
            UGCStatisticType::SecondsPlayedDuringTimePeriod => {
                sys::EItemStatistic::k_EItemStatistic_NumSecondsPlayedDuringTimePeriod
            }
            UGCStatisticType::PlaytimeSessionsDuringTimePeriod => {
                sys::EItemStatistic::k_EItemStatistic_NumPlaytimeSessionsDuringTimePeriod
            }
        }
    }
}

bitflags! {
    pub struct ItemState: u32 {
        const NONE = 0;
        const SUBSCRIBED = 1;
        const LEGACY_ITEM = 2;
        const INSTALLED = 4;
        const NEEDS_UPDATE = 8;
        const DOWNLOADING = 16;
        const DOWNLOAD_PENDING = 32;
    }
}

/// Users can control what user-generated content they want to see under the Mature Content Filtering section in their preferences.
/// This filtering is done automatically by Steam servers, but first, user-generated content must be tagged appropriately.
/// Developers can use AddContentDescriptor and RemoveContentDescriptor calls to manage content descriptors a piece of UGC has.
/// These can be retrieved from the result of a query via GetQueryUGCContentDescriptors.
pub enum UGCContentDescriptorID {
    /// Some Nudity or Sexual Content: Contains content that has some nudity or sexual themes, but not as the primary focus.
    NudityOrSexualContent = 1,
    /// Frequent Violence or Gore: Contains content that features extreme violence or gore.
    FrequentViolenceOrGore = 2,
    /// Adult Only Sexual Content: Contains content that is sexually explicit or graphic and is intended for adults only. Users must affirm that they are at least eighteen years old before they can view content with this content descriptor.
    AdultOnlySexualContent = 3,
    /// Frequent Nudity or Sexual Content: Contains content that primarily features nudity or sexual themes. Users must affirm that they are at least eighteen years old before they can view content with this content descriptor.
    GratuitousSexualContent = 4,
    /// General Mature Content: Contains mature topics that may not be appropriate for all audiences.
    AnyMatureContent = 5,
}
impl Into<sys::EUGCContentDescriptorID> for UGCContentDescriptorID {
    fn into(self) -> sys::EUGCContentDescriptorID {
        match self {
            UGCContentDescriptorID::NudityOrSexualContent => {
                sys::EUGCContentDescriptorID::k_EUGCContentDescriptor_NudityOrSexualContent
            }
            UGCContentDescriptorID::FrequentViolenceOrGore => {
                sys::EUGCContentDescriptorID::k_EUGCContentDescriptor_FrequentViolenceOrGore
            }
            UGCContentDescriptorID::AdultOnlySexualContent => {
                sys::EUGCContentDescriptorID::k_EUGCContentDescriptor_AdultOnlySexualContent
            }
            UGCContentDescriptorID::GratuitousSexualContent => {
                sys::EUGCContentDescriptorID::k_EUGCContentDescriptor_GratuitousSexualContent
            }
            UGCContentDescriptorID::AnyMatureContent => {
                sys::EUGCContentDescriptorID::k_EUGCContentDescriptor_AnyMatureContent
            }
        }
    }
}
impl From<sys::EUGCContentDescriptorID> for UGCContentDescriptorID {
    fn from(content_descriptor_id: sys::EUGCContentDescriptorID) -> UGCContentDescriptorID {
        match content_descriptor_id {
            sys::EUGCContentDescriptorID::k_EUGCContentDescriptor_NudityOrSexualContent => {
                UGCContentDescriptorID::NudityOrSexualContent
            }
            sys::EUGCContentDescriptorID::k_EUGCContentDescriptor_FrequentViolenceOrGore => {
                UGCContentDescriptorID::FrequentViolenceOrGore
            }
            sys::EUGCContentDescriptorID::k_EUGCContentDescriptor_AdultOnlySexualContent => {
                UGCContentDescriptorID::AdultOnlySexualContent
            }
            sys::EUGCContentDescriptorID::k_EUGCContentDescriptor_GratuitousSexualContent => {
                UGCContentDescriptorID::GratuitousSexualContent
            }
            sys::EUGCContentDescriptorID::k_EUGCContentDescriptor_AnyMatureContent => {
                UGCContentDescriptorID::AnyMatureContent
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DownloadItemResult {
    pub app_id: AppId,
    pub published_file_id: PublishedFileId,
    pub error: Option<SteamError>,
}

unsafe impl Callback for DownloadItemResult {
    const ID: i32 = CALLBACK_BASE_ID + 6;
    const SIZE: i32 = ::std::mem::size_of::<sys::DownloadItemResult_t>() as i32;

    unsafe fn from_raw(raw: *mut c_void) -> Self {
        let val = &mut *(raw as *mut sys::DownloadItemResult_t);
        DownloadItemResult {
            app_id: AppId(val.m_unAppID),
            published_file_id: PublishedFileId(val.m_nPublishedFileId),

            error: match val.m_eResult {
                sys::EResult::k_EResultOK => None,
                error => Some(error.into()),
            },
        }
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct InstallInfo {
    pub folder: String,
    pub size_on_disk: u64,
    pub timestamp: u32,
}

impl<Manager> UGC<Manager> {
    /// Suspends or resumes all workshop downloads
    pub fn suspend_downloads(&self, suspend: bool) {
        unsafe {
            sys::SteamAPI_ISteamUGC_SuspendDownloads(self.ugc, suspend);
        }
    }

    /// Creates a workshop item
    pub fn create_item<F>(&self, app_id: AppId, file_type: FileType, cb: F)
    where
        F: FnOnce(Result<(PublishedFileId, bool), SteamError>) + 'static + Send,
    {
        unsafe {
            let api_call = sys::SteamAPI_ISteamUGC_CreateItem(self.ugc, app_id.0, file_type.into());
            register_call_result::<sys::CreateItemResult_t, _, _>(
                &self.inner,
                api_call,
                CALLBACK_BASE_ID + 3,
                move |v, io_error| {
                    cb(if io_error {
                        Err(SteamError::IOFailure)
                    } else if v.m_eResult != sys::EResult::k_EResultOK {
                        Err(v.m_eResult.into())
                    } else {
                        Ok((
                            PublishedFileId(v.m_nPublishedFileId),
                            v.m_bUserNeedsToAcceptWorkshopLegalAgreement,
                        ))
                    })
                },
            );
        }
    }

    /// Starts an item update process
    #[must_use]
    pub fn start_item_update(
        &self,
        app_id: AppId,
        file_id: PublishedFileId,
    ) -> UpdateHandle<Manager> {
        unsafe {
            let handle = sys::SteamAPI_ISteamUGC_StartItemUpdate(self.ugc, app_id.0, file_id.0);
            UpdateHandle {
                ugc: self.ugc,
                inner: self.inner.clone(),

                handle,
            }
        }
    }

    /// Subscribes to a workshop item
    pub fn subscribe_item<F>(&self, published_file_id: PublishedFileId, cb: F)
    where
        F: FnOnce(Result<(), SteamError>) + 'static + Send,
    {
        unsafe {
            let api_call = sys::SteamAPI_ISteamUGC_SubscribeItem(self.ugc, published_file_id.0);
            register_call_result::<sys::RemoteStorageSubscribePublishedFileResult_t, _, _>(
                &self.inner,
                api_call,
                CALLBACK_REMOTE_STORAGE_BASE_ID + 13,
                move |v, io_error| {
                    cb(if io_error {
                        Err(SteamError::IOFailure)
                    } else if v.m_eResult != sys::EResult::k_EResultOK {
                        Err(v.m_eResult.into())
                    } else {
                        Ok(())
                    })
                },
            );
        }
    }

    pub fn unsubscribe_item<F>(&self, published_file_id: PublishedFileId, cb: F)
    where
        F: FnOnce(Result<(), SteamError>) + 'static + Send,
    {
        unsafe {
            let api_call = sys::SteamAPI_ISteamUGC_UnsubscribeItem(self.ugc, published_file_id.0);
            register_call_result::<sys::RemoteStorageUnsubscribePublishedFileResult_t, _, _>(
                &self.inner,
                api_call,
                CALLBACK_REMOTE_STORAGE_BASE_ID + 15,
                move |v, io_error| {
                    cb(if io_error {
                        Err(SteamError::IOFailure)
                    } else if v.m_eResult != sys::EResult::k_EResultOK {
                        Err(v.m_eResult.into())
                    } else {
                        Ok(())
                    })
                },
            );
        }
    }

    /// Gets the publisher file IDs of all currently subscribed items.
    pub fn subscribed_items(&self) -> Vec<PublishedFileId> {
        unsafe {
            let count = sys::SteamAPI_ISteamUGC_GetNumSubscribedItems(self.ugc);
            let mut data: Vec<sys::PublishedFileId_t> = vec![0; count as usize];
            let gotten_count =
                sys::SteamAPI_ISteamUGC_GetSubscribedItems(self.ugc, data.as_mut_ptr(), count);
            debug_assert!(count == gotten_count);
            data.into_iter().map(|v| PublishedFileId(v)).collect()
        }
    }

    pub fn item_state(&self, item: PublishedFileId) -> ItemState {
        unsafe {
            let state = sys::SteamAPI_ISteamUGC_GetItemState(self.ugc, item.0);
            ItemState::from_bits_truncate(state)
        }
    }

    pub fn item_download_info(&self, item: PublishedFileId) -> Option<(u64, u64)> {
        unsafe {
            let mut current = 0u64;
            let mut total = 0u64;
            if sys::SteamAPI_ISteamUGC_GetItemDownloadInfo(
                self.ugc,
                item.0,
                &mut current,
                &mut total,
            ) {
                Some((current, total))
            } else {
                None
            }
        }
    }

    pub fn item_install_info(&self, item: PublishedFileId) -> Option<InstallInfo> {
        unsafe {
            let mut size_on_disk = 0u64;
            let mut folder = [0 as c_char; 4096];
            let mut timestamp = 0u32;
            if sys::SteamAPI_ISteamUGC_GetItemInstallInfo(
                self.ugc,
                item.0,
                &mut size_on_disk,
                folder.as_mut_ptr(),
                folder.len() as _,
                &mut timestamp,
            ) {
                Some(InstallInfo {
                    folder: CStr::from_ptr(folder.as_ptr() as *const _)
                        .to_string_lossy()
                        .into_owned(),
                    size_on_disk,
                    timestamp,
                })
            } else {
                None
            }
        }
    }

    pub fn download_item(&self, item: PublishedFileId, high_priority: bool) -> bool {
        unsafe { sys::SteamAPI_ISteamUGC_DownloadItem(self.ugc, item.0, high_priority) }
    }

    /// Queries a paged list of all workshop items.
    pub fn query_all(
        &self,
        query_type: UGCQueryType,
        item_type: UGCType,
        appids: AppIDs,
        page: u32,
    ) -> Result<QueryHandle<Manager>, CreateQueryError> {
        // Call the external function with the correct parameters
        let handle = unsafe {
            sys::SteamAPI_ISteamUGC_CreateQueryAllUGCRequestPage(
                self.ugc,
                query_type.into(),
                item_type.into(),
                appids.creator_app_id().unwrap_or(AppId(0)).0,
                appids.consumer_app_id().unwrap_or(AppId(0)).0,
                page,
            )
        };

        // Check for an invalid handle
        if handle == UGCQueryHandleInvalid {
            return Err(CreateQueryError);
        }

        // Return a new AllQuery instance
        Ok(QueryHandle {
            ugc: self.ugc,
            inner: Arc::clone(&self.inner),
            handle: Some(handle),
        })
    }

    /// Queries a list of workshop itmes, related to a user in some way (Ex. user's subscriptions, favorites, upvoted, ...)
    pub fn query_user(
        &self,
        account: AccountId,
        list_type: UserList,
        item_type: UGCType,
        sort_order: UserListOrder,
        appids: AppIDs,
        page: u32,
    ) -> Result<QueryHandle<Manager>, CreateQueryError> {
        let res = unsafe {
            sys::SteamAPI_ISteamUGC_CreateQueryUserUGCRequest(
                self.ugc,
                account.0,
                list_type.into(),
                item_type.into(),
                sort_order.into(),
                appids.creator_app_id().unwrap_or(AppId(0)).0,
                appids.consumer_app_id().unwrap_or(AppId(0)).0,
                page,
            )
        };

        if res == UGCQueryHandleInvalid {
            return Err(CreateQueryError);
        }

        Ok(QueryHandle {
            ugc: self.ugc,
            inner: Arc::clone(&self.inner),
            handle: Some(res),
        })
    }

    pub fn query_items(
        &self,
        mut items: Vec<PublishedFileId>,
    ) -> Result<QueryHandle<Manager>, CreateQueryError> {
        debug_assert!(items.len() > 0);

        let res = unsafe {
            sys::SteamAPI_ISteamUGC_CreateQueryUGCDetailsRequest(
                self.ugc,
                items.as_mut_ptr() as _,
                items.len() as _,
            )
        };

        if res == UGCQueryHandleInvalid {
            return Err(CreateQueryError);
        }

        Ok(QueryHandle {
            ugc: self.ugc,
            inner: Arc::clone(&self.inner),
            handle: Some(res),
        })
    }

    pub fn query_item(
        &self,
        item: PublishedFileId,
    ) -> Result<QueryHandle<Manager>, CreateQueryError> {
        let mut items = vec![item];

        let res = unsafe {
            sys::SteamAPI_ISteamUGC_CreateQueryUGCDetailsRequest(
                self.ugc,
                items.as_mut_ptr() as _,
                1 as _,
            )
        };

        if res == UGCQueryHandleInvalid {
            return Err(CreateQueryError);
        }

        Ok(QueryHandle {
            ugc: self.ugc,
            inner: Arc::clone(&self.inner),
            handle: Some(res),
        })
    }

    /// **DELETES** the item from the Steam Workshop.
    pub fn delete_item<F>(&self, published_file_id: PublishedFileId, cb: F)
    where
        F: FnOnce(Result<(), SteamError>) + 'static + Send,
    {
        unsafe {
            let api_call = sys::SteamAPI_ISteamUGC_DeleteItem(self.ugc, published_file_id.0);
            register_call_result::<sys::DownloadItemResult_t, _, _>(
                &self.inner,
                api_call,
                CALLBACK_REMOTE_STORAGE_BASE_ID + 17,
                move |v, io_error| {
                    cb(if io_error {
                        Err(SteamError::IOFailure)
                    } else if v.m_eResult != sys::EResult::k_EResultNone
                        && v.m_eResult != sys::EResult::k_EResultOK
                    {
                        Err(v.m_eResult.into())
                    } else {
                        Ok(())
                    })
                },
            );
        }
    }
}

impl UGC<ServerManager> {
    /// Initialize this UGC interface for a Steam game server.
    ///
    /// You should pass in the Workshop depot, you can find this on SteamDB. It's usually just the app ID.
    ///
    /// The folder is a path to the directory where you wish for this game server to store UGC content.
    ///
    /// `true` upon success; otherwise, `false` if the calling user is not a game server or if the workshop is currently updating its content.
    pub fn init_for_game_server(&self, workshop_depot: sys::DepotId_t, folder: &str) -> bool {
        unsafe {
            let folder = CString::new(folder).unwrap();
            sys::SteamAPI_ISteamUGC_BInitWorkshopForGameServer(
                self.ugc,
                workshop_depot,
                folder.as_ptr(),
            )
        }
    }
}

/// A handle to update a published item
pub struct UpdateHandle<Manager> {
    ugc: *mut sys::ISteamUGC,
    inner: Arc<Inner<Manager>>,

    handle: sys::UGCUpdateHandle_t,
}

impl<Manager> UpdateHandle<Manager> {
    #[must_use]
    pub fn title(self, title: &str) -> Self {
        unsafe {
            let title = CString::new(title).unwrap();
            assert!(sys::SteamAPI_ISteamUGC_SetItemTitle(
                self.ugc,
                self.handle,
                title.as_ptr()
            ));
        }
        self
    }

    #[must_use]
    pub fn description(self, description: &str) -> Self {
        unsafe {
            let description = CString::new(description).unwrap();
            assert!(sys::SteamAPI_ISteamUGC_SetItemDescription(
                self.ugc,
                self.handle,
                description.as_ptr()
            ));
        }
        self
    }

    #[must_use]
    pub fn preview_path(self, path: &Path) -> Self {
        unsafe {
            let path = path.canonicalize().unwrap();
            let preview_path = CString::new(&*path.to_string_lossy()).unwrap();
            assert!(sys::SteamAPI_ISteamUGC_SetItemPreview(
                self.ugc,
                self.handle,
                preview_path.as_ptr()
            ));
        }
        self
    }

    #[must_use]
    pub fn content_path(self, path: &Path) -> Self {
        unsafe {
            let path = path.canonicalize().unwrap();
            let content_path = CString::new(&*path.to_string_lossy()).unwrap();
            assert!(sys::SteamAPI_ISteamUGC_SetItemContent(
                self.ugc,
                self.handle,
                content_path.as_ptr()
            ));
        }
        self
    }

    #[must_use]
    pub fn metadata(self, metadata: &str) -> Self {
        unsafe {
            let metadata = CString::new(metadata).unwrap();
            assert!(sys::SteamAPI_ISteamUGC_SetItemMetadata(
                self.ugc,
                self.handle,
                metadata.as_ptr()
            ));
        }
        self
    }

    pub fn visibility(self, visibility: remote_storage::PublishedFileVisibility) -> Self {
        unsafe {
            assert!(sys::SteamAPI_ISteamUGC_SetItemVisibility(
                self.ugc,
                self.handle,
                visibility.into()
            ));
        }
        self
    }

    pub fn tags<S: AsRef<str>>(self, tags: Vec<S>, allow_admin_tags: bool) -> Self {
        unsafe {
            let mut tags = SteamParamStringArray::new(&tags);
            assert!(sys::SteamAPI_ISteamUGC_SetItemTags(
                self.ugc,
                self.handle,
                &tags.as_raw(),
                allow_admin_tags
            ));
        }
        self
    }

    pub fn add_key_value_tag(self, key: &str, value: &str) -> Self {
        unsafe {
            let key = CString::new(key).unwrap();
            let value = CString::new(value).unwrap();
            assert!(sys::SteamAPI_ISteamUGC_AddItemKeyValueTag(
                self.ugc,
                self.handle,
                key.as_ptr(),
                value.as_ptr()
            ));
        }
        self
    }

    pub fn remove_key_value_tag(self, key: &str) -> Self {
        unsafe {
            let key = CString::new(key).unwrap();
            assert!(sys::SteamAPI_ISteamUGC_RemoveItemKeyValueTags(
                self.ugc,
                self.handle,
                key.as_ptr()
            ));
        }
        self
    }

    pub fn add_content_descriptor(self, desc_id: UGCContentDescriptorID) -> Self {
        unsafe {
            assert!(sys::SteamAPI_ISteamUGC_AddContentDescriptor(
                self.ugc,
                self.handle,
                desc_id.into(),
            ));
        }
        self
    }

    pub fn remove_content_descriptor(self, desc_id: UGCContentDescriptorID) -> Self {
        unsafe {
            assert!(sys::SteamAPI_ISteamUGC_RemoveContentDescriptor(
                self.ugc,
                self.handle,
                desc_id.into()
            ));
        }
        self
    }

    pub fn remove_all_key_value_tags(self) -> Self {
        unsafe {
            assert!(sys::SteamAPI_ISteamUGC_RemoveAllItemKeyValueTags(
                self.ugc,
                self.handle
            ));
        }
        self
    }

    pub fn submit<F>(self, change_note: Option<&str>, cb: F) -> UpdateWatchHandle<Manager>
    where
        F: FnOnce(Result<(PublishedFileId, bool), SteamError>) + 'static + Send,
    {
        use std::ptr;
        unsafe {
            let change_note = change_note.and_then(|v| CString::new(v).ok());
            let note = change_note.as_ref().map_or(ptr::null(), |v| v.as_ptr());
            let api_call = sys::SteamAPI_ISteamUGC_SubmitItemUpdate(self.ugc, self.handle, note);
            register_call_result::<sys::SubmitItemUpdateResult_t, _, _>(
                &self.inner,
                api_call,
                CALLBACK_BASE_ID + 4,
                move |v, io_error| {
                    cb(if io_error {
                        Err(SteamError::IOFailure)
                    } else if v.m_eResult != sys::EResult::k_EResultOK {
                        Err(v.m_eResult.into())
                    } else {
                        Ok((
                            PublishedFileId(v.m_nPublishedFileId),
                            v.m_bUserNeedsToAcceptWorkshopLegalAgreement,
                        ))
                    })
                },
            );
        }
        UpdateWatchHandle {
            ugc: self.ugc,
            _inner: self.inner,
            handle: self.handle,
        }
    }
}

/// A handle to watch an update of a published item
pub struct UpdateWatchHandle<Manager> {
    ugc: *mut sys::ISteamUGC,
    _inner: Arc<Inner<Manager>>,

    handle: sys::UGCUpdateHandle_t,
}

unsafe impl<Manager> Send for UpdateWatchHandle<Manager> {}
unsafe impl<Manager> Sync for UpdateWatchHandle<Manager> {}

impl<Manager> UpdateWatchHandle<Manager> {
    pub fn progress(&self) -> (UpdateStatus, u64, u64) {
        unsafe {
            let mut progress = 0;
            let mut total = 0;
            let status = sys::SteamAPI_ISteamUGC_GetItemUpdateProgress(
                self.ugc,
                self.handle,
                &mut progress,
                &mut total,
            );
            let status = match status {
                sys::EItemUpdateStatus::k_EItemUpdateStatusInvalid => UpdateStatus::Invalid,
                sys::EItemUpdateStatus::k_EItemUpdateStatusPreparingConfig => {
                    UpdateStatus::PreparingConfig
                }
                sys::EItemUpdateStatus::k_EItemUpdateStatusPreparingContent => {
                    UpdateStatus::PreparingContent
                }
                sys::EItemUpdateStatus::k_EItemUpdateStatusUploadingContent => {
                    UpdateStatus::UploadingContent
                }
                sys::EItemUpdateStatus::k_EItemUpdateStatusUploadingPreviewFile => {
                    UpdateStatus::UploadingPreviewFile
                }
                sys::EItemUpdateStatus::k_EItemUpdateStatusCommittingChanges => {
                    UpdateStatus::CommittingChanges
                }
                _ => unreachable!(),
            };
            (status, progress, total)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateStatus {
    Invalid,
    PreparingConfig,
    PreparingContent,
    UploadingContent,
    UploadingPreviewFile,
    CommittingChanges,
}

/// Query handle, to allow for more filtering.
pub struct QueryHandle<Manager> {
    ugc: *mut sys::ISteamUGC,
    inner: Arc<Inner<Manager>>,

    // Note: this is always filled except in `fetch`, where it must be taken
    // to prevent the handle from being dropped when this query is dropped.
    handle: Option<sys::UGCQueryHandle_t>,
}
impl<Manager> Drop for QueryHandle<Manager> {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.as_mut() {
            unsafe {
                sys::SteamAPI_ISteamUGC_ReleaseQueryUGCRequest(self.ugc, *handle);
            }
        }
    }
}
impl<Manager> QueryHandle<Manager> {
    /// Excludes items with a specific tag.
    ///
    /// Panics if `tag` could not be converted to a `CString`.
    pub fn exclude_tag(self, tag: &str) -> Self {
        let cstr = CString::new(tag)
            .expect("String passed to exclude_tag could not be converted to a c string");
        let ok = unsafe {
            sys::SteamAPI_ISteamUGC_AddExcludedTag(self.ugc, self.handle.unwrap(), cstr.as_ptr())
        };
        debug_assert!(ok);
        self
    }

    /// Only include items with a specific tag.
    ///
    /// Panics if `tag` could not be converted to a `CString`.
    pub fn require_tag(self, tag: &str) -> Self {
        let cstr = CString::new(tag)
            .expect("String passed to require_tag could not be converted to a c string");
        let ok = unsafe {
            sys::SteamAPI_ISteamUGC_AddRequiredTag(self.ugc, self.handle.unwrap(), cstr.as_ptr())
        };
        debug_assert!(ok);
        self
    }

    /// Sets how to match tags added by `require_tag`. If `true`, then any tag may match. If `false`, all required tags must match.
    pub fn any_required(self, any: bool) -> Self {
        let ok =
            unsafe { sys::SteamAPI_ISteamUGC_SetMatchAnyTag(self.ugc, self.handle.unwrap(), any) };
        debug_assert!(ok);
        self
    }

    /// Sets the language to return the title and description in for the items on a pending UGC Query.
    ///
    /// Defaults to "english"
    pub fn language(self, language: &str) -> Self {
        let cstr = CString::new(language)
            .expect("String passed to language could not be converted to a c string");
        let ok = unsafe {
            sys::SteamAPI_ISteamUGC_SetLanguage(self.ugc, self.handle.unwrap(), cstr.as_ptr())
        };
        debug_assert!(ok);
        self
    }

    /// Sets whether results will be returned from the cache for the specific period of time on a pending UGC Query.
    ///
    /// Age is in seconds.
    pub fn allow_cached_response(self, max_age_s: u32) -> Self {
        let ok = unsafe {
            sys::SteamAPI_ISteamUGC_SetAllowCachedResponse(
                self.ugc,
                self.handle.unwrap(),
                max_age_s,
            )
        };
        debug_assert!(ok);
        self
    }

    /// Include the full description in results
    pub fn include_long_desc(self, include: bool) -> Self {
        let ok = unsafe {
            sys::SteamAPI_ISteamUGC_SetReturnLongDescription(
                self.ugc,
                self.handle.unwrap(),
                include,
            )
        };
        debug_assert!(ok);
        self
    }

    /// Include children in results
    pub fn include_children(self, include: bool) -> Self {
        let ok = unsafe {
            sys::SteamAPI_ISteamUGC_SetReturnChildren(self.ugc, self.handle.unwrap(), include)
        };
        debug_assert!(ok);
        self
    }

    /// Include metadata in results
    pub fn include_metadata(self, include: bool) -> Self {
        let ok = unsafe {
            sys::SteamAPI_ISteamUGC_SetReturnMetadata(self.ugc, self.handle.unwrap(), include)
        };
        debug_assert!(ok);
        self
    }

    /// Include additional previews in results
    pub fn include_additional_previews(self, include: bool) -> Self {
        let ok = unsafe {
            sys::SteamAPI_ISteamUGC_SetReturnAdditionalPreviews(
                self.ugc,
                self.handle.unwrap(),
                include,
            )
        };
        debug_assert!(ok);
        self
    }

    /// Include key value tags in results
    pub fn include_key_value_tags(self, include: bool) -> Self {
        let ok = unsafe {
            sys::SteamAPI_ISteamUGC_SetReturnKeyValueTags(self.ugc, self.handle.unwrap(), include)
        };
        debug_assert!(ok);
        self
    }

    /// Adds a tag that must be present on all returned items.
    pub fn add_required_tag(self, tag: &str) -> Self {
        let cstr = CString::new(tag).unwrap();
        let ok = unsafe {
            sys::SteamAPI_ISteamUGC_AddRequiredTag(self.ugc, self.handle.unwrap(), cstr.as_ptr())
        };
        debug_assert!(ok);
        self
    }

    /// Adds a tag that must not be present on any returned items.
    pub fn add_excluded_tag(self, tag: &str) -> Self {
        let cstr = CString::new(tag).unwrap();
        let ok = unsafe {
            sys::SteamAPI_ISteamUGC_AddExcludedTag(self.ugc, self.handle.unwrap(), cstr.as_ptr())
        };
        debug_assert!(ok);
        self
    }

    /// Sets whether to only return the IDs of the items.
    pub fn set_return_only_ids(self, return_only_ids: bool) -> Self {
        let ok = unsafe {
            sys::SteamAPI_ISteamUGC_SetReturnOnlyIDs(
                self.ugc,
                self.handle.unwrap(),
                return_only_ids,
            )
        };
        debug_assert!(ok);
        self
    }

    /// Sets whether to return key value tags with the items.
    pub fn set_return_key_value_tags(self, return_kv_tags: bool) -> Self {
        let ok = unsafe {
            sys::SteamAPI_ISteamUGC_SetReturnKeyValueTags(
                self.ugc,
                self.handle.unwrap(),
                return_kv_tags,
            )
        };
        debug_assert!(ok);
        self
    }

    /// Sets whether to return the full description of the items.
    pub fn set_return_long_description(self, return_long_desc: bool) -> Self {
        let ok = unsafe {
            sys::SteamAPI_ISteamUGC_SetReturnLongDescription(
                self.ugc,
                self.handle.unwrap(),
                return_long_desc,
            )
        };
        debug_assert!(ok);
        self
    }

    /// Sets whether to return metadata with the items.
    pub fn set_return_metadata(self, return_metadata: bool) -> Self {
        let ok = unsafe {
            sys::SteamAPI_ISteamUGC_SetReturnMetadata(
                self.ugc,
                self.handle.unwrap(),
                return_metadata,
            )
        };
        debug_assert!(ok);
        self
    }

    /// Sets whether to return children with the items.
    pub fn set_return_children(self, return_children: bool) -> Self {
        let ok = unsafe {
            sys::SteamAPI_ISteamUGC_SetReturnChildren(
                self.ugc,
                self.handle.unwrap(),
                return_children,
            )
        };
        debug_assert!(ok);
        self
    }

    /// Sets whether to return additional previews with the items.
    pub fn set_return_additional_previews(self, return_additional_previews: bool) -> Self {
        let ok = unsafe {
            sys::SteamAPI_ISteamUGC_SetReturnAdditionalPreviews(
                self.ugc,
                self.handle.unwrap(),
                return_additional_previews,
            )
        };
        debug_assert!(ok);
        self
    }

    /// Sets whether to only return the total number of items.
    pub fn set_return_total_only(self, return_total_only: bool) -> Self {
        let ok = unsafe {
            sys::SteamAPI_ISteamUGC_SetReturnTotalOnly(
                self.ugc,
                self.handle.unwrap(),
                return_total_only,
            )
        };
        debug_assert!(ok);
        self
    }

    /// Sets the language to return the title and description in.
    pub fn set_language(self, language: &str) -> Self {
        let cstr = CString::new(language).unwrap();
        let ok = unsafe {
            sys::SteamAPI_ISteamUGC_SetLanguage(self.ugc, self.handle.unwrap(), cstr.as_ptr())
        };
        debug_assert!(ok);
        self
    }

    /// Sets whether results will be returned from the cache.
    pub fn set_allow_cached_response(self, max_age_seconds: u32) -> Self {
        let ok = unsafe {
            sys::SteamAPI_ISteamUGC_SetAllowCachedResponse(
                self.ugc,
                self.handle.unwrap(),
                max_age_seconds,
            )
        };
        debug_assert!(ok);
        self
    }

    /// Sets a filter for the cloud file name.
    pub fn set_cloud_file_name_filter(self, file_name: &str) -> Self {
        let cstr = CString::new(file_name).unwrap();
        let ok = unsafe {
            sys::SteamAPI_ISteamUGC_SetCloudFileNameFilter(
                self.ugc,
                self.handle.unwrap(),
                cstr.as_ptr(),
            )
        };
        debug_assert!(ok);
        self
    }

    /// Sets whether any of the required tags are sufficient for an item to be returned.
    pub fn set_match_any_tag(self, match_any_tag: bool) -> Self {
        let ok = unsafe {
            sys::SteamAPI_ISteamUGC_SetMatchAnyTag(self.ugc, self.handle.unwrap(), match_any_tag)
        };
        debug_assert!(ok);
        self
    }

    /// Sets the full-text search string.
    pub fn set_search_text(self, search_text: &str) -> Self {
        let cstr = CString::new(search_text).unwrap();
        let ok = unsafe {
            sys::SteamAPI_ISteamUGC_SetSearchText(self.ugc, self.handle.unwrap(), cstr.as_ptr())
        };
        debug_assert!(ok);
        self
    }

    /// Sets the number of days to consider for trending items.
    pub fn set_ranked_by_trend_days(self, days: u32) -> Self {
        let ok = unsafe {
            sys::SteamAPI_ISteamUGC_SetRankedByTrendDays(self.ugc, self.handle.unwrap(), days)
        };
        debug_assert!(ok);
        self
    }

    /// Adds a required key-value tag that must be present on all returned items.
    pub fn add_required_key_value_tag(self, key: &str, value: &str) -> Self {
        let key_cstr = CString::new(key).unwrap();
        let value_cstr = CString::new(value).unwrap();
        let ok = unsafe {
            sys::SteamAPI_ISteamUGC_AddRequiredKeyValueTag(
                self.ugc,
                self.handle.unwrap(),
                key_cstr.as_ptr(),
                value_cstr.as_ptr(),
            )
        };
        debug_assert!(ok);
        self
    }

    /// Sends the query to Steam and calls the provided callback with the results when completed.
    pub fn fetch<F>(mut self, cb: F)
    where
        F: for<'a> FnOnce(Result<QueryResults<'a>, SteamError>) + 'static + Send,
    {
        let ugc = self.ugc;
        let inner = Arc::clone(&self.inner);
        let handle = self.handle.take().unwrap();
        mem::drop(self);

        unsafe {
            let api_call = sys::SteamAPI_ISteamUGC_SendQueryUGCRequest(ugc, handle);
            register_call_result::<sys::SteamUGCQueryCompleted_t, _, _>(
                &inner,
                api_call,
                CALLBACK_BASE_ID + 1,
                move |v, io_error| {
                    let ugc = sys::SteamAPI_SteamUGC_v018();
                    if io_error {
                        sys::SteamAPI_ISteamUGC_ReleaseQueryUGCRequest(ugc, handle);
                        cb(Err(SteamError::IOFailure));
                        return;
                    } else if v.m_eResult != sys::EResult::k_EResultOK {
                        sys::SteamAPI_ISteamUGC_ReleaseQueryUGCRequest(ugc, handle);
                        cb(Err(v.m_eResult.into()));
                        return;
                    }

                    let result = QueryResults {
                        ugc,
                        handle,
                        num_results_returned: v.m_unNumResultsReturned,
                        num_results_total: v.m_unTotalMatchingResults,
                        was_cached: v.m_bCachedData,
                        _phantom: Default::default(),
                    };
                    cb(Ok(result));
                },
            );
        }
    }

    /// Runs the query, only fetching the total number of results.
    pub fn fetch_total<F>(self, cb: F)
    where
        F: Fn(Result<u32, SteamError>) + 'static + Send,
    {
        unsafe {
            let ok =
                sys::SteamAPI_ISteamUGC_SetReturnTotalOnly(self.ugc, self.handle.unwrap(), true);
            debug_assert!(ok);
        }

        self.fetch(move |res| cb(res.map(|qr| qr.total_results())))
    }

    /// Runs the query, only fetching the IDs.
    pub fn fetch_ids<F>(self, cb: F)
    where
        F: Fn(Result<Vec<PublishedFileId>, SteamError>) + 'static + Send,
    {
        unsafe {
            let ok = sys::SteamAPI_ISteamUGC_SetReturnOnlyIDs(self.ugc, self.handle.unwrap(), true);
            debug_assert!(ok);
        }

        self.fetch(move |res| {
            cb(res.map(|qr| {
                qr.iter()
                    .filter_map(|v| v.map(|v| PublishedFileId(v.published_file_id.0)))
                    .collect::<Vec<_>>()
            }))
        })
    }
}

/// Query results
pub struct QueryResults<'a> {
    ugc: *mut sys::ISteamUGC,
    handle: sys::UGCQueryHandle_t,
    num_results_returned: u32,
    num_results_total: u32,
    was_cached: bool,
    _phantom: marker::PhantomData<&'a sys::ISteamUGC>,
}
impl<'a> Drop for QueryResults<'a> {
    fn drop(&mut self) {
        unsafe {
            sys::SteamAPI_ISteamUGC_ReleaseQueryUGCRequest(self.ugc, self.handle);
        }
    }
}
impl<'a> QueryResults<'a> {
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

    /// Gets the preview URL of the published file at the specified index.
    pub fn preview_url(&self, index: u32) -> Option<String> {
        let mut url = [0 as c_char; 4096];

        let ok = unsafe {
            sys::SteamAPI_ISteamUGC_GetQueryUGCPreviewURL(
                self.ugc,
                self.handle,
                index,
                url.as_mut_ptr(),
                url.len() as _,
            )
        };

        if ok {
            Some(unsafe {
                CStr::from_ptr(url.as_ptr() as *const _)
                    .to_string_lossy()
                    .into_owned()
            })
        } else {
            None
        }
    }

    /// Gets a UGC statistic about the published file at the specified index.
    pub fn statistic(&self, index: u32, stat_type: UGCStatisticType) -> Option<u64> {
        let mut value = 0u64;

        let ok = unsafe {
            sys::SteamAPI_ISteamUGC_GetQueryUGCStatistic(
                self.ugc,
                self.handle,
                index,
                stat_type.into(),
                &mut value,
            )
        };

        debug_assert!(ok);

        if ok {
            Some(value)
        } else {
            None
        }
    }

    /// Gets UGCContentDescriptors of the published file at the specified index.
    pub fn content_descriptor(&self, index: u32) -> Vec<UGCContentDescriptorID> {
        let mut descriptors: [sys::EUGCContentDescriptorID; 10] = unsafe { std::mem::zeroed() };
        let max_entries = descriptors.len() as std::ffi::c_uint;

        let num_descriptors = unsafe {
            sys::SteamAPI_ISteamUGC_GetQueryUGCContentDescriptors(
                self.ugc,
                self.handle,
                index,
                descriptors.as_mut_ptr(),
                max_entries,
            )
        } as usize;

        Vec::from(&descriptors[..num_descriptors])
            .iter()
            .map(|&x| x.into())
            .collect()
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
            let ok = sys::SteamAPI_ISteamUGC_GetQueryUGCResult(
                self.ugc,
                self.handle,
                index,
                &mut raw_details,
            );
            debug_assert!(ok);

            if raw_details.m_eResult != sys::EResult::k_EResultOK {
                return None;
            }

            let tags = CStr::from_ptr(raw_details.m_rgchTags.as_ptr())
                .to_string_lossy()
                .split(',')
                .map(|s| String::from(s))
                .collect::<Vec<_>>();

            Some(QueryResult {
                published_file_id: PublishedFileId(raw_details.m_nPublishedFileId),
                creator_app_id: if raw_details.m_nCreatorAppID != 0 {
                    Some(AppId(raw_details.m_nCreatorAppID))
                } else {
                    None
                },
                consumer_app_id: if raw_details.m_nConsumerAppID != 0 {
                    Some(AppId(raw_details.m_nConsumerAppID))
                } else {
                    None
                },
                title: CStr::from_ptr(raw_details.m_rgchTitle.as_ptr())
                    .to_string_lossy()
                    .into_owned(),
                description: CStr::from_ptr(raw_details.m_rgchDescription.as_ptr())
                    .to_string_lossy()
                    .into_owned(),
                owner: SteamId(raw_details.m_ulSteamIDOwner),
                time_created: raw_details.m_rtimeCreated,
                time_updated: raw_details.m_rtimeUpdated,
                time_added_to_user_list: raw_details.m_rtimeAddedToUserList,
                visibility: raw_details.m_eVisibility.into(),
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
                file_name: CStr::from_ptr(raw_details.m_pchFileName.as_ptr())
                    .to_string_lossy()
                    .into_owned(),
                file_type: raw_details.m_eFileType.into(),
                file_size: raw_details.m_nFileSize.max(0) as u32,
            })
        }
    }

    /// Returns an iterator that runs over all the fetched results
    pub fn iter<'b>(&'b self) -> impl Iterator<Item = Option<QueryResult>> + 'b {
        (0..self.returned_results()).map(move |i| self.get(i))
    }

    /// Returns the given index's children as a list of PublishedFileId.
    ///
    /// You must call `include_children(true)` before fetching the query for this to work.
    ///
    /// Returns None if the index was out of bounds.
    pub fn get_children(&self, index: u32) -> Option<Vec<PublishedFileId>> {
        let num_children = self.get(index)?.num_children;
        let mut children: Vec<sys::PublishedFileId_t> = vec![0; num_children as usize];

        let ok = unsafe {
            sys::SteamAPI_ISteamUGC_GetQueryUGCChildren(
                self.ugc,
                self.handle,
                index,
                children.as_mut_ptr(),
                num_children,
            )
        };

        if ok {
            Some(children.into_iter().map(Into::into).collect())
        } else {
            None
        }
    }

    /// Returns the number of key value tags associated with the item at the specified index.
    pub fn key_value_tags(&self, index: u32) -> u32 {
        unsafe { sys::SteamAPI_ISteamUGC_GetQueryUGCNumKeyValueTags(self.ugc, self.handle, index) }
    }

    /// Gets the key value pair of a specified key value tag associated with the item at the specified index.
    pub fn get_key_value_tag(&self, index: u32, kv_tag_index: u32) -> Option<(String, String)> {
        let mut key = [0 as c_char; 256];
        let mut value = [0 as c_char; 256];

        let ok = unsafe {
            sys::SteamAPI_ISteamUGC_GetQueryUGCKeyValueTag(
                self.ugc,
                self.handle,
                index,
                kv_tag_index,
                key.as_mut_ptr(),
                256,
                value.as_mut_ptr(),
                256,
            )
        };

        if ok {
            Some(unsafe {
                (
                    CStr::from_ptr(key.as_ptr() as *const _)
                        .to_string_lossy()
                        .into_owned(),
                    CStr::from_ptr(value.as_ptr() as *const _)
                        .to_string_lossy()
                        .into_owned(),
                )
            })
        } else {
            None
        }
    }

    /// Gets the developer-set metadata associated with the item at the specified index.
    ///
    /// This is returned as a vector of raw bytes.
    pub fn get_metadata(&self, index: u32) -> Option<Vec<u8>> {
        let mut metadata = [0 as c_char; sys::k_cchDeveloperMetadataMax as usize];

        let ok = unsafe {
            sys::SteamAPI_ISteamUGC_GetQueryUGCMetadata(
                self.ugc,
                self.handle,
                index,
                metadata.as_mut_ptr(),
                sys::k_cchDeveloperMetadataMax,
            )
        };

        if ok {
            let metadata = unsafe { CStr::from_ptr(metadata.as_ptr() as *const _).to_bytes() };
            if metadata.is_empty() {
                None
            } else {
                Some(metadata.to_vec())
            }
        } else {
            None
        }
    }
}

/// Query result
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub published_file_id: PublishedFileId,
    pub creator_app_id: Option<AppId>,
    pub consumer_app_id: Option<AppId>,
    pub title: String,
    pub description: String,
    pub owner: SteamId,
    /// Time created in unix epoch seconds format
    pub time_created: u32,
    /// Time updated in unix epoch seconds format
    pub time_updated: u32,
    /// Time when the user added the published item to their list (not always applicable), provided in Unix epoch format (time since Jan 1st, 1970).
    pub time_added_to_user_list: u32,
    pub visibility: PublishedFileVisibility,
    pub banned: bool,
    pub accepted_for_use: bool,
    pub tags: Vec<String>,
    pub tags_truncated: bool,
    /// Original file name of the workshop item. Used in old games, like Total War: Shogun 2.
    pub file_name: String,
    pub file_type: FileType,
    pub file_size: u32,

    pub url: String,
    pub num_upvotes: u32,
    pub num_downvotes: u32,
    /// The bayesian average for up votes / total votes, between \[0,1\].
    pub score: f32,
    pub num_children: u32,
    // TODO: Add missing fields as needed
}

#[derive(Debug, Clone, Copy)]
pub struct CreateQueryError;
impl fmt::Display for CreateQueryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Could not create workshop query")
    }
}
impl error::Error for CreateQueryError {}
