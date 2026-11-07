#include <cstddef>
#include <cstdio>
#include <steam/steam_api.h>

#define BEGIN(T) std::printf("%s %zu %zu", #T, sizeof(T), alignof(T))
#define FIELD(T, f) std::printf(" %zu", offsetof(T, f))
#define END() std::puts("")

int main() {
    BEGIN(EResult); END();
    BEGIN(EWorkshopFileType); END();
    BEGIN(ERemoteStoragePublishedFileVisibility); END();
    BEGIN(EItemUpdateStatus); END();
    BEGIN(bool); END();
    BEGIN(CreateItemResult_t); FIELD(CreateItemResult_t, m_eResult); FIELD(CreateItemResult_t, m_nPublishedFileId); FIELD(CreateItemResult_t, m_bUserNeedsToAcceptWorkshopLegalAgreement); END();
    BEGIN(SubmitItemUpdateResult_t); FIELD(SubmitItemUpdateResult_t, m_eResult); FIELD(SubmitItemUpdateResult_t, m_nPublishedFileId); FIELD(SubmitItemUpdateResult_t, m_bUserNeedsToAcceptWorkshopLegalAgreement); END();
    BEGIN(RemoteStorageSubscribePublishedFileResult_t); FIELD(RemoteStorageSubscribePublishedFileResult_t, m_eResult); FIELD(RemoteStorageSubscribePublishedFileResult_t, m_nPublishedFileId); END();
    BEGIN(RemoteStorageUnsubscribePublishedFileResult_t); FIELD(RemoteStorageUnsubscribePublishedFileResult_t, m_eResult); FIELD(RemoteStorageUnsubscribePublishedFileResult_t, m_nPublishedFileId); END();
    BEGIN(UserFavoriteItemsListChanged_t); FIELD(UserFavoriteItemsListChanged_t, m_nPublishedFileId); FIELD(UserFavoriteItemsListChanged_t, m_eResult); FIELD(UserFavoriteItemsListChanged_t, m_bWasAddRequest); END();
    BEGIN(GetUserItemVoteResult_t); FIELD(GetUserItemVoteResult_t, m_nPublishedFileId); FIELD(GetUserItemVoteResult_t, m_eResult); FIELD(GetUserItemVoteResult_t, m_bVotedUp); FIELD(GetUserItemVoteResult_t, m_bVotedDown); FIELD(GetUserItemVoteResult_t, m_bVoteSkipped); END();
    BEGIN(SetUserItemVoteResult_t); FIELD(SetUserItemVoteResult_t, m_nPublishedFileId); FIELD(SetUserItemVoteResult_t, m_eResult); FIELD(SetUserItemVoteResult_t, m_bVoteUp); END();
    BEGIN(DownloadItemResult_t); FIELD(DownloadItemResult_t, m_unAppID); FIELD(DownloadItemResult_t, m_nPublishedFileId); FIELD(DownloadItemResult_t, m_eResult); END();
    BEGIN(SteamUGCQueryCompleted_t); FIELD(SteamUGCQueryCompleted_t, m_handle); FIELD(SteamUGCQueryCompleted_t, m_eResult); FIELD(SteamUGCQueryCompleted_t, m_unNumResultsReturned); FIELD(SteamUGCQueryCompleted_t, m_unTotalMatchingResults); FIELD(SteamUGCQueryCompleted_t, m_bCachedData); END();
    BEGIN(SteamUGCDetails_t);
    FIELD(SteamUGCDetails_t, m_nPublishedFileId); FIELD(SteamUGCDetails_t, m_eResult); FIELD(SteamUGCDetails_t, m_eFileType);
    FIELD(SteamUGCDetails_t, m_nCreatorAppID); FIELD(SteamUGCDetails_t, m_nConsumerAppID); FIELD(SteamUGCDetails_t, m_rgchTitle); FIELD(SteamUGCDetails_t, m_rgchDescription);
    FIELD(SteamUGCDetails_t, m_ulSteamIDOwner); FIELD(SteamUGCDetails_t, m_rtimeCreated); FIELD(SteamUGCDetails_t, m_rtimeUpdated); FIELD(SteamUGCDetails_t, m_rtimeAddedToUserList);
    FIELD(SteamUGCDetails_t, m_eVisibility); FIELD(SteamUGCDetails_t, m_bBanned); FIELD(SteamUGCDetails_t, m_bAcceptedForUse); FIELD(SteamUGCDetails_t, m_bTagsTruncated); FIELD(SteamUGCDetails_t, m_rgchTags);
    FIELD(SteamUGCDetails_t, m_pchFileName); FIELD(SteamUGCDetails_t, m_nFileSize); FIELD(SteamUGCDetails_t, m_rgchURL); FIELD(SteamUGCDetails_t, m_unVotesUp); FIELD(SteamUGCDetails_t, m_unVotesDown); FIELD(SteamUGCDetails_t, m_flScore); FIELD(SteamUGCDetails_t, m_unNumChildren); END();
}
