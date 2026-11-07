use std::collections::HashMap;
use std::mem::{align_of, offset_of, size_of};
use std::process::Command;
use steamworks_sys::*;

/// Compile tests/checked_ugc_layout.cpp with the bundled SDK headers first, then
/// set STEAMWORKS_ABI_PROBE to that executable and run this ignored test.
#[test]
#[ignore = "requires the separately compiled C++ ABI probe"]
fn checked_ugc_native_abi() {
    let probe = std::env::var_os("STEAMWORKS_ABI_PROBE").expect("set STEAMWORKS_ABI_PROBE");
    let output = Command::new(probe).output().expect("run C++ probe");
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    let rows: HashMap<_, _> = text
        .lines()
        .map(|line| {
            let mut cols = line.split_whitespace();
            (
                cols.next().unwrap(),
                cols.map(|c| c.parse::<usize>().unwrap())
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    macro_rules! check {
        ($ty:ty $(, $field:ident)*) => {
            assert_eq!(rows.get(stringify!($ty)).expect(stringify!($ty)).as_slice(),
                &[size_of::<$ty>(),align_of::<$ty>() $(, offset_of!($ty,$field))*], stringify!($ty));
        };
    }
    check!(EResult);
    check!(EWorkshopFileType);
    check!(ERemoteStoragePublishedFileVisibility);
    check!(EItemUpdateStatus);
    check!(bool);
    check!(
        CreateItemResult_t,
        m_eResult,
        m_nPublishedFileId,
        m_bUserNeedsToAcceptWorkshopLegalAgreement
    );
    check!(
        SubmitItemUpdateResult_t,
        m_eResult,
        m_nPublishedFileId,
        m_bUserNeedsToAcceptWorkshopLegalAgreement
    );
    check!(
        RemoteStorageSubscribePublishedFileResult_t,
        m_eResult,
        m_nPublishedFileId
    );
    check!(
        RemoteStorageUnsubscribePublishedFileResult_t,
        m_eResult,
        m_nPublishedFileId
    );
    check!(
        UserFavoriteItemsListChanged_t,
        m_nPublishedFileId,
        m_eResult,
        m_bWasAddRequest
    );
    check!(
        GetUserItemVoteResult_t,
        m_nPublishedFileId,
        m_eResult,
        m_bVotedUp,
        m_bVotedDown,
        m_bVoteSkipped
    );
    check!(
        SetUserItemVoteResult_t,
        m_nPublishedFileId,
        m_eResult,
        m_bVoteUp
    );
    check!(
        DownloadItemResult_t,
        m_unAppID,
        m_nPublishedFileId,
        m_eResult
    );
    check!(
        SteamUGCQueryCompleted_t,
        m_handle,
        m_eResult,
        m_unNumResultsReturned,
        m_unTotalMatchingResults,
        m_bCachedData
    );
    check!(
        SteamUGCDetails_t,
        m_nPublishedFileId,
        m_eResult,
        m_eFileType,
        m_nCreatorAppID,
        m_nConsumerAppID,
        m_rgchTitle,
        m_rgchDescription,
        m_ulSteamIDOwner,
        m_rtimeCreated,
        m_rtimeUpdated,
        m_rtimeAddedToUserList,
        m_eVisibility,
        m_bBanned,
        m_bAcceptedForUse,
        m_bTagsTruncated,
        m_rgchTags,
        m_pchFileName,
        m_nFileSize,
        m_rgchURL,
        m_unVotesUp,
        m_unVotesDown,
        m_flScore,
        m_unNumChildren
    );
}
