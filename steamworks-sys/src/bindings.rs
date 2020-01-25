
    use libc::*;
    
    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct Salt_t(pub [u8; 8]);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct GID_t(pub u64);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct JobID_t(pub u64);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct TxnID_t(pub GID_t);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct PackageId_t(pub u32);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct BundleId_t(pub u32);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct AppId_t(pub u32);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct AssetClassId_t(pub u64);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct PhysicalItemId_t(pub u32);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct DepotId_t(pub u32);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct RTime32(pub u32);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct CellID_t(pub u32);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct SteamAPICall_t(pub u64);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct AccountID_t(pub u32);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct PartnerId_t(pub u32);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct ManifestId_t(pub u64);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct SiteId_t(pub u64);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct HAuthTicket(pub u32);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct BREAKPAD_HANDLE(pub *mut c_void);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct HSteamPipe(pub i32);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct HSteamUser(pub i32);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct FriendsGroupID_t(pub i16);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct HServerListRequest(pub *mut c_void);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct HServerQuery(pub c_int);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct UGCHandle_t(pub u64);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct PublishedFileUpdateHandle_t(pub u64);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct PublishedFileId_t(pub u64);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct UGCFileWriteStreamHandle_t(pub u64);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct SteamLeaderboard_t(pub u64);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct SteamLeaderboardEntries_t(pub u64);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct SNetSocket_t(pub u32);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct SNetListenSocket_t(pub u32);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct ScreenshotHandle(pub u32);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct HTTPRequestHandle(pub u32);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct HTTPCookieContainerHandle(pub u32);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct ControllerHandle_t(pub u64);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct ControllerActionSetHandle_t(pub u64);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct ControllerDigitalActionHandle_t(pub u64);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct ControllerAnalogActionHandle_t(pub u64);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct UGCQueryHandle_t(pub u64);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct UGCUpdateHandle_t(pub u64);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct HHTMLBrowser(pub u32);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct SteamItemInstanceID_t(pub u64);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct SteamItemDef_t(pub i32);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct SteamInventoryResult_t(pub i32);

    #[repr(transparent)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct SteamInventoryUpdateHandle_t(pub u64);

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EUniverse {
    EUniverseInvalid = 0,
    EUniversePublic = 1,
    EUniverseBeta = 2,
    EUniverseInternal = 3,
    EUniverseDev = 4,
    EUniverseMax = 5,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EResult {
    EResultOK = 1,
    EResultFail = 2,
    EResultNoConnection = 3,
    EResultInvalidPassword = 5,
    EResultLoggedInElsewhere = 6,
    EResultInvalidProtocolVer = 7,
    EResultInvalidParam = 8,
    EResultFileNotFound = 9,
    EResultBusy = 10,
    EResultInvalidState = 11,
    EResultInvalidName = 12,
    EResultInvalidEmail = 13,
    EResultDuplicateName = 14,
    EResultAccessDenied = 15,
    EResultTimeout = 16,
    EResultBanned = 17,
    EResultAccountNotFound = 18,
    EResultInvalidSteamID = 19,
    EResultServiceUnavailable = 20,
    EResultNotLoggedOn = 21,
    EResultPending = 22,
    EResultEncryptionFailure = 23,
    EResultInsufficientPrivilege = 24,
    EResultLimitExceeded = 25,
    EResultRevoked = 26,
    EResultExpired = 27,
    EResultAlreadyRedeemed = 28,
    EResultDuplicateRequest = 29,
    EResultAlreadyOwned = 30,
    EResultIPNotFound = 31,
    EResultPersistFailed = 32,
    EResultLockingFailed = 33,
    EResultLogonSessionReplaced = 34,
    EResultConnectFailed = 35,
    EResultHandshakeFailed = 36,
    EResultIOFailure = 37,
    EResultRemoteDisconnect = 38,
    EResultShoppingCartNotFound = 39,
    EResultBlocked = 40,
    EResultIgnored = 41,
    EResultNoMatch = 42,
    EResultAccountDisabled = 43,
    EResultServiceReadOnly = 44,
    EResultAccountNotFeatured = 45,
    EResultAdministratorOK = 46,
    EResultContentVersion = 47,
    EResultTryAnotherCM = 48,
    EResultPasswordRequiredToKickSession = 49,
    EResultAlreadyLoggedInElsewhere = 50,
    EResultSuspended = 51,
    EResultCancelled = 52,
    EResultDataCorruption = 53,
    EResultDiskFull = 54,
    EResultRemoteCallFailed = 55,
    EResultPasswordUnset = 56,
    EResultExternalAccountUnlinked = 57,
    EResultPSNTicketInvalid = 58,
    EResultExternalAccountAlreadyLinked = 59,
    EResultRemoteFileConflict = 60,
    EResultIllegalPassword = 61,
    EResultSameAsPreviousValue = 62,
    EResultAccountLogonDenied = 63,
    EResultCannotUseOldPassword = 64,
    EResultInvalidLoginAuthCode = 65,
    EResultAccountLogonDeniedNoMail = 66,
    EResultHardwareNotCapableOfIPT = 67,
    EResultIPTInitError = 68,
    EResultParentalControlRestricted = 69,
    EResultFacebookQueryError = 70,
    EResultExpiredLoginAuthCode = 71,
    EResultIPLoginRestrictionFailed = 72,
    EResultAccountLockedDown = 73,
    EResultAccountLogonDeniedVerifiedEmailRequired = 74,
    EResultNoMatchingURL = 75,
    EResultBadResponse = 76,
    EResultRequirePasswordReEntry = 77,
    EResultValueOutOfRange = 78,
    EResultUnexpectedError = 79,
    EResultDisabled = 80,
    EResultInvalidCEGSubmission = 81,
    EResultRestrictedDevice = 82,
    EResultRegionLocked = 83,
    EResultRateLimitExceeded = 84,
    EResultAccountLoginDeniedNeedTwoFactor = 85,
    EResultItemDeleted = 86,
    EResultAccountLoginDeniedThrottle = 87,
    EResultTwoFactorCodeMismatch = 88,
    EResultTwoFactorActivationCodeMismatch = 89,
    EResultAccountAssociatedToMultiplePartners = 90,
    EResultNotModified = 91,
    EResultNoMobileDevice = 92,
    EResultTimeNotSynced = 93,
    EResultSmsCodeFailed = 94,
    EResultAccountLimitExceeded = 95,
    EResultAccountActivityLimitExceeded = 96,
    EResultPhoneActivityLimitExceeded = 97,
    EResultRefundToWallet = 98,
    EResultEmailSendFailure = 99,
    EResultNotSettled = 100,
    EResultNeedCaptcha = 101,
    EResultGSLTDenied = 102,
    EResultGSOwnerDenied = 103,
    EResultInvalidItemType = 104,
    EResultIPBanned = 105,
    EResultGSLTExpired = 106,
    EResultInsufficientFunds = 107,
    EResultTooManyPending = 108,
    EResultNoSiteLicensesFound = 109,
    EResultWGNetworkSendExceeded = 110,
    EResultAccountNotFriends = 111,
    EResultLimitedUserAccount = 112,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EVoiceResult {
    EVoiceResultOK = 0,
    EVoiceResultNotInitialized = 1,
    EVoiceResultNotRecording = 2,
    EVoiceResultNoData = 3,
    EVoiceResultBufferTooSmall = 4,
    EVoiceResultDataCorrupted = 5,
    EVoiceResultRestricted = 6,
    EVoiceResultUnsupportedCodec = 7,
    EVoiceResultReceiverOutOfDate = 8,
    EVoiceResultReceiverDidNotAnswer = 9,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EDenyReason {
    EDenyInvalid = 0,
    EDenyInvalidVersion = 1,
    EDenyGeneric = 2,
    EDenyNotLoggedOn = 3,
    EDenyNoLicense = 4,
    EDenyCheater = 5,
    EDenyLoggedInElseWhere = 6,
    EDenyUnknownText = 7,
    EDenyIncompatibleAnticheat = 8,
    EDenyMemoryCorruption = 9,
    EDenyIncompatibleSoftware = 10,
    EDenySteamConnectionLost = 11,
    EDenySteamConnectionError = 12,
    EDenySteamResponseTimedOut = 13,
    EDenySteamValidationStalled = 14,
    EDenySteamOwnerLeftGuestUser = 15,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EBeginAuthSessionResult {
    EBeginAuthSessionResultOK = 0,
    EBeginAuthSessionResultInvalidTicket = 1,
    EBeginAuthSessionResultDuplicateRequest = 2,
    EBeginAuthSessionResultInvalidVersion = 3,
    EBeginAuthSessionResultGameMismatch = 4,
    EBeginAuthSessionResultExpiredTicket = 5,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EAuthSessionResponse {
    EAuthSessionResponseOK = 0,
    EAuthSessionResponseUserNotConnectedToSteam = 1,
    EAuthSessionResponseNoLicenseOrExpired = 2,
    EAuthSessionResponseVACBanned = 3,
    EAuthSessionResponseLoggedInElseWhere = 4,
    EAuthSessionResponseVACCheckTimedOut = 5,
    EAuthSessionResponseAuthTicketCanceled = 6,
    EAuthSessionResponseAuthTicketInvalidAlreadyUsed = 7,
    EAuthSessionResponseAuthTicketInvalid = 8,
    EAuthSessionResponsePublisherIssuedBan = 9,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EUserHasLicenseForAppResult {
    EUserHasLicenseResultHasLicense = 0,
    EUserHasLicenseResultDoesNotHaveLicense = 1,
    EUserHasLicenseResultNoAuth = 2,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EAccountType {
    EAccountTypeInvalid = 0,
    EAccountTypeIndividual = 1,
    EAccountTypeMultiseat = 2,
    EAccountTypeGameServer = 3,
    EAccountTypeAnonGameServer = 4,
    EAccountTypePending = 5,
    EAccountTypeContentServer = 6,
    EAccountTypeClan = 7,
    EAccountTypeChat = 8,
    EAccountTypeConsoleUser = 9,
    EAccountTypeAnonUser = 10,
    EAccountTypeMax = 11,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EAppReleaseState {
    EAppReleaseState_Unknown = 0,
    EAppReleaseState_Unavailable = 1,
    EAppReleaseState_Prerelease = 2,
    EAppReleaseState_PreloadOnly = 3,
    EAppReleaseState_Released = 4,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EAppOwnershipFlags {
    EAppOwnershipFlags_None = 0,
    EAppOwnershipFlags_OwnsLicense = 1,
    EAppOwnershipFlags_FreeLicense = 2,
    EAppOwnershipFlags_RegionRestricted = 4,
    EAppOwnershipFlags_LowViolence = 8,
    EAppOwnershipFlags_InvalidPlatform = 16,
    EAppOwnershipFlags_SharedLicense = 32,
    EAppOwnershipFlags_FreeWeekend = 64,
    EAppOwnershipFlags_RetailLicense = 128,
    EAppOwnershipFlags_LicenseLocked = 256,
    EAppOwnershipFlags_LicensePending = 512,
    EAppOwnershipFlags_LicenseExpired = 1024,
    EAppOwnershipFlags_LicensePermanent = 2048,
    EAppOwnershipFlags_LicenseRecurring = 4096,
    EAppOwnershipFlags_LicenseCanceled = 8192,
    EAppOwnershipFlags_AutoGrant = 16384,
    EAppOwnershipFlags_PendingGift = 32768,
    EAppOwnershipFlags_RentalNotActivated = 65536,
    EAppOwnershipFlags_Rental = 131072,
    EAppOwnershipFlags_SiteLicense = 262144,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EAppType {
    EAppType_Invalid = 0,
    EAppType_Game = 1,
    EAppType_Application = 2,
    EAppType_Tool = 4,
    EAppType_Demo = 8,
    EAppType_Media_DEPRECATED = 16,
    EAppType_DLC = 32,
    EAppType_Guide = 64,
    EAppType_Driver = 128,
    EAppType_Config = 256,
    EAppType_Hardware = 512,
    EAppType_Franchise = 1024,
    EAppType_Video = 2048,
    EAppType_Plugin = 4096,
    EAppType_Music = 8192,
    EAppType_Series = 16384,
    EAppType_Comic = 32768,
    EAppType_Shortcut = 1073741824,
    EAppType_DepotOnly = -2147483648,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum ESteamUserStatType {
    ESteamUserStatTypeINVALID = 0,
    ESteamUserStatTypeINT = 1,
    ESteamUserStatTypeFLOAT = 2,
    ESteamUserStatTypeAVGRATE = 3,
    ESteamUserStatTypeACHIEVEMENTS = 4,
    ESteamUserStatTypeGROUPACHIEVEMENTS = 5,
    ESteamUserStatTypeMAX = 6,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EChatEntryType {
    EChatEntryTypeInvalid = 0,
    EChatEntryTypeChatMsg = 1,
    EChatEntryTypeTyping = 2,
    EChatEntryTypeInviteGame = 3,
    EChatEntryTypeEmote = 4,
    EChatEntryTypeLeftConversation = 6,
    EChatEntryTypeEntered = 7,
    EChatEntryTypeWasKicked = 8,
    EChatEntryTypeWasBanned = 9,
    EChatEntryTypeDisconnected = 10,
    EChatEntryTypeHistoricalChat = 11,
    EChatEntryTypeLinkBlocked = 14,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EChatRoomEnterResponse {
    EChatRoomEnterResponseSuccess = 1,
    EChatRoomEnterResponseDoesntExist = 2,
    EChatRoomEnterResponseNotAllowed = 3,
    EChatRoomEnterResponseFull = 4,
    EChatRoomEnterResponseError = 5,
    EChatRoomEnterResponseBanned = 6,
    EChatRoomEnterResponseLimited = 7,
    EChatRoomEnterResponseClanDisabled = 8,
    EChatRoomEnterResponseCommunityBan = 9,
    EChatRoomEnterResponseMemberBlockedYou = 10,
    EChatRoomEnterResponseYouBlockedMember = 11,
    EChatRoomEnterResponseRatelimitExceeded = 15,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EChatSteamIDInstanceFlags {
    EChatAccountInstanceMask = 4095,
    EChatInstanceFlagClan = 524288,
    EChatInstanceFlagLobby = 262144,
    EChatInstanceFlagMMSLobby = 131072,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EMarketingMessageFlags {
    EMarketingMessageFlagsNone = 0,
    EMarketingMessageFlagsHighPriority = 1,
    EMarketingMessageFlagsPlatformWindows = 2,
    EMarketingMessageFlagsPlatformMac = 4,
    EMarketingMessageFlagsPlatformLinux = 8,
    EMarketingMessageFlagsPlatformRestrictions = 14,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum ENotificationPosition {
    EPositionTopLeft = 0,
    EPositionTopRight = 1,
    EPositionBottomLeft = 2,
    EPositionBottomRight = 3,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EBroadcastUploadResult {
    EBroadcastUploadResultNone = 0,
    EBroadcastUploadResultOK = 1,
    EBroadcastUploadResultInitFailed = 2,
    EBroadcastUploadResultFrameFailed = 3,
    EBroadcastUploadResultTimeout = 4,
    EBroadcastUploadResultBandwidthExceeded = 5,
    EBroadcastUploadResultLowFPS = 6,
    EBroadcastUploadResultMissingKeyFrames = 7,
    EBroadcastUploadResultNoConnection = 8,
    EBroadcastUploadResultRelayFailed = 9,
    EBroadcastUploadResultSettingsChanged = 10,
    EBroadcastUploadResultMissingAudio = 11,
    EBroadcastUploadResultTooFarBehind = 12,
    EBroadcastUploadResultTranscodeBehind = 13,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum ELaunchOptionType {
    ELaunchOptionType_None = 0,
    ELaunchOptionType_Default = 1,
    ELaunchOptionType_SafeMode = 2,
    ELaunchOptionType_Multiplayer = 3,
    ELaunchOptionType_Config = 4,
    ELaunchOptionType_OpenVR = 5,
    ELaunchOptionType_Server = 6,
    ELaunchOptionType_Editor = 7,
    ELaunchOptionType_Manual = 8,
    ELaunchOptionType_Benchmark = 9,
    ELaunchOptionType_Option1 = 10,
    ELaunchOptionType_Option2 = 11,
    ELaunchOptionType_Option3 = 12,
    ELaunchOptionType_OculusVR = 13,
    ELaunchOptionType_OpenVROverlay = 14,
    ELaunchOptionType_OSVR = 15,
    ELaunchOptionType_Dialog = 1000,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EVRHMDType {
    eEVRHMDType_None = -1,
    eEVRHMDType_Unknown = 0,
    eEVRHMDType_HTC_Dev = 1,
    eEVRHMDType_HTC_VivePre = 2,
    eEVRHMDType_HTC_Vive = 3,
    eEVRHMDType_HTC_Unknown = 20,
    eEVRHMDType_Oculus_DK1 = 21,
    eEVRHMDType_Oculus_DK2 = 22,
    eEVRHMDType_Oculus_Rift = 23,
    eEVRHMDType_Oculus_Unknown = 40,
    eEVRHMDType_Acer_Unknown = 50,
    eEVRHMDType_Acer_WindowsMR = 51,
    eEVRHMDType_Dell_Unknown = 60,
    eEVRHMDType_Dell_Visor = 61,
    eEVRHMDType_Lenovo_Unknown = 70,
    eEVRHMDType_Lenovo_Explorer = 71,
    eEVRHMDType_HP_Unknown = 80,
    eEVRHMDType_HP_WindowsMR = 81,
    eEVRHMDType_Samsung_Unknown = 90,
    eEVRHMDType_Samsung_Odyssey = 91,
    eEVRHMDType_Unannounced_Unknown = 100,
    eEVRHMDType_Unannounced_WindowsMR = 101,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EFriendRelationship {
    EFriendRelationshipNone = 0,
    EFriendRelationshipBlocked = 1,
    EFriendRelationshipRequestRecipient = 2,
    EFriendRelationshipFriend = 3,
    EFriendRelationshipRequestInitiator = 4,
    EFriendRelationshipIgnored = 5,
    EFriendRelationshipIgnoredFriend = 6,
    EFriendRelationshipSuggested_DEPRECATED = 7,
    EFriendRelationshipMax = 8,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EPersonaState {
    EPersonaStateOffline = 0,
    EPersonaStateOnline = 1,
    EPersonaStateBusy = 2,
    EPersonaStateAway = 3,
    EPersonaStateSnooze = 4,
    EPersonaStateLookingToTrade = 5,
    EPersonaStateLookingToPlay = 6,
    EPersonaStateMax = 7,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EFriendFlags {
    EFriendFlagNone = 0,
    EFriendFlagBlocked = 1,
    EFriendFlagFriendshipRequested = 2,
    EFriendFlagImmediate = 4,
    EFriendFlagClanMember = 8,
    EFriendFlagOnGameServer = 16,
    EFriendFlagRequestingFriendship = 128,
    EFriendFlagRequestingInfo = 256,
    EFriendFlagIgnored = 512,
    EFriendFlagIgnoredFriend = 1024,
    EFriendFlagChatMember = 4096,
    EFriendFlagAll = 65535,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EUserRestriction {
    nUserRestrictionNone = 0,
    nUserRestrictionUnknown = 1,
    nUserRestrictionAnyChat = 2,
    nUserRestrictionVoiceChat = 4,
    nUserRestrictionGroupChat = 8,
    nUserRestrictionRating = 16,
    nUserRestrictionGameInvites = 32,
    nUserRestrictionTrading = 64,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EOverlayToStoreFlag {
    EOverlayToStoreFlag_None = 0,
    EOverlayToStoreFlag_AddToCart = 1,
    EOverlayToStoreFlag_AddToCartAndShow = 2,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EPersonaChange {
    EPersonaChangeName = 1,
    EPersonaChangeStatus = 2,
    EPersonaChangeComeOnline = 4,
    EPersonaChangeGoneOffline = 8,
    EPersonaChangeGamePlayed = 16,
    EPersonaChangeGameServer = 32,
    EPersonaChangeAvatar = 64,
    EPersonaChangeJoinedSource = 128,
    EPersonaChangeLeftSource = 256,
    EPersonaChangeRelationshipChanged = 512,
    EPersonaChangeNameFirstSet = 1024,
    EPersonaChangeFacebookInfo = 2048,
    EPersonaChangeNickname = 4096,
    EPersonaChangeSteamLevel = 8192,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum ESteamAPICallFailure {
    ESteamAPICallFailureNone = -1,
    ESteamAPICallFailureSteamGone = 0,
    ESteamAPICallFailureNetworkFailure = 1,
    ESteamAPICallFailureInvalidHandle = 2,
    ESteamAPICallFailureMismatchedCallback = 3,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EGamepadTextInputMode {
    EGamepadTextInputModeNormal = 0,
    EGamepadTextInputModePassword = 1,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EGamepadTextInputLineMode {
    EGamepadTextInputLineModeSingleLine = 0,
    EGamepadTextInputLineModeMultipleLines = 1,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum ECheckFileSignature {
    ECheckFileSignatureInvalidSignature = 0,
    ECheckFileSignatureValidSignature = 1,
    ECheckFileSignatureFileNotFound = 2,
    ECheckFileSignatureNoSignaturesFoundForThisApp = 3,
    ECheckFileSignatureNoSignaturesFoundForThisFile = 4,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EMatchMakingServerResponse {
    eServerResponded = 0,
    eServerFailedToRespond = 1,
    eNoServersListedOnMasterServer = 2,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum ELobbyType {
    ELobbyTypePrivate = 0,
    ELobbyTypeFriendsOnly = 1,
    ELobbyTypePublic = 2,
    ELobbyTypeInvisible = 3,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum ELobbyComparison {
    ELobbyComparisonEqualToOrLessThan = -2,
    ELobbyComparisonLessThan = -1,
    ELobbyComparisonEqual = 0,
    ELobbyComparisonGreaterThan = 1,
    ELobbyComparisonEqualToOrGreaterThan = 2,
    ELobbyComparisonNotEqual = 3,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum ELobbyDistanceFilter {
    ELobbyDistanceFilterClose = 0,
    ELobbyDistanceFilterDefault = 1,
    ELobbyDistanceFilterFar = 2,
    ELobbyDistanceFilterWorldwide = 3,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EChatMemberStateChange {
    EChatMemberStateChangeEntered = 1,
    EChatMemberStateChangeLeft = 2,
    EChatMemberStateChangeDisconnected = 4,
    EChatMemberStateChangeKicked = 8,
    EChatMemberStateChangeBanned = 16,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum ERemoteStoragePlatform {
    ERemoteStoragePlatformNone = 0,
    ERemoteStoragePlatformWindows = 1,
    ERemoteStoragePlatformOSX = 2,
    ERemoteStoragePlatformPS3 = 4,
    ERemoteStoragePlatformLinux = 8,
    ERemoteStoragePlatformReserved2 = 16,
    ERemoteStoragePlatformAll = -1,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum ERemoteStoragePublishedFileVisibility {
    ERemoteStoragePublishedFileVisibilityPublic = 0,
    ERemoteStoragePublishedFileVisibilityFriendsOnly = 1,
    ERemoteStoragePublishedFileVisibilityPrivate = 2,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EWorkshopFileType {
    EWorkshopFileTypeCommunity = 0,
    EWorkshopFileTypeMicrotransaction = 1,
    EWorkshopFileTypeCollection = 2,
    EWorkshopFileTypeArt = 3,
    EWorkshopFileTypeVideo = 4,
    EWorkshopFileTypeScreenshot = 5,
    EWorkshopFileTypeGame = 6,
    EWorkshopFileTypeSoftware = 7,
    EWorkshopFileTypeConcept = 8,
    EWorkshopFileTypeWebGuide = 9,
    EWorkshopFileTypeIntegratedGuide = 10,
    EWorkshopFileTypeMerch = 11,
    EWorkshopFileTypeControllerBinding = 12,
    EWorkshopFileTypeSteamworksAccessInvite = 13,
    EWorkshopFileTypeSteamVideo = 14,
    EWorkshopFileTypeGameManagedItem = 15,
    EWorkshopFileTypeMax = 16,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EWorkshopVote {
    EWorkshopVoteUnvoted = 0,
    EWorkshopVoteFor = 1,
    EWorkshopVoteAgainst = 2,
    EWorkshopVoteLater = 3,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EWorkshopFileAction {
    EWorkshopFileActionPlayed = 0,
    EWorkshopFileActionCompleted = 1,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EWorkshopEnumerationType {
    EWorkshopEnumerationTypeRankedByVote = 0,
    EWorkshopEnumerationTypeRecent = 1,
    EWorkshopEnumerationTypeTrending = 2,
    EWorkshopEnumerationTypeFavoritesOfFriends = 3,
    EWorkshopEnumerationTypeVotedByFriends = 4,
    EWorkshopEnumerationTypeContentByFriends = 5,
    EWorkshopEnumerationTypeRecentFromFollowedUsers = 6,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EWorkshopVideoProvider {
    EWorkshopVideoProviderNone = 0,
    EWorkshopVideoProviderYoutube = 1,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EUGCReadAction {
    EUGCRead_ContinueReadingUntilFinished = 0,
    EUGCRead_ContinueReading = 1,
    EUGCRead_Close = 2,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum ELeaderboardDataRequest {
    ELeaderboardDataRequestGlobal = 0,
    ELeaderboardDataRequestGlobalAroundUser = 1,
    ELeaderboardDataRequestFriends = 2,
    ELeaderboardDataRequestUsers = 3,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum ELeaderboardSortMethod {
    ELeaderboardSortMethodNone = 0,
    ELeaderboardSortMethodAscending = 1,
    ELeaderboardSortMethodDescending = 2,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum ELeaderboardDisplayType {
    ELeaderboardDisplayTypeNone = 0,
    ELeaderboardDisplayTypeNumeric = 1,
    ELeaderboardDisplayTypeTimeSeconds = 2,
    ELeaderboardDisplayTypeTimeMilliSeconds = 3,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum ELeaderboardUploadScoreMethod {
    ELeaderboardUploadScoreMethodNone = 0,
    ELeaderboardUploadScoreMethodKeepBest = 1,
    ELeaderboardUploadScoreMethodForceUpdate = 2,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum ERegisterActivationCodeResult {
    ERegisterActivationCodeResultOK = 0,
    ERegisterActivationCodeResultFail = 1,
    ERegisterActivationCodeResultAlreadyRegistered = 2,
    ERegisterActivationCodeResultTimeout = 3,
    ERegisterActivationCodeAlreadyOwned = 4,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EP2PSessionError {
    EP2PSessionErrorNone = 0,
    EP2PSessionErrorNotRunningApp = 1,
    EP2PSessionErrorNoRightsToApp = 2,
    EP2PSessionErrorDestinationNotLoggedIn = 3,
    EP2PSessionErrorTimeout = 4,
    EP2PSessionErrorMax = 5,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EP2PSend {
    EP2PSendUnreliable = 0,
    EP2PSendUnreliableNoDelay = 1,
    EP2PSendReliable = 2,
    EP2PSendReliableWithBuffering = 3,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum ESNetSocketState {
    ESNetSocketStateInvalid = 0,
    ESNetSocketStateConnected = 1,
    ESNetSocketStateInitiated = 10,
    ESNetSocketStateLocalCandidatesFound = 11,
    ESNetSocketStateReceivedRemoteCandidates = 12,
    ESNetSocketStateChallengeHandshake = 15,
    ESNetSocketStateDisconnecting = 21,
    ESNetSocketStateLocalDisconnect = 22,
    ESNetSocketStateTimeoutDuringConnect = 23,
    ESNetSocketStateRemoteEndDisconnected = 24,
    ESNetSocketStateConnectionBroken = 25,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum ESNetSocketConnectionType {
    ESNetSocketConnectionTypeNotConnected = 0,
    ESNetSocketConnectionTypeUDP = 1,
    ESNetSocketConnectionTypeUDPRelay = 2,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EVRScreenshotType {
    EVRScreenshotType_None = 0,
    EVRScreenshotType_Mono = 1,
    EVRScreenshotType_Stereo = 2,
    EVRScreenshotType_MonoCubemap = 3,
    EVRScreenshotType_MonoPanorama = 4,
    EVRScreenshotType_StereoPanorama = 5,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum AudioPlayback_Status {
    AudioPlayback_Undefined = 0,
    AudioPlayback_Playing = 1,
    AudioPlayback_Paused = 2,
    AudioPlayback_Idle = 3,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EHTTPMethod {
    EHTTPMethodInvalid = 0,
    EHTTPMethodGET = 1,
    EHTTPMethodHEAD = 2,
    EHTTPMethodPOST = 3,
    EHTTPMethodPUT = 4,
    EHTTPMethodDELETE = 5,
    EHTTPMethodOPTIONS = 6,
    EHTTPMethodPATCH = 7,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EHTTPStatusCode {
    EHTTPStatusCodeInvalid = 0,
    EHTTPStatusCode100Continue = 100,
    EHTTPStatusCode101SwitchingProtocols = 101,
    EHTTPStatusCode200OK = 200,
    EHTTPStatusCode201Created = 201,
    EHTTPStatusCode202Accepted = 202,
    EHTTPStatusCode203NonAuthoritative = 203,
    EHTTPStatusCode204NoContent = 204,
    EHTTPStatusCode205ResetContent = 205,
    EHTTPStatusCode206PartialContent = 206,
    EHTTPStatusCode300MultipleChoices = 300,
    EHTTPStatusCode301MovedPermanently = 301,
    EHTTPStatusCode302Found = 302,
    EHTTPStatusCode303SeeOther = 303,
    EHTTPStatusCode304NotModified = 304,
    EHTTPStatusCode305UseProxy = 305,
    EHTTPStatusCode307TemporaryRedirect = 307,
    EHTTPStatusCode400BadRequest = 400,
    EHTTPStatusCode401Unauthorized = 401,
    EHTTPStatusCode402PaymentRequired = 402,
    EHTTPStatusCode403Forbidden = 403,
    EHTTPStatusCode404NotFound = 404,
    EHTTPStatusCode405MethodNotAllowed = 405,
    EHTTPStatusCode406NotAcceptable = 406,
    EHTTPStatusCode407ProxyAuthRequired = 407,
    EHTTPStatusCode408RequestTimeout = 408,
    EHTTPStatusCode409Conflict = 409,
    EHTTPStatusCode410Gone = 410,
    EHTTPStatusCode411LengthRequired = 411,
    EHTTPStatusCode412PreconditionFailed = 412,
    EHTTPStatusCode413RequestEntityTooLarge = 413,
    EHTTPStatusCode414RequestURITooLong = 414,
    EHTTPStatusCode415UnsupportedMediaType = 415,
    EHTTPStatusCode416RequestedRangeNotSatisfiable = 416,
    EHTTPStatusCode417ExpectationFailed = 417,
    EHTTPStatusCode4xxUnknown = 418,
    EHTTPStatusCode429TooManyRequests = 429,
    EHTTPStatusCode500InternalServerError = 500,
    EHTTPStatusCode501NotImplemented = 501,
    EHTTPStatusCode502BadGateway = 502,
    EHTTPStatusCode503ServiceUnavailable = 503,
    EHTTPStatusCode504GatewayTimeout = 504,
    EHTTPStatusCode505HTTPVersionNotSupported = 505,
    EHTTPStatusCode5xxUnknown = 599,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum ESteamControllerPad {
    ESteamControllerPad_Left = 0,
    ESteamControllerPad_Right = 1,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EControllerSource {
    EControllerSource_None = 0,
    EControllerSource_LeftTrackpad = 1,
    EControllerSource_RightTrackpad = 2,
    EControllerSource_Joystick = 3,
    EControllerSource_ABXY = 4,
    EControllerSource_Switch = 5,
    EControllerSource_LeftTrigger = 6,
    EControllerSource_RightTrigger = 7,
    EControllerSource_Gyro = 8,
    EControllerSource_CenterTrackpad = 9,
    EControllerSource_RightJoystick = 10,
    EControllerSource_DPad = 11,
    EControllerSource_Key = 12,
    EControllerSource_Mouse = 13,
    EControllerSource_Count = 14,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EControllerSourceMode {
    EControllerSourceMode_None = 0,
    EControllerSourceMode_Dpad = 1,
    EControllerSourceMode_Buttons = 2,
    EControllerSourceMode_FourButtons = 3,
    EControllerSourceMode_AbsoluteMouse = 4,
    EControllerSourceMode_RelativeMouse = 5,
    EControllerSourceMode_JoystickMove = 6,
    EControllerSourceMode_JoystickMouse = 7,
    EControllerSourceMode_JoystickCamera = 8,
    EControllerSourceMode_ScrollWheel = 9,
    EControllerSourceMode_Trigger = 10,
    EControllerSourceMode_TouchMenu = 11,
    EControllerSourceMode_MouseJoystick = 12,
    EControllerSourceMode_MouseRegion = 13,
    EControllerSourceMode_RadialMenu = 14,
    EControllerSourceMode_SingleButton = 15,
    EControllerSourceMode_Switches = 16,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EControllerActionOrigin {
    EControllerActionOrigin_None = 0,
    EControllerActionOrigin_A = 1,
    EControllerActionOrigin_B = 2,
    EControllerActionOrigin_X = 3,
    EControllerActionOrigin_Y = 4,
    EControllerActionOrigin_LeftBumper = 5,
    EControllerActionOrigin_RightBumper = 6,
    EControllerActionOrigin_LeftGrip = 7,
    EControllerActionOrigin_RightGrip = 8,
    EControllerActionOrigin_Start = 9,
    EControllerActionOrigin_Back = 10,
    EControllerActionOrigin_LeftPad_Touch = 11,
    EControllerActionOrigin_LeftPad_Swipe = 12,
    EControllerActionOrigin_LeftPad_Click = 13,
    EControllerActionOrigin_LeftPad_DPadNorth = 14,
    EControllerActionOrigin_LeftPad_DPadSouth = 15,
    EControllerActionOrigin_LeftPad_DPadWest = 16,
    EControllerActionOrigin_LeftPad_DPadEast = 17,
    EControllerActionOrigin_RightPad_Touch = 18,
    EControllerActionOrigin_RightPad_Swipe = 19,
    EControllerActionOrigin_RightPad_Click = 20,
    EControllerActionOrigin_RightPad_DPadNorth = 21,
    EControllerActionOrigin_RightPad_DPadSouth = 22,
    EControllerActionOrigin_RightPad_DPadWest = 23,
    EControllerActionOrigin_RightPad_DPadEast = 24,
    EControllerActionOrigin_LeftTrigger_Pull = 25,
    EControllerActionOrigin_LeftTrigger_Click = 26,
    EControllerActionOrigin_RightTrigger_Pull = 27,
    EControllerActionOrigin_RightTrigger_Click = 28,
    EControllerActionOrigin_LeftStick_Move = 29,
    EControllerActionOrigin_LeftStick_Click = 30,
    EControllerActionOrigin_LeftStick_DPadNorth = 31,
    EControllerActionOrigin_LeftStick_DPadSouth = 32,
    EControllerActionOrigin_LeftStick_DPadWest = 33,
    EControllerActionOrigin_LeftStick_DPadEast = 34,
    EControllerActionOrigin_Gyro_Move = 35,
    EControllerActionOrigin_Gyro_Pitch = 36,
    EControllerActionOrigin_Gyro_Yaw = 37,
    EControllerActionOrigin_Gyro_Roll = 38,
    EControllerActionOrigin_PS4_X = 39,
    EControllerActionOrigin_PS4_Circle = 40,
    EControllerActionOrigin_PS4_Triangle = 41,
    EControllerActionOrigin_PS4_Square = 42,
    EControllerActionOrigin_PS4_LeftBumper = 43,
    EControllerActionOrigin_PS4_RightBumper = 44,
    EControllerActionOrigin_PS4_Options = 45,
    EControllerActionOrigin_PS4_Share = 46,
    EControllerActionOrigin_PS4_LeftPad_Touch = 47,
    EControllerActionOrigin_PS4_LeftPad_Swipe = 48,
    EControllerActionOrigin_PS4_LeftPad_Click = 49,
    EControllerActionOrigin_PS4_LeftPad_DPadNorth = 50,
    EControllerActionOrigin_PS4_LeftPad_DPadSouth = 51,
    EControllerActionOrigin_PS4_LeftPad_DPadWest = 52,
    EControllerActionOrigin_PS4_LeftPad_DPadEast = 53,
    EControllerActionOrigin_PS4_RightPad_Touch = 54,
    EControllerActionOrigin_PS4_RightPad_Swipe = 55,
    EControllerActionOrigin_PS4_RightPad_Click = 56,
    EControllerActionOrigin_PS4_RightPad_DPadNorth = 57,
    EControllerActionOrigin_PS4_RightPad_DPadSouth = 58,
    EControllerActionOrigin_PS4_RightPad_DPadWest = 59,
    EControllerActionOrigin_PS4_RightPad_DPadEast = 60,
    EControllerActionOrigin_PS4_CenterPad_Touch = 61,
    EControllerActionOrigin_PS4_CenterPad_Swipe = 62,
    EControllerActionOrigin_PS4_CenterPad_Click = 63,
    EControllerActionOrigin_PS4_CenterPad_DPadNorth = 64,
    EControllerActionOrigin_PS4_CenterPad_DPadSouth = 65,
    EControllerActionOrigin_PS4_CenterPad_DPadWest = 66,
    EControllerActionOrigin_PS4_CenterPad_DPadEast = 67,
    EControllerActionOrigin_PS4_LeftTrigger_Pull = 68,
    EControllerActionOrigin_PS4_LeftTrigger_Click = 69,
    EControllerActionOrigin_PS4_RightTrigger_Pull = 70,
    EControllerActionOrigin_PS4_RightTrigger_Click = 71,
    EControllerActionOrigin_PS4_LeftStick_Move = 72,
    EControllerActionOrigin_PS4_LeftStick_Click = 73,
    EControllerActionOrigin_PS4_LeftStick_DPadNorth = 74,
    EControllerActionOrigin_PS4_LeftStick_DPadSouth = 75,
    EControllerActionOrigin_PS4_LeftStick_DPadWest = 76,
    EControllerActionOrigin_PS4_LeftStick_DPadEast = 77,
    EControllerActionOrigin_PS4_RightStick_Move = 78,
    EControllerActionOrigin_PS4_RightStick_Click = 79,
    EControllerActionOrigin_PS4_RightStick_DPadNorth = 80,
    EControllerActionOrigin_PS4_RightStick_DPadSouth = 81,
    EControllerActionOrigin_PS4_RightStick_DPadWest = 82,
    EControllerActionOrigin_PS4_RightStick_DPadEast = 83,
    EControllerActionOrigin_PS4_DPad_North = 84,
    EControllerActionOrigin_PS4_DPad_South = 85,
    EControllerActionOrigin_PS4_DPad_West = 86,
    EControllerActionOrigin_PS4_DPad_East = 87,
    EControllerActionOrigin_PS4_Gyro_Move = 88,
    EControllerActionOrigin_PS4_Gyro_Pitch = 89,
    EControllerActionOrigin_PS4_Gyro_Yaw = 90,
    EControllerActionOrigin_PS4_Gyro_Roll = 91,
    EControllerActionOrigin_XBoxOne_A = 92,
    EControllerActionOrigin_XBoxOne_B = 93,
    EControllerActionOrigin_XBoxOne_X = 94,
    EControllerActionOrigin_XBoxOne_Y = 95,
    EControllerActionOrigin_XBoxOne_LeftBumper = 96,
    EControllerActionOrigin_XBoxOne_RightBumper = 97,
    EControllerActionOrigin_XBoxOne_Menu = 98,
    EControllerActionOrigin_XBoxOne_View = 99,
    EControllerActionOrigin_XBoxOne_LeftTrigger_Pull = 100,
    EControllerActionOrigin_XBoxOne_LeftTrigger_Click = 101,
    EControllerActionOrigin_XBoxOne_RightTrigger_Pull = 102,
    EControllerActionOrigin_XBoxOne_RightTrigger_Click = 103,
    EControllerActionOrigin_XBoxOne_LeftStick_Move = 104,
    EControllerActionOrigin_XBoxOne_LeftStick_Click = 105,
    EControllerActionOrigin_XBoxOne_LeftStick_DPadNorth = 106,
    EControllerActionOrigin_XBoxOne_LeftStick_DPadSouth = 107,
    EControllerActionOrigin_XBoxOne_LeftStick_DPadWest = 108,
    EControllerActionOrigin_XBoxOne_LeftStick_DPadEast = 109,
    EControllerActionOrigin_XBoxOne_RightStick_Move = 110,
    EControllerActionOrigin_XBoxOne_RightStick_Click = 111,
    EControllerActionOrigin_XBoxOne_RightStick_DPadNorth = 112,
    EControllerActionOrigin_XBoxOne_RightStick_DPadSouth = 113,
    EControllerActionOrigin_XBoxOne_RightStick_DPadWest = 114,
    EControllerActionOrigin_XBoxOne_RightStick_DPadEast = 115,
    EControllerActionOrigin_XBoxOne_DPad_North = 116,
    EControllerActionOrigin_XBoxOne_DPad_South = 117,
    EControllerActionOrigin_XBoxOne_DPad_West = 118,
    EControllerActionOrigin_XBoxOne_DPad_East = 119,
    EControllerActionOrigin_XBox360_A = 120,
    EControllerActionOrigin_XBox360_B = 121,
    EControllerActionOrigin_XBox360_X = 122,
    EControllerActionOrigin_XBox360_Y = 123,
    EControllerActionOrigin_XBox360_LeftBumper = 124,
    EControllerActionOrigin_XBox360_RightBumper = 125,
    EControllerActionOrigin_XBox360_Start = 126,
    EControllerActionOrigin_XBox360_Back = 127,
    EControllerActionOrigin_XBox360_LeftTrigger_Pull = 128,
    EControllerActionOrigin_XBox360_LeftTrigger_Click = 129,
    EControllerActionOrigin_XBox360_RightTrigger_Pull = 130,
    EControllerActionOrigin_XBox360_RightTrigger_Click = 131,
    EControllerActionOrigin_XBox360_LeftStick_Move = 132,
    EControllerActionOrigin_XBox360_LeftStick_Click = 133,
    EControllerActionOrigin_XBox360_LeftStick_DPadNorth = 134,
    EControllerActionOrigin_XBox360_LeftStick_DPadSouth = 135,
    EControllerActionOrigin_XBox360_LeftStick_DPadWest = 136,
    EControllerActionOrigin_XBox360_LeftStick_DPadEast = 137,
    EControllerActionOrigin_XBox360_RightStick_Move = 138,
    EControllerActionOrigin_XBox360_RightStick_Click = 139,
    EControllerActionOrigin_XBox360_RightStick_DPadNorth = 140,
    EControllerActionOrigin_XBox360_RightStick_DPadSouth = 141,
    EControllerActionOrigin_XBox360_RightStick_DPadWest = 142,
    EControllerActionOrigin_XBox360_RightStick_DPadEast = 143,
    EControllerActionOrigin_XBox360_DPad_North = 144,
    EControllerActionOrigin_XBox360_DPad_South = 145,
    EControllerActionOrigin_XBox360_DPad_West = 146,
    EControllerActionOrigin_XBox360_DPad_East = 147,
    EControllerActionOrigin_SteamV2_A = 148,
    EControllerActionOrigin_SteamV2_B = 149,
    EControllerActionOrigin_SteamV2_X = 150,
    EControllerActionOrigin_SteamV2_Y = 151,
    EControllerActionOrigin_SteamV2_LeftBumper = 152,
    EControllerActionOrigin_SteamV2_RightBumper = 153,
    EControllerActionOrigin_SteamV2_LeftGrip = 154,
    EControllerActionOrigin_SteamV2_RightGrip = 155,
    EControllerActionOrigin_SteamV2_LeftGrip_Upper = 156,
    EControllerActionOrigin_SteamV2_RightGrip_Upper = 157,
    EControllerActionOrigin_SteamV2_LeftBumper_Pressure = 158,
    EControllerActionOrigin_SteamV2_RightBumper_Pressure = 159,
    EControllerActionOrigin_SteamV2_LeftGrip_Pressure = 160,
    EControllerActionOrigin_SteamV2_RightGrip_Pressure = 161,
    EControllerActionOrigin_SteamV2_LeftGrip_Upper_Pressure = 162,
    EControllerActionOrigin_SteamV2_RightGrip_Upper_Pressure = 163,
    EControllerActionOrigin_SteamV2_Start = 164,
    EControllerActionOrigin_SteamV2_Back = 165,
    EControllerActionOrigin_SteamV2_LeftPad_Touch = 166,
    EControllerActionOrigin_SteamV2_LeftPad_Swipe = 167,
    EControllerActionOrigin_SteamV2_LeftPad_Click = 168,
    EControllerActionOrigin_SteamV2_LeftPad_Pressure = 169,
    EControllerActionOrigin_SteamV2_LeftPad_DPadNorth = 170,
    EControllerActionOrigin_SteamV2_LeftPad_DPadSouth = 171,
    EControllerActionOrigin_SteamV2_LeftPad_DPadWest = 172,
    EControllerActionOrigin_SteamV2_LeftPad_DPadEast = 173,
    EControllerActionOrigin_SteamV2_RightPad_Touch = 174,
    EControllerActionOrigin_SteamV2_RightPad_Swipe = 175,
    EControllerActionOrigin_SteamV2_RightPad_Click = 176,
    EControllerActionOrigin_SteamV2_RightPad_Pressure = 177,
    EControllerActionOrigin_SteamV2_RightPad_DPadNorth = 178,
    EControllerActionOrigin_SteamV2_RightPad_DPadSouth = 179,
    EControllerActionOrigin_SteamV2_RightPad_DPadWest = 180,
    EControllerActionOrigin_SteamV2_RightPad_DPadEast = 181,
    EControllerActionOrigin_SteamV2_LeftTrigger_Pull = 182,
    EControllerActionOrigin_SteamV2_LeftTrigger_Click = 183,
    EControllerActionOrigin_SteamV2_RightTrigger_Pull = 184,
    EControllerActionOrigin_SteamV2_RightTrigger_Click = 185,
    EControllerActionOrigin_SteamV2_LeftStick_Move = 186,
    EControllerActionOrigin_SteamV2_LeftStick_Click = 187,
    EControllerActionOrigin_SteamV2_LeftStick_DPadNorth = 188,
    EControllerActionOrigin_SteamV2_LeftStick_DPadSouth = 189,
    EControllerActionOrigin_SteamV2_LeftStick_DPadWest = 190,
    EControllerActionOrigin_SteamV2_LeftStick_DPadEast = 191,
    EControllerActionOrigin_SteamV2_Gyro_Move = 192,
    EControllerActionOrigin_SteamV2_Gyro_Pitch = 193,
    EControllerActionOrigin_SteamV2_Gyro_Yaw = 194,
    EControllerActionOrigin_SteamV2_Gyro_Roll = 195,
    EControllerActionOrigin_Count = 196,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum ESteamControllerLEDFlag {
    ESteamControllerLEDFlag_SetColor = 0,
    ESteamControllerLEDFlag_RestoreUserDefault = 1,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum ESteamInputType {
    ESteamInputType_Unknown = 0,
    ESteamInputType_SteamController = 1,
    ESteamInputType_XBox360Controller = 2,
    ESteamInputType_XBoxOneController = 3,
    ESteamInputType_GenericXInput = 4,
    ESteamInputType_PS4Controller = 5,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EUGCMatchingUGCType {
    EUGCMatchingUGCType_Items = 0,
    EUGCMatchingUGCType_Items_Mtx = 1,
    EUGCMatchingUGCType_Items_ReadyToUse = 2,
    EUGCMatchingUGCType_Collections = 3,
    EUGCMatchingUGCType_Artwork = 4,
    EUGCMatchingUGCType_Videos = 5,
    EUGCMatchingUGCType_Screenshots = 6,
    EUGCMatchingUGCType_AllGuides = 7,
    EUGCMatchingUGCType_WebGuides = 8,
    EUGCMatchingUGCType_IntegratedGuides = 9,
    EUGCMatchingUGCType_UsableInGame = 10,
    EUGCMatchingUGCType_ControllerBindings = 11,
    EUGCMatchingUGCType_GameManagedItems = 12,
    EUGCMatchingUGCType_All = -1,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EUserUGCList {
    EUserUGCList_Published = 0,
    EUserUGCList_VotedOn = 1,
    EUserUGCList_VotedUp = 2,
    EUserUGCList_VotedDown = 3,
    EUserUGCList_WillVoteLater = 4,
    EUserUGCList_Favorited = 5,
    EUserUGCList_Subscribed = 6,
    EUserUGCList_UsedOrPlayed = 7,
    EUserUGCList_Followed = 8,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EUserUGCListSortOrder {
    EUserUGCListSortOrder_CreationOrderDesc = 0,
    EUserUGCListSortOrder_CreationOrderAsc = 1,
    EUserUGCListSortOrder_TitleAsc = 2,
    EUserUGCListSortOrder_LastUpdatedDesc = 3,
    EUserUGCListSortOrder_SubscriptionDateDesc = 4,
    EUserUGCListSortOrder_VoteScoreDesc = 5,
    EUserUGCListSortOrder_ForModeration = 6,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EUGCQuery {
    EUGCQuery_RankedByVote = 0,
    EUGCQuery_RankedByPublicationDate = 1,
    EUGCQuery_AcceptedForGameRankedByAcceptanceDate = 2,
    EUGCQuery_RankedByTrend = 3,
    EUGCQuery_FavoritedByFriendsRankedByPublicationDate = 4,
    EUGCQuery_CreatedByFriendsRankedByPublicationDate = 5,
    EUGCQuery_RankedByNumTimesReported = 6,
    EUGCQuery_CreatedByFollowedUsersRankedByPublicationDate = 7,
    EUGCQuery_NotYetRated = 8,
    EUGCQuery_RankedByTotalVotesAsc = 9,
    EUGCQuery_RankedByVotesUp = 10,
    EUGCQuery_RankedByTextSearch = 11,
    EUGCQuery_RankedByTotalUniqueSubscriptions = 12,
    EUGCQuery_RankedByPlaytimeTrend = 13,
    EUGCQuery_RankedByTotalPlaytime = 14,
    EUGCQuery_RankedByAveragePlaytimeTrend = 15,
    EUGCQuery_RankedByLifetimeAveragePlaytime = 16,
    EUGCQuery_RankedByPlaytimeSessionsTrend = 17,
    EUGCQuery_RankedByLifetimePlaytimeSessions = 18,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EItemUpdateStatus {
    EItemUpdateStatusInvalid = 0,
    EItemUpdateStatusPreparingConfig = 1,
    EItemUpdateStatusPreparingContent = 2,
    EItemUpdateStatusUploadingContent = 3,
    EItemUpdateStatusUploadingPreviewFile = 4,
    EItemUpdateStatusCommittingChanges = 5,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EItemState {
    EItemStateNone = 0,
    EItemStateSubscribed = 1,
    EItemStateLegacyItem = 2,
    EItemStateInstalled = 4,
    EItemStateNeedsUpdate = 8,
    EItemStateDownloading = 16,
    EItemStateDownloadPending = 32,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EItemStatistic {
    EItemStatistic_NumSubscriptions = 0,
    EItemStatistic_NumFavorites = 1,
    EItemStatistic_NumFollowers = 2,
    EItemStatistic_NumUniqueSubscriptions = 3,
    EItemStatistic_NumUniqueFavorites = 4,
    EItemStatistic_NumUniqueFollowers = 5,
    EItemStatistic_NumUniqueWebsiteViews = 6,
    EItemStatistic_ReportScore = 7,
    EItemStatistic_NumSecondsPlayed = 8,
    EItemStatistic_NumPlaytimeSessions = 9,
    EItemStatistic_NumComments = 10,
    EItemStatistic_NumSecondsPlayedDuringTimePeriod = 11,
    EItemStatistic_NumPlaytimeSessionsDuringTimePeriod = 12,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EItemPreviewType {
    EItemPreviewType_Image = 0,
    EItemPreviewType_YouTubeVideo = 1,
    EItemPreviewType_Sketchfab = 2,
    EItemPreviewType_EnvironmentMap_HorizontalCross = 3,
    EItemPreviewType_EnvironmentMap_LatLong = 4,
    EItemPreviewType_ReservedMax = 255,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum ESteamItemFlags {
    ESteamItemNoTrade = 1,
    ESteamItemRemoved = 256,
    ESteamItemConsumed = 512,
}

    #[repr(C)]
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub enum EParentalFeature {
    EFeatureInvalid = 0,
    EFeatureStore = 1,
    EFeatureCommunity = 2,
    EFeatureProfile = 3,
    EFeatureFriends = 4,
    EFeatureNews = 5,
    EFeatureTrading = 6,
    EFeatureSettings = 7,
    EFeatureConsole = 8,
    EFeatureBrowser = 9,
    EFeatureParentalSetup = 10,
    EFeatureLibrary = 11,
    EFeatureTest = 12,
    EFeatureMax = 13,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy)]
    pub struct ValvePackingSentinel_t {
    pub m_u32: u32,
    pub m_u64: u64,
    pub m_u16: u16,
    pub m_d: c_double,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct CallbackMsg_t {
    pub m_hSteamUser: HSteamUser,
    pub m_iCallback: c_int,
    pub m_pubParam: *mut u8,
    pub m_cubParam: c_int,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct SteamServerConnectFailure_t {
    pub m_eResult: EResult,
    pub m_bStillRetrying: bool,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct SteamServersDisconnected_t {
    pub m_eResult: EResult,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ClientGameServerDeny_t {
    pub m_uAppID: u32,
    pub m_unGameServerIP: u32,
    pub m_usGameServerPort: u16,
    pub m_bSecure: u16,
    pub m_uReason: u32,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ValidateAuthTicketResponse_t {
    pub m_SteamID: CSteamID,
    pub m_eAuthSessionResponse: EAuthSessionResponse,
    pub m_OwnerSteamID: CSteamID,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct MicroTxnAuthorizationResponse_t {
    pub m_unAppID: u32,
    pub m_ulOrderID: u64,
    pub m_bAuthorized: u8,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct EncryptedAppTicketResponse_t {
    pub m_eResult: EResult,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct GetAuthSessionTicketResponse_t {
    pub m_hAuthTicket: HAuthTicket,
    pub m_eResult: EResult,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy)]
    pub struct GameWebCallback_t {
    pub m_szURL: [c_char; 256],
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy)]
    pub struct StoreAuthURLResponse_t {
    pub m_szURL: [c_char; 512],
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct FriendGameInfo_t {
    pub m_gameID: CGameID,
    pub m_unGameIP: u32,
    pub m_usGamePort: u16,
    pub m_usQueryPort: u16,
    pub m_steamIDLobby: CSteamID,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct FriendSessionStateInfo_t {
    pub m_uiOnlineSessionInstances: u32,
    pub m_uiPublishedToFriendsSessionInstance: u8,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct PersonaStateChange_t {
    pub m_ulSteamID: u64,
    pub m_nChangeFlags: c_int,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct GameOverlayActivated_t {
    pub m_bActive: u8,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy)]
    pub struct GameServerChangeRequested_t {
    pub m_rgchServer: [c_char; 64],
    pub m_rgchPassword: [c_char; 64],
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct GameLobbyJoinRequested_t {
    pub m_steamIDLobby: CSteamID,
    pub m_steamIDFriend: CSteamID,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct AvatarImageLoaded_t {
    pub m_steamID: CSteamID,
    pub m_iImage: c_int,
    pub m_iWide: c_int,
    pub m_iTall: c_int,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ClanOfficerListResponse_t {
    pub m_steamIDClan: CSteamID,
    pub m_cOfficers: c_int,
    pub m_bSuccess: u8,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct FriendRichPresenceUpdate_t {
    pub m_steamIDFriend: CSteamID,
    pub m_nAppID: AppId_t,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy)]
    pub struct GameRichPresenceJoinRequested_t {
    pub m_steamIDFriend: CSteamID,
    pub m_rgchConnect: [c_char; 256],
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct GameConnectedClanChatMsg_t {
    pub m_steamIDClanChat: CSteamID,
    pub m_steamIDUser: CSteamID,
    pub m_iMessageID: c_int,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct GameConnectedChatJoin_t {
    pub m_steamIDClanChat: CSteamID,
    pub m_steamIDUser: CSteamID,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct GameConnectedChatLeave_t {
    pub m_steamIDClanChat: CSteamID,
    pub m_steamIDUser: CSteamID,
    pub m_bKicked: bool,
    pub m_bDropped: bool,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct DownloadClanActivityCountsResult_t {
    pub m_bSuccess: bool,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct JoinClanChatRoomCompletionResult_t {
    pub m_steamIDClanChat: CSteamID,
    pub m_eChatRoomEnterResponse: EChatRoomEnterResponse,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct GameConnectedFriendChatMsg_t {
    pub m_steamIDUser: CSteamID,
    pub m_iMessageID: c_int,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct FriendsGetFollowerCount_t {
    pub m_eResult: EResult,
    pub m_steamID: CSteamID,
    pub m_nCount: c_int,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct FriendsIsFollowing_t {
    pub m_eResult: EResult,
    pub m_steamID: CSteamID,
    pub m_bIsFollowing: bool,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy)]
    pub struct FriendsEnumerateFollowingList_t {
    pub m_eResult: EResult,
    pub m_rgSteamID: [CSteamID; 50],
    pub m_nResultsReturned: i32,
    pub m_nTotalResultCount: i32,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct SetPersonaNameResponse_t {
    pub m_bSuccess: bool,
    pub m_bLocalSuccess: bool,
    pub m_result: EResult,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct LowBatteryPower_t {
    pub m_nMinutesBatteryLeft: u8,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct SteamAPICallCompleted_t {
    pub m_hAsyncCall: SteamAPICall_t,
    pub m_iCallback: c_int,
    pub m_cubParam: u32,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct CheckFileSignature_t {
    pub m_eCheckFileSignature: ECheckFileSignature,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct GamepadTextInputDismissed_t {
    pub m_bSubmitted: bool,
    pub m_unSubmittedText: u32,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy)]
    pub struct MatchMakingKeyValuePair_t {
    pub m_szKey: [c_char; 256],
    pub m_szValue: [c_char; 256],
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct servernetadr_t {
    pub m_usConnectionPort: u16,
    pub m_usQueryPort: u16,
    pub m_unIP: u32,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct FavoritesListChanged_t {
    pub m_nIP: u32,
    pub m_nQueryPort: u32,
    pub m_nConnPort: u32,
    pub m_nAppID: u32,
    pub m_nFlags: u32,
    pub m_bAdd: bool,
    pub m_unAccountId: AccountID_t,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct LobbyInvite_t {
    pub m_ulSteamIDUser: u64,
    pub m_ulSteamIDLobby: u64,
    pub m_ulGameID: u64,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct LobbyEnter_t {
    pub m_ulSteamIDLobby: u64,
    pub m_rgfChatPermissions: u32,
    pub m_bLocked: bool,
    pub m_EChatRoomEnterResponse: u32,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct LobbyDataUpdate_t {
    pub m_ulSteamIDLobby: u64,
    pub m_ulSteamIDMember: u64,
    pub m_bSuccess: u8,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct LobbyChatUpdate_t {
    pub m_ulSteamIDLobby: u64,
    pub m_ulSteamIDUserChanged: u64,
    pub m_ulSteamIDMakingChange: u64,
    pub m_rgfChatMemberStateChange: u32,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct LobbyChatMsg_t {
    pub m_ulSteamIDLobby: u64,
    pub m_ulSteamIDUser: u64,
    pub m_eChatEntryType: u8,
    pub m_iChatID: u32,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct LobbyGameCreated_t {
    pub m_ulSteamIDLobby: u64,
    pub m_ulSteamIDGameServer: u64,
    pub m_unIP: u32,
    pub m_usPort: u16,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct LobbyMatchList_t {
    pub m_nLobbiesMatching: u32,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct LobbyKicked_t {
    pub m_ulSteamIDLobby: u64,
    pub m_ulSteamIDAdmin: u64,
    pub m_bKickedDueToDisconnect: u8,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct LobbyCreated_t {
    pub m_eResult: EResult,
    pub m_ulSteamIDLobby: u64,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct PSNGameBootInviteResult_t {
    pub m_bGameBootInviteExists: bool,
    pub m_steamIDLobby: CSteamID,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct FavoritesListAccountsUpdated_t {
    pub m_eResult: EResult,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct SteamParamStringArray_t {
    pub m_ppStrings: *mut *const c_char,
    pub m_nNumStrings: i32,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct RemoteStorageAppSyncedClient_t {
    pub m_nAppID: AppId_t,
    pub m_eResult: EResult,
    pub m_unNumDownloads: c_int,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct RemoteStorageAppSyncedServer_t {
    pub m_nAppID: AppId_t,
    pub m_eResult: EResult,
    pub m_unNumUploads: c_int,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy)]
    pub struct RemoteStorageAppSyncProgress_t {
    pub m_rgchCurrentFile: [c_char; 260],
    pub m_nAppID: AppId_t,
    pub m_uBytesTransferredThisChunk: u32,
    pub m_dAppPercentComplete: c_double,
    pub m_bUploading: bool,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct RemoteStorageAppSyncStatusCheck_t {
    pub m_nAppID: AppId_t,
    pub m_eResult: EResult,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy)]
    pub struct RemoteStorageFileShareResult_t {
    pub m_eResult: EResult,
    pub m_hFile: UGCHandle_t,
    pub m_rgchFilename: [c_char; 260],
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct RemoteStoragePublishFileResult_t {
    pub m_eResult: EResult,
    pub m_nPublishedFileId: PublishedFileId_t,
    pub m_bUserNeedsToAcceptWorkshopLegalAgreement: bool,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct RemoteStorageDeletePublishedFileResult_t {
    pub m_eResult: EResult,
    pub m_nPublishedFileId: PublishedFileId_t,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy)]
    pub struct RemoteStorageEnumerateUserPublishedFilesResult_t {
    pub m_eResult: EResult,
    pub m_nResultsReturned: i32,
    pub m_nTotalResultCount: i32,
    pub m_rgPublishedFileId: [PublishedFileId_t; 50],
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct RemoteStorageSubscribePublishedFileResult_t {
    pub m_eResult: EResult,
    pub m_nPublishedFileId: PublishedFileId_t,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy)]
    pub struct RemoteStorageEnumerateUserSubscribedFilesResult_t {
    pub m_eResult: EResult,
    pub m_nResultsReturned: i32,
    pub m_nTotalResultCount: i32,
    pub m_rgPublishedFileId: [PublishedFileId_t; 50],
    pub m_rgRTimeSubscribed: [u32; 50],
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct RemoteStorageUnsubscribePublishedFileResult_t {
    pub m_eResult: EResult,
    pub m_nPublishedFileId: PublishedFileId_t,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct RemoteStorageUpdatePublishedFileResult_t {
    pub m_eResult: EResult,
    pub m_nPublishedFileId: PublishedFileId_t,
    pub m_bUserNeedsToAcceptWorkshopLegalAgreement: bool,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy)]
    pub struct RemoteStorageDownloadUGCResult_t {
    pub m_eResult: EResult,
    pub m_hFile: UGCHandle_t,
    pub m_nAppID: AppId_t,
    pub m_nSizeInBytes: i32,
    pub m_pchFileName: [c_char; 260],
    pub m_ulSteamIDOwner: u64,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy)]
    pub struct RemoteStorageGetPublishedFileDetailsResult_t {
    pub m_eResult: EResult,
    pub m_nPublishedFileId: PublishedFileId_t,
    pub m_nCreatorAppID: AppId_t,
    pub m_nConsumerAppID: AppId_t,
    pub m_rgchTitle: [c_char; 129],
    pub m_rgchDescription: [c_char; 8000],
    pub m_hFile: UGCHandle_t,
    pub m_hPreviewFile: UGCHandle_t,
    pub m_ulSteamIDOwner: u64,
    pub m_rtimeCreated: u32,
    pub m_rtimeUpdated: u32,
    pub m_eVisibility: ERemoteStoragePublishedFileVisibility,
    pub m_bBanned: bool,
    pub m_rgchTags: [c_char; 1025],
    pub m_bTagsTruncated: bool,
    pub m_pchFileName: [c_char; 260],
    pub m_nFileSize: i32,
    pub m_nPreviewFileSize: i32,
    pub m_rgchURL: [c_char; 256],
    pub m_eFileType: EWorkshopFileType,
    pub m_bAcceptedForUse: bool,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy)]
    pub struct RemoteStorageEnumerateWorkshopFilesResult_t {
    pub m_eResult: EResult,
    pub m_nResultsReturned: i32,
    pub m_nTotalResultCount: i32,
    pub m_rgPublishedFileId: [PublishedFileId_t; 50],
    pub m_rgScore: [c_float; 50],
    pub m_nAppId: AppId_t,
    pub m_unStartIndex: u32,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy)]
    pub struct RemoteStorageGetPublishedItemVoteDetailsResult_t {
    pub m_eResult: EResult,
    pub m_unPublishedFileId: PublishedFileId_t,
    pub m_nVotesFor: i32,
    pub m_nVotesAgainst: i32,
    pub m_nReports: i32,
    pub m_fScore: c_float,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct RemoteStoragePublishedFileSubscribed_t {
    pub m_nPublishedFileId: PublishedFileId_t,
    pub m_nAppID: AppId_t,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct RemoteStoragePublishedFileUnsubscribed_t {
    pub m_nPublishedFileId: PublishedFileId_t,
    pub m_nAppID: AppId_t,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct RemoteStoragePublishedFileDeleted_t {
    pub m_nPublishedFileId: PublishedFileId_t,
    pub m_nAppID: AppId_t,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct RemoteStorageUpdateUserPublishedItemVoteResult_t {
    pub m_eResult: EResult,
    pub m_nPublishedFileId: PublishedFileId_t,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct RemoteStorageUserVoteDetails_t {
    pub m_eResult: EResult,
    pub m_nPublishedFileId: PublishedFileId_t,
    pub m_eVote: EWorkshopVote,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy)]
    pub struct RemoteStorageEnumerateUserSharedWorkshopFilesResult_t {
    pub m_eResult: EResult,
    pub m_nResultsReturned: i32,
    pub m_nTotalResultCount: i32,
    pub m_rgPublishedFileId: [PublishedFileId_t; 50],
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct RemoteStorageSetUserPublishedFileActionResult_t {
    pub m_eResult: EResult,
    pub m_nPublishedFileId: PublishedFileId_t,
    pub m_eAction: EWorkshopFileAction,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy)]
    pub struct RemoteStorageEnumeratePublishedFilesByUserActionResult_t {
    pub m_eResult: EResult,
    pub m_eAction: EWorkshopFileAction,
    pub m_nResultsReturned: i32,
    pub m_nTotalResultCount: i32,
    pub m_rgPublishedFileId: [PublishedFileId_t; 50],
    pub m_rgRTimeUpdated: [u32; 50],
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy)]
    pub struct RemoteStoragePublishFileProgress_t {
    pub m_dPercentFile: c_double,
    pub m_bPreview: bool,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct RemoteStoragePublishedFileUpdated_t {
    pub m_nPublishedFileId: PublishedFileId_t,
    pub m_nAppID: AppId_t,
    pub m_ulUnused: u64,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct RemoteStorageFileWriteAsyncComplete_t {
    pub m_eResult: EResult,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct RemoteStorageFileReadAsyncComplete_t {
    pub m_hFileReadAsync: SteamAPICall_t,
    pub m_eResult: EResult,
    pub m_nOffset: u32,
    pub m_cubRead: u32,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct LeaderboardEntry_t {
    pub m_steamIDUser: CSteamID,
    pub m_nGlobalRank: i32,
    pub m_nScore: i32,
    pub m_cDetails: i32,
    pub m_hUGC: UGCHandle_t,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct UserStatsReceived_t {
    pub m_nGameID: u64,
    pub m_eResult: EResult,
    pub m_steamIDUser: CSteamID,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct UserStatsStored_t {
    pub m_nGameID: u64,
    pub m_eResult: EResult,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy)]
    pub struct UserAchievementStored_t {
    pub m_nGameID: u64,
    pub m_bGroupAchievement: bool,
    pub m_rgchAchievementName: [c_char; 128],
    pub m_nCurProgress: u32,
    pub m_nMaxProgress: u32,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct LeaderboardFindResult_t {
    pub m_hSteamLeaderboard: SteamLeaderboard_t,
    pub m_bLeaderboardFound: u8,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct LeaderboardScoresDownloaded_t {
    pub m_hSteamLeaderboard: SteamLeaderboard_t,
    pub m_hSteamLeaderboardEntries: SteamLeaderboardEntries_t,
    pub m_cEntryCount: c_int,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct LeaderboardScoreUploaded_t {
    pub m_bSuccess: u8,
    pub m_hSteamLeaderboard: SteamLeaderboard_t,
    pub m_nScore: i32,
    pub m_bScoreChanged: u8,
    pub m_nGlobalRankNew: c_int,
    pub m_nGlobalRankPrevious: c_int,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct NumberOfCurrentPlayers_t {
    pub m_bSuccess: u8,
    pub m_cPlayers: i32,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct UserStatsUnloaded_t {
    pub m_steamIDUser: CSteamID,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy)]
    pub struct UserAchievementIconFetched_t {
    pub m_nGameID: CGameID,
    pub m_rgchAchievementName: [c_char; 128],
    pub m_bAchieved: bool,
    pub m_nIconHandle: c_int,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct GlobalAchievementPercentagesReady_t {
    pub m_nGameID: u64,
    pub m_eResult: EResult,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct LeaderboardUGCSet_t {
    pub m_eResult: EResult,
    pub m_hSteamLeaderboard: SteamLeaderboard_t,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct PS3TrophiesInstalled_t {
    pub m_nGameID: u64,
    pub m_eResult: EResult,
    pub m_ulRequiredDiskSpace: u64,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct GlobalStatsReceived_t {
    pub m_nGameID: u64,
    pub m_eResult: EResult,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct DlcInstalled_t {
    pub m_nAppID: AppId_t,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct RegisterActivationCodeResponse_t {
    pub m_eResult: ERegisterActivationCodeResult,
    pub m_unPackageRegistered: u32,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy)]
    pub struct AppProofOfPurchaseKeyResponse_t {
    pub m_eResult: EResult,
    pub m_nAppID: u32,
    pub m_cchKeyLength: u32,
    pub m_rgchKey: [c_char; 240],
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy)]
    pub struct FileDetailsResult_t {
    pub m_eResult: EResult,
    pub m_ulFileSize: u64,
    pub m_FileSHA: [u8; 20],
    pub m_unFlags: u32,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct P2PSessionState_t {
    pub m_bConnectionActive: u8,
    pub m_bConnecting: u8,
    pub m_eP2PSessionError: u8,
    pub m_bUsingRelay: u8,
    pub m_nBytesQueuedForSend: i32,
    pub m_nPacketsQueuedForSend: i32,
    pub m_nRemoteIP: u32,
    pub m_nRemotePort: u16,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct P2PSessionRequest_t {
    pub m_steamIDRemote: CSteamID,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct P2PSessionConnectFail_t {
    pub m_steamIDRemote: CSteamID,
    pub m_eP2PSessionError: u8,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct SocketStatusCallback_t {
    pub m_hSocket: SNetSocket_t,
    pub m_hListenSocket: SNetListenSocket_t,
    pub m_steamIDRemote: CSteamID,
    pub m_eSNetSocketState: c_int,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ScreenshotReady_t {
    pub m_hLocal: ScreenshotHandle,
    pub m_eResult: EResult,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy)]
    pub struct VolumeHasChanged_t {
    pub m_flNewVolume: c_float,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct MusicPlayerWantsShuffled_t {
    pub m_bShuffled: bool,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct MusicPlayerWantsLooped_t {
    pub m_bLooped: bool,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy)]
    pub struct MusicPlayerWantsVolume_t {
    pub m_flNewVolume: c_float,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct MusicPlayerSelectsQueueEntry_t {
    pub nID: c_int,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct MusicPlayerSelectsPlaylistEntry_t {
    pub nID: c_int,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct MusicPlayerWantsPlayingRepeatStatus_t {
    pub m_nPlayingRepeatStatus: c_int,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct HTTPRequestCompleted_t {
    pub m_hRequest: HTTPRequestHandle,
    pub m_ulContextValue: u64,
    pub m_bRequestSuccessful: bool,
    pub m_eStatusCode: EHTTPStatusCode,
    pub m_unBodySize: u32,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct HTTPRequestHeadersReceived_t {
    pub m_hRequest: HTTPRequestHandle,
    pub m_ulContextValue: u64,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct HTTPRequestDataReceived_t {
    pub m_hRequest: HTTPRequestHandle,
    pub m_ulContextValue: u64,
    pub m_cOffset: u32,
    pub m_cBytesReceived: u32,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy)]
    pub struct ControllerAnalogActionData_t {
    pub eMode: EControllerSourceMode,
    pub x: c_float,
    pub y: c_float,
    pub bActive: bool,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ControllerDigitalActionData_t {
    pub bState: bool,
    pub bActive: bool,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy)]
    pub struct ControllerMotionData_t {
    pub rotQuatX: c_float,
    pub rotQuatY: c_float,
    pub rotQuatZ: c_float,
    pub rotQuatW: c_float,
    pub posAccelX: c_float,
    pub posAccelY: c_float,
    pub posAccelZ: c_float,
    pub rotVelX: c_float,
    pub rotVelY: c_float,
    pub rotVelZ: c_float,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy)]
    pub struct SteamUGCDetails_t {
    pub m_nPublishedFileId: PublishedFileId_t,
    pub m_eResult: EResult,
    pub m_eFileType: EWorkshopFileType,
    pub m_nCreatorAppID: AppId_t,
    pub m_nConsumerAppID: AppId_t,
    pub m_rgchTitle: [c_char; 129],
    pub m_rgchDescription: [c_char; 8000],
    pub m_ulSteamIDOwner: u64,
    pub m_rtimeCreated: u32,
    pub m_rtimeUpdated: u32,
    pub m_rtimeAddedToUserList: u32,
    pub m_eVisibility: ERemoteStoragePublishedFileVisibility,
    pub m_bBanned: bool,
    pub m_bAcceptedForUse: bool,
    pub m_bTagsTruncated: bool,
    pub m_rgchTags: [c_char; 1025],
    pub m_hFile: UGCHandle_t,
    pub m_hPreviewFile: UGCHandle_t,
    pub m_pchFileName: [c_char; 260],
    pub m_nFileSize: i32,
    pub m_nPreviewFileSize: i32,
    pub m_rgchURL: [c_char; 256],
    pub m_unVotesUp: u32,
    pub m_unVotesDown: u32,
    pub m_flScore: c_float,
    pub m_unNumChildren: u32,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct SteamUGCQueryCompleted_t {
    pub m_handle: UGCQueryHandle_t,
    pub m_eResult: EResult,
    pub m_unNumResultsReturned: u32,
    pub m_unTotalMatchingResults: u32,
    pub m_bCachedData: bool,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy)]
    pub struct SteamUGCRequestUGCDetailsResult_t {
    pub m_details: SteamUGCDetails_t,
    pub m_bCachedData: bool,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct CreateItemResult_t {
    pub m_eResult: EResult,
    pub m_nPublishedFileId: PublishedFileId_t,
    pub m_bUserNeedsToAcceptWorkshopLegalAgreement: bool,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct SubmitItemUpdateResult_t {
    pub m_eResult: EResult,
    pub m_bUserNeedsToAcceptWorkshopLegalAgreement: bool,
    pub m_nPublishedFileId: PublishedFileId_t,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct DownloadItemResult_t {
    pub m_unAppID: AppId_t,
    pub m_nPublishedFileId: PublishedFileId_t,
    pub m_eResult: EResult,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct UserFavoriteItemsListChanged_t {
    pub m_nPublishedFileId: PublishedFileId_t,
    pub m_eResult: EResult,
    pub m_bWasAddRequest: bool,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct SetUserItemVoteResult_t {
    pub m_nPublishedFileId: PublishedFileId_t,
    pub m_eResult: EResult,
    pub m_bVoteUp: bool,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct GetUserItemVoteResult_t {
    pub m_nPublishedFileId: PublishedFileId_t,
    pub m_eResult: EResult,
    pub m_bVotedUp: bool,
    pub m_bVotedDown: bool,
    pub m_bVoteSkipped: bool,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct StartPlaytimeTrackingResult_t {
    pub m_eResult: EResult,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct StopPlaytimeTrackingResult_t {
    pub m_eResult: EResult,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct AddUGCDependencyResult_t {
    pub m_eResult: EResult,
    pub m_nPublishedFileId: PublishedFileId_t,
    pub m_nChildPublishedFileId: PublishedFileId_t,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct RemoveUGCDependencyResult_t {
    pub m_eResult: EResult,
    pub m_nPublishedFileId: PublishedFileId_t,
    pub m_nChildPublishedFileId: PublishedFileId_t,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct AddAppDependencyResult_t {
    pub m_eResult: EResult,
    pub m_nPublishedFileId: PublishedFileId_t,
    pub m_nAppID: AppId_t,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct RemoveAppDependencyResult_t {
    pub m_eResult: EResult,
    pub m_nPublishedFileId: PublishedFileId_t,
    pub m_nAppID: AppId_t,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy)]
    pub struct GetAppDependenciesResult_t {
    pub m_eResult: EResult,
    pub m_nPublishedFileId: PublishedFileId_t,
    pub m_rgAppIDs: [AppId_t; 32],
    pub m_nNumAppDependencies: u32,
    pub m_nTotalNumAppDependencies: u32,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct DeleteItemResult_t {
    pub m_eResult: EResult,
    pub m_nPublishedFileId: PublishedFileId_t,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct SteamAppInstalled_t {
    pub m_nAppID: AppId_t,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct SteamAppUninstalled_t {
    pub m_nAppID: AppId_t,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct HTML_BrowserReady_t {
    pub unBrowserHandle: HHTMLBrowser,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy)]
    pub struct HTML_NeedsPaint_t {
    pub unBrowserHandle: HHTMLBrowser,
    pub pBGRA: *const c_char,
    pub unWide: u32,
    pub unTall: u32,
    pub unUpdateX: u32,
    pub unUpdateY: u32,
    pub unUpdateWide: u32,
    pub unUpdateTall: u32,
    pub unScrollX: u32,
    pub unScrollY: u32,
    pub flPageScale: c_float,
    pub unPageSerial: u32,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct HTML_StartRequest_t {
    pub unBrowserHandle: HHTMLBrowser,
    pub pchURL: *const c_char,
    pub pchTarget: *const c_char,
    pub pchPostData: *const c_char,
    pub bIsRedirect: bool,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct HTML_CloseBrowser_t {
    pub unBrowserHandle: HHTMLBrowser,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct HTML_URLChanged_t {
    pub unBrowserHandle: HHTMLBrowser,
    pub pchURL: *const c_char,
    pub pchPostData: *const c_char,
    pub bIsRedirect: bool,
    pub pchPageTitle: *const c_char,
    pub bNewNavigation: bool,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct HTML_FinishedRequest_t {
    pub unBrowserHandle: HHTMLBrowser,
    pub pchURL: *const c_char,
    pub pchPageTitle: *const c_char,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct HTML_OpenLinkInNewTab_t {
    pub unBrowserHandle: HHTMLBrowser,
    pub pchURL: *const c_char,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct HTML_ChangedTitle_t {
    pub unBrowserHandle: HHTMLBrowser,
    pub pchTitle: *const c_char,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct HTML_SearchResults_t {
    pub unBrowserHandle: HHTMLBrowser,
    pub unResults: u32,
    pub unCurrentMatch: u32,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct HTML_CanGoBackAndForward_t {
    pub unBrowserHandle: HHTMLBrowser,
    pub bCanGoBack: bool,
    pub bCanGoForward: bool,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy)]
    pub struct HTML_HorizontalScroll_t {
    pub unBrowserHandle: HHTMLBrowser,
    pub unScrollMax: u32,
    pub unScrollCurrent: u32,
    pub flPageScale: c_float,
    pub bVisible: bool,
    pub unPageSize: u32,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy)]
    pub struct HTML_VerticalScroll_t {
    pub unBrowserHandle: HHTMLBrowser,
    pub unScrollMax: u32,
    pub unScrollCurrent: u32,
    pub flPageScale: c_float,
    pub bVisible: bool,
    pub unPageSize: u32,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct HTML_LinkAtPosition_t {
    pub unBrowserHandle: HHTMLBrowser,
    pub x: u32,
    pub y: u32,
    pub pchURL: *const c_char,
    pub bInput: bool,
    pub bLiveLink: bool,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct HTML_JSAlert_t {
    pub unBrowserHandle: HHTMLBrowser,
    pub pchMessage: *const c_char,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct HTML_JSConfirm_t {
    pub unBrowserHandle: HHTMLBrowser,
    pub pchMessage: *const c_char,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct HTML_FileOpenDialog_t {
    pub unBrowserHandle: HHTMLBrowser,
    pub pchTitle: *const c_char,
    pub pchInitialFile: *const c_char,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct HTML_NewWindow_t {
    pub unBrowserHandle: HHTMLBrowser,
    pub pchURL: *const c_char,
    pub unX: u32,
    pub unY: u32,
    pub unWide: u32,
    pub unTall: u32,
    pub unNewWindow_BrowserHandle: HHTMLBrowser,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct HTML_SetCursor_t {
    pub unBrowserHandle: HHTMLBrowser,
    pub eMouseCursor: u32,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct HTML_StatusText_t {
    pub unBrowserHandle: HHTMLBrowser,
    pub pchMsg: *const c_char,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct HTML_ShowToolTip_t {
    pub unBrowserHandle: HHTMLBrowser,
    pub pchMsg: *const c_char,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct HTML_UpdateToolTip_t {
    pub unBrowserHandle: HHTMLBrowser,
    pub pchMsg: *const c_char,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct HTML_HideToolTip_t {
    pub unBrowserHandle: HHTMLBrowser,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct HTML_BrowserRestarted_t {
    pub unBrowserHandle: HHTMLBrowser,
    pub unOldBrowserHandle: HHTMLBrowser,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct SteamItemDetails_t {
    pub m_itemId: SteamItemInstanceID_t,
    pub m_iDefinition: SteamItemDef_t,
    pub m_unQuantity: u16,
    pub m_unFlags: u16,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct SteamInventoryResultReady_t {
    pub m_handle: SteamInventoryResult_t,
    pub m_result: EResult,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct SteamInventoryFullUpdate_t {
    pub m_handle: SteamInventoryResult_t,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct SteamInventoryEligiblePromoItemDefIDs_t {
    pub m_result: EResult,
    pub m_steamID: CSteamID,
    pub m_numEligiblePromoItemDefs: c_int,
    pub m_bCachedData: bool,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct SteamInventoryStartPurchaseResult_t {
    pub m_result: EResult,
    pub m_ulOrderID: u64,
    pub m_ulTransID: u64,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy)]
    pub struct SteamInventoryRequestPricesResult_t {
    pub m_result: EResult,
    pub m_rgchCurrency: [c_char; 4],
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct BroadcastUploadStop_t {
    pub m_eResult: EBroadcastUploadResult,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy)]
    pub struct GetVideoURLResult_t {
    pub m_eResult: EResult,
    pub m_unVideoAppID: AppId_t,
    pub m_rgchURL: [c_char; 256],
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct GetOPFSettingsResult_t {
    pub m_eResult: EResult,
    pub m_unVideoAppID: AppId_t,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct CCallbackBase {
    pub m_nCallbackFlags: u8,
    pub m_iCallback: c_int,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct GSClientApprove_t {
    pub m_SteamID: CSteamID,
    pub m_OwnerSteamID: CSteamID,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy)]
    pub struct GSClientDeny_t {
    pub m_SteamID: CSteamID,
    pub m_eDenyReason: EDenyReason,
    pub m_rgchOptionalText: [c_char; 128],
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct GSClientKick_t {
    pub m_SteamID: CSteamID,
    pub m_eDenyReason: EDenyReason,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy)]
    pub struct GSClientAchievementStatus_t {
    pub m_SteamID: u64,
    pub m_pchAchievement: [c_char; 128],
    pub m_bUnlocked: bool,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct GSPolicyResponse_t {
    pub m_bSecure: u8,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct GSGameplayStats_t {
    pub m_eResult: EResult,
    pub m_nRank: i32,
    pub m_unTotalConnects: u32,
    pub m_unTotalMinutesPlayed: u32,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct GSClientGroupStatus_t {
    pub m_SteamIDUser: CSteamID,
    pub m_SteamIDGroup: CSteamID,
    pub m_bMember: bool,
    pub m_bOfficer: bool,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct GSReputation_t {
    pub m_eResult: EResult,
    pub m_unReputationScore: u32,
    pub m_bBanned: bool,
    pub m_unBannedIP: u32,
    pub m_usBannedPort: u16,
    pub m_ulBannedGameID: u64,
    pub m_unBanExpires: u32,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct AssociateWithClanResult_t {
    pub m_eResult: EResult,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ComputeNewPlayerCompatibilityResult_t {
    pub m_eResult: EResult,
    pub m_cPlayersThatDontLikeCandidate: c_int,
    pub m_cPlayersThatCandidateDoesntLike: c_int,
    pub m_cClanPlayersThatDontLikeCandidate: c_int,
    pub m_SteamIDCandidate: CSteamID,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct GSStatsReceived_t {
    pub m_eResult: EResult,
    pub m_steamIDUser: CSteamID,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct GSStatsStored_t {
    pub m_eResult: EResult,
    pub m_steamIDUser: CSteamID,
}

    #[repr(C, packed(4))]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct GSStatsUnloaded_t {
    pub m_steamIDUser: CSteamID,
}
