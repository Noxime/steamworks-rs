//! Read only wire-compatible primitives before interpreting native enums/bools.
//! The dispatcher validates the callback ID and size before calling these readers.
use super::*;

macro_rules! field {
    ($p:expr, $ty:ty, $name:ident, $wire:ty) => {
        $p.add(offset_of!($ty, $name))
            .cast::<$wire>()
            .read_unaligned()
    };
}

fn flag(value: u8) -> Result<bool, UgcError> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(UgcError::InvalidPayload("boolean")),
    }
}

pub(super) fn text(chars: &[c_char], field: &'static str) -> Result<String, UgcError> {
    let end = chars
        .iter()
        .position(|&c| c == 0)
        .ok_or(UgcError::InvalidPayload(field))?;
    let bytes: Vec<u8> = chars[..end].iter().map(|&c| c as u8).collect();
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

pub(super) unsafe fn created(p: *const u8) -> Result<(PublishedFileId, bool), UgcError> {
    let item = PublishedFileId(field!(p, sys::CreateItemResult_t, m_nPublishedFileId, u64));
    status(
        field!(p, sys::CreateItemResult_t, m_eResult, u32),
        Some(item),
    )?;
    Ok((
        item,
        flag(field!(
            p,
            sys::CreateItemResult_t,
            m_bUserNeedsToAcceptWorkshopLegalAgreement,
            u8
        ))?,
    ))
}

pub(super) unsafe fn submitted(p: *const u8) -> Result<(PublishedFileId, bool), UgcError> {
    let item = PublishedFileId(field!(
        p,
        sys::SubmitItemUpdateResult_t,
        m_nPublishedFileId,
        u64
    ));
    status(
        field!(p, sys::SubmitItemUpdateResult_t, m_eResult, u32),
        Some(item),
    )?;
    Ok((
        item,
        flag(field!(
            p,
            sys::SubmitItemUpdateResult_t,
            m_bUserNeedsToAcceptWorkshopLegalAgreement,
            u8
        ))?,
    ))
}

pub(super) unsafe fn subscribed(p: *const u8) -> Result<PublishedFileId, UgcError> {
    let item = PublishedFileId(field!(
        p,
        sys::RemoteStorageSubscribePublishedFileResult_t,
        m_nPublishedFileId,
        u64
    ));
    status(
        field!(
            p,
            sys::RemoteStorageSubscribePublishedFileResult_t,
            m_eResult,
            u32
        ),
        Some(item),
    )?;
    Ok(item)
}

pub(super) unsafe fn unsubscribed(p: *const u8) -> Result<PublishedFileId, UgcError> {
    let item = PublishedFileId(field!(
        p,
        sys::RemoteStorageUnsubscribePublishedFileResult_t,
        m_nPublishedFileId,
        u64
    ));
    status(
        field!(
            p,
            sys::RemoteStorageUnsubscribePublishedFileResult_t,
            m_eResult,
            u32
        ),
        Some(item),
    )?;
    Ok(item)
}

pub(super) unsafe fn favorite(p: *const u8) -> Result<FavoriteChangeResult, UgcError> {
    let item = PublishedFileId(field!(
        p,
        sys::UserFavoriteItemsListChanged_t,
        m_nPublishedFileId,
        u64
    ));
    status(
        field!(p, sys::UserFavoriteItemsListChanged_t, m_eResult, u32),
        Some(item),
    )?;
    Ok(FavoriteChangeResult {
        published_file_id: item,
        was_add_request: flag(field!(
            p,
            sys::UserFavoriteItemsListChanged_t,
            m_bWasAddRequest,
            u8
        ))?,
    })
}

pub(super) unsafe fn vote(p: *const u8) -> Result<UserItemVoteResult, UgcError> {
    let item = PublishedFileId(field!(
        p,
        sys::GetUserItemVoteResult_t,
        m_nPublishedFileId,
        u64
    ));
    status(
        field!(p, sys::GetUserItemVoteResult_t, m_eResult, u32),
        Some(item),
    )?;
    Ok(UserItemVoteResult {
        published_file_id: item,
        voted_up: flag(field!(p, sys::GetUserItemVoteResult_t, m_bVotedUp, u8))?,
        voted_down: flag(field!(p, sys::GetUserItemVoteResult_t, m_bVotedDown, u8))?,
        vote_skipped: flag(field!(p, sys::GetUserItemVoteResult_t, m_bVoteSkipped, u8))?,
    })
}

pub(super) unsafe fn set_vote(p: *const u8) -> Result<SetUserItemVoteResult, UgcError> {
    let item = PublishedFileId(field!(
        p,
        sys::SetUserItemVoteResult_t,
        m_nPublishedFileId,
        u64
    ));
    status(
        field!(p, sys::SetUserItemVoteResult_t, m_eResult, u32),
        Some(item),
    )?;
    Ok(SetUserItemVoteResult {
        published_file_id: item,
        vote_up: flag(field!(p, sys::SetUserItemVoteResult_t, m_bVoteUp, u8))?,
    })
}

pub(super) unsafe fn query(p: *const u8, expected: u64) -> Result<(u32, u32, bool), UgcError> {
    if field!(p, sys::SteamUGCQueryCompleted_t, m_handle, u64) != expected {
        return Err(UgcError::InvalidPayload("query handle"));
    }
    status(
        field!(p, sys::SteamUGCQueryCompleted_t, m_eResult, u32),
        None,
    )?;
    let returned = field!(
        p,
        sys::SteamUGCQueryCompleted_t,
        m_unNumResultsReturned,
        u32
    );
    Ok((
        returned,
        field!(
            p,
            sys::SteamUGCQueryCompleted_t,
            m_unTotalMatchingResults,
            u32
        ),
        flag(field!(p, sys::SteamUGCQueryCompleted_t, m_bCachedData, u8))?,
    ))
}

pub(super) unsafe fn download(p: *const u8) -> DownloadItemResult {
    let code = field!(p, sys::DownloadItemResult_t, m_eResult, u32);
    DownloadItemResult {
        app_id: AppId(field!(p, sys::DownloadItemResult_t, m_unAppID, u32)),
        published_file_id: PublishedFileId(field!(
            p,
            sys::DownloadItemResult_t,
            m_nPublishedFileId,
            u64
        )),
        error: status(code, None).err().map(UgcError::into_legacy),
    }
}

pub(super) unsafe fn details(p: *const u8) -> Result<QueryResult, UgcError> {
    let item = PublishedFileId(field!(p, sys::SteamUGCDetails_t, m_nPublishedFileId, u64));
    status(
        field!(p, sys::SteamUGCDetails_t, m_eResult, u32),
        Some(item),
    )?;
    let file_type = match field!(p, sys::SteamUGCDetails_t, m_eFileType, u32) {
        0 => FileType::Community,
        1 => FileType::Microtransaction,
        2 => FileType::Collection,
        3 => FileType::Art,
        4 => FileType::Video,
        5 => FileType::Screenshot,
        6 => FileType::Game,
        7 => FileType::Software,
        8 => FileType::Concept,
        9 => FileType::WebGuide,
        10 => FileType::IntegratedGuide,
        11 => FileType::Merch,
        12 => FileType::ControllerBinding,
        13 => FileType::SteamworksAccessInvite,
        14 => FileType::SteamVideo,
        15 => FileType::GameManagedItem,
        16 => FileType::Clip,
        value => {
            return Err(UgcError::UnknownValue {
                field: "file type",
                value,
            })
        }
    };
    let visibility = match field!(p, sys::SteamUGCDetails_t, m_eVisibility, u32) {
        0 => PublishedFileVisibility::Public,
        1 => PublishedFileVisibility::FriendsOnly,
        2 => PublishedFileVisibility::Private,
        3 => PublishedFileVisibility::Unlisted,
        value => {
            return Err(UgcError::UnknownValue {
                field: "visibility",
                value,
            })
        }
    };
    let creator = field!(p, sys::SteamUGCDetails_t, m_nCreatorAppID, u32);
    let consumer = field!(p, sys::SteamUGCDetails_t, m_nConsumerAppID, u32);
    Ok(QueryResult {
        published_file_id: item,
        creator_app_id: (creator != 0).then_some(AppId(creator)),
        consumer_app_id: (consumer != 0).then_some(AppId(consumer)),
        title: text(
            &field!(p, sys::SteamUGCDetails_t, m_rgchTitle, [c_char; 129]),
            "title",
        )?,
        description: text(
            &field!(p, sys::SteamUGCDetails_t, m_rgchDescription, [c_char; 8000]),
            "description",
        )?,
        owner: SteamId(field!(p, sys::SteamUGCDetails_t, m_ulSteamIDOwner, u64)),
        time_created: field!(p, sys::SteamUGCDetails_t, m_rtimeCreated, u32),
        time_updated: field!(p, sys::SteamUGCDetails_t, m_rtimeUpdated, u32),
        time_added_to_user_list: field!(p, sys::SteamUGCDetails_t, m_rtimeAddedToUserList, u32),
        visibility,
        banned: flag(field!(p, sys::SteamUGCDetails_t, m_bBanned, u8))?,
        accepted_for_use: flag(field!(p, sys::SteamUGCDetails_t, m_bAcceptedForUse, u8))?,
        tags: text(
            &field!(p, sys::SteamUGCDetails_t, m_rgchTags, [c_char; 1025]),
            "tags",
        )?
        .split(',')
        .map(str::to_owned)
        .collect(),
        tags_truncated: flag(field!(p, sys::SteamUGCDetails_t, m_bTagsTruncated, u8))?,
        file_name: text(
            &field!(p, sys::SteamUGCDetails_t, m_pchFileName, [c_char; 260]),
            "file name",
        )?,
        file_type,
        file_size: field!(p, sys::SteamUGCDetails_t, m_nFileSize, i32).max(0) as u32,
        url: text(
            &field!(p, sys::SteamUGCDetails_t, m_rgchURL, [c_char; 256]),
            "url",
        )?,
        num_upvotes: field!(p, sys::SteamUGCDetails_t, m_unVotesUp, u32),
        num_downvotes: field!(p, sys::SteamUGCDetails_t, m_unVotesDown, u32),
        score: field!(p, sys::SteamUGCDetails_t, m_flScore, f32),
        num_children: field!(p, sys::SteamUGCDetails_t, m_unNumChildren, u32),
    })
}

// A u32 return is intentional: receiving an unknown value as a Rust enum would
// already be invalid before a match could handle it. Same C symbol and ABI.
extern "C" {
    #[link_name = "SteamAPI_ISteamUGC_GetItemUpdateProgress"]
    pub(super) fn update_progress_raw(
        ugc: *mut sys::ISteamUGC,
        handle: u64,
        processed: *mut u64,
        total: *mut u64,
    ) -> u32;
}

pub(super) fn progress(value: u32) -> Result<UpdateStatus, UgcError> {
    match value {
        0 => Ok(UpdateStatus::Invalid),
        1 => Ok(UpdateStatus::PreparingConfig),
        2 => Ok(UpdateStatus::PreparingContent),
        3 => Ok(UpdateStatus::UploadingContent),
        4 => Ok(UpdateStatus::UploadingPreviewFile),
        5 => Ok(UpdateStatus::CommittingChanges),
        value => Err(UgcError::UnknownValue {
            field: "update status",
            value,
        }),
    }
}

#[cfg(test)]
mod checked_ugc_wire_tests {
    use super::*;

    fn write<T: Copy>(bytes: &mut [u8], offset: usize, value: T) {
        assert!(offset + size_of::<T>() <= bytes.len());
        unsafe {
            bytes
                .as_mut_ptr()
                .add(offset)
                .cast::<T>()
                .write_unaligned(value);
        }
    }

    #[test]
    fn unknown_result_is_read_as_integer_and_retains_returned_id() {
        let mut bytes = vec![0; size_of::<sys::CreateItemResult_t>()];
        write(
            &mut bytes,
            offset_of!(sys::CreateItemResult_t, m_eResult),
            0xffffu32,
        );
        write(
            &mut bytes,
            offset_of!(sys::CreateItemResult_t, m_nPublishedFileId),
            9123u64,
        );
        assert!(matches!(
            unsafe { created(bytes.as_ptr()) },
            Err(UgcError::Steam {
                code: 65535,
                published_file_id: Some(PublishedFileId(9123)),
                known_error: None
            })
        ));
    }

    #[test]
    fn vote_flags_and_returned_id_are_not_derived_from_the_request() {
        let mut bytes = vec![0; size_of::<sys::GetUserItemVoteResult_t>()];
        write(
            &mut bytes,
            offset_of!(sys::GetUserItemVoteResult_t, m_eResult),
            1u32,
        );
        write(
            &mut bytes,
            offset_of!(sys::GetUserItemVoteResult_t, m_nPublishedFileId),
            32u64,
        );
        write(
            &mut bytes,
            offset_of!(sys::GetUserItemVoteResult_t, m_bVotedDown),
            1u8,
        );
        let r = unsafe { vote(bytes.as_ptr()) }.unwrap();
        assert_eq!(r.published_file_id, PublishedFileId(32));
        assert!(r.voted_down && !r.voted_up && !r.vote_skipped);
        write(
            &mut bytes,
            offset_of!(sys::GetUserItemVoteResult_t, m_bVotedUp),
            2u8,
        );
        assert!(matches!(
            unsafe { vote(bytes.as_ptr()) },
            Err(UgcError::InvalidPayload("boolean"))
        ));
    }

    #[test]
    fn query_rejects_wrong_handle_and_keeps_counts_and_cache() {
        let mut bytes = vec![0; size_of::<sys::SteamUGCQueryCompleted_t>()];
        write(
            &mut bytes,
            offset_of!(sys::SteamUGCQueryCompleted_t, m_handle),
            42u64,
        );
        write(
            &mut bytes,
            offset_of!(sys::SteamUGCQueryCompleted_t, m_eResult),
            1u32,
        );
        write(
            &mut bytes,
            offset_of!(sys::SteamUGCQueryCompleted_t, m_unNumResultsReturned),
            2u32,
        );
        write(
            &mut bytes,
            offset_of!(sys::SteamUGCQueryCompleted_t, m_unTotalMatchingResults),
            53u32,
        );
        write(
            &mut bytes,
            offset_of!(sys::SteamUGCQueryCompleted_t, m_bCachedData),
            1u8,
        );
        assert_eq!(unsafe { query(bytes.as_ptr(), 42) }.unwrap(), (2, 53, true));
        assert!(unsafe { query(bytes.as_ptr(), 43) }.is_err());
    }

    #[test]
    fn full_details_reject_unknown_enum_without_constructing_it() {
        let mut bytes = vec![0; size_of::<sys::SteamUGCDetails_t>()];
        write(
            &mut bytes,
            offset_of!(sys::SteamUGCDetails_t, m_eResult),
            1u32,
        );
        write(
            &mut bytes,
            offset_of!(sys::SteamUGCDetails_t, m_eFileType),
            0xffffu32,
        );
        assert!(matches!(
            unsafe { details(bytes.as_ptr()) },
            Err(UgcError::UnknownValue {
                field: "file type",
                value: 65535
            })
        ));
        write(
            &mut bytes,
            offset_of!(sys::SteamUGCDetails_t, m_eFileType),
            0u32,
        );
        write(
            &mut bytes,
            offset_of!(sys::SteamUGCDetails_t, m_eVisibility),
            99u32,
        );
        assert!(matches!(
            unsafe { details(bytes.as_ptr()) },
            Err(UgcError::UnknownValue {
                field: "visibility",
                value: 99
            })
        ));
    }

    #[test]
    fn text_is_bounded_and_lossy_like_existing_display_fields() {
        assert!(text(&[65, 66], "text").is_err());
        assert_eq!(text(&[0xffu8 as c_char, 0], "text").unwrap(), "\u{fffd}");
    }

    #[test]
    fn result_none_and_unknown_progress_are_failures() {
        assert!(status(0, None).is_err());
        assert!(progress(77).is_err());
    }

    #[test]
    fn subscription_and_creation_payloads_keep_native_ids_and_legal_flag() {
        let mut bytes = vec![0; size_of::<sys::RemoteStorageSubscribePublishedFileResult_t>()];
        write(
            &mut bytes,
            offset_of!(sys::RemoteStorageSubscribePublishedFileResult_t, m_eResult),
            1u32,
        );
        write(
            &mut bytes,
            offset_of!(
                sys::RemoteStorageSubscribePublishedFileResult_t,
                m_nPublishedFileId
            ),
            500u64,
        );
        assert_eq!(
            unsafe { subscribed(bytes.as_ptr()) }.unwrap(),
            PublishedFileId(500)
        );
        let mut bytes = vec![0; size_of::<sys::RemoteStorageUnsubscribePublishedFileResult_t>()];
        write(
            &mut bytes,
            offset_of!(
                sys::RemoteStorageUnsubscribePublishedFileResult_t,
                m_eResult
            ),
            1u32,
        );
        write(
            &mut bytes,
            offset_of!(
                sys::RemoteStorageUnsubscribePublishedFileResult_t,
                m_nPublishedFileId
            ),
            501u64,
        );
        assert_eq!(
            unsafe { unsubscribed(bytes.as_ptr()) }.unwrap(),
            PublishedFileId(501)
        );
        let mut bytes = vec![0; size_of::<sys::SubmitItemUpdateResult_t>()];
        write(
            &mut bytes,
            offset_of!(sys::SubmitItemUpdateResult_t, m_eResult),
            1u32,
        );
        write(
            &mut bytes,
            offset_of!(sys::SubmitItemUpdateResult_t, m_nPublishedFileId),
            502u64,
        );
        write(
            &mut bytes,
            offset_of!(
                sys::SubmitItemUpdateResult_t,
                m_bUserNeedsToAcceptWorkshopLegalAgreement
            ),
            1u8,
        );
        assert_eq!(
            unsafe { submitted(bytes.as_ptr()) }.unwrap(),
            (PublishedFileId(502), true)
        );
    }

    #[test]
    fn favorites_and_written_votes_keep_direction() {
        let mut bytes = vec![0; size_of::<sys::UserFavoriteItemsListChanged_t>()];
        write(
            &mut bytes,
            offset_of!(sys::UserFavoriteItemsListChanged_t, m_eResult),
            1u32,
        );
        write(
            &mut bytes,
            offset_of!(sys::UserFavoriteItemsListChanged_t, m_nPublishedFileId),
            17u64,
        );
        write(
            &mut bytes,
            offset_of!(sys::UserFavoriteItemsListChanged_t, m_bWasAddRequest),
            1u8,
        );
        let r = unsafe { favorite(bytes.as_ptr()) }.unwrap();
        assert_eq!(r.published_file_id, PublishedFileId(17));
        assert!(r.was_add_request);
        let mut bytes = vec![0; size_of::<sys::SetUserItemVoteResult_t>()];
        write(
            &mut bytes,
            offset_of!(sys::SetUserItemVoteResult_t, m_eResult),
            1u32,
        );
        write(
            &mut bytes,
            offset_of!(sys::SetUserItemVoteResult_t, m_nPublishedFileId),
            18u64,
        );
        assert_eq!(
            unsafe { set_vote(bytes.as_ptr()) }.unwrap(),
            SetUserItemVoteResult {
                published_file_id: PublishedFileId(18),
                vote_up: false
            }
        );
    }

    #[test]
    fn download_unknown_status_is_an_error_not_invalid_enum_or_success() {
        let mut bytes = vec![0; size_of::<sys::DownloadItemResult_t>()];
        write(
            &mut bytes,
            offset_of!(sys::DownloadItemResult_t, m_eResult),
            0xffffu32,
        );
        write(
            &mut bytes,
            offset_of!(sys::DownloadItemResult_t, m_nPublishedFileId),
            22u64,
        );
        let r = unsafe { download(bytes.as_ptr()) };
        assert_eq!(r.published_file_id, PublishedFileId(22));
        assert!(r.error.is_some());
    }

    #[test]
    fn valid_details_decode_and_nonterminated_title_fails_in_bounds() {
        let mut bytes = vec![0; size_of::<sys::SteamUGCDetails_t>()];
        write(
            &mut bytes,
            offset_of!(sys::SteamUGCDetails_t, m_eResult),
            1u32,
        );
        write(
            &mut bytes,
            offset_of!(sys::SteamUGCDetails_t, m_nPublishedFileId),
            27u64,
        );
        let r = unsafe { details(bytes.as_ptr()) }.unwrap();
        assert_eq!(r.published_file_id, PublishedFileId(27));
        let title = offset_of!(sys::SteamUGCDetails_t, m_rgchTitle);
        bytes[title..title + 129].fill(65);
        assert!(matches!(
            unsafe { details(bytes.as_ptr()) },
            Err(UgcError::InvalidPayload("title"))
        ));
    }
}
