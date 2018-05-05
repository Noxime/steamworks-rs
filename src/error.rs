use sys;

/// Covers errors that can be returned by the steamworks API
///
/// Documentation is based on official documentation which doesn't
/// always explain when an error could be returned or its meaning.
#[derive(Debug, Fail)]
pub enum SteamError {
    /// Returned if the steamworks API fails to initialize.
    #[fail(display = "failed to init the steamworks API")]
    InitFailed,
    /// Returned if the steamworks API fails to perform an action
    #[fail(display = "a generic failure from the steamworks API")]
    Generic,
    /// Returned when steam fails performing a network request
    #[fail(display = "there isn't a network connection to steam or it failed to connect")]
    NoConnection,
    /// Return when the password or ticked used is invalid
    #[fail(display = "password or ticket is invalid")]
    InvalidPassword,
    /// Returned when the user is already logged in at another location
    #[fail(display = "user logged in elsewhere")]
    LoggedInElsewhere,
    /// Returned when the protocol version is incorrect
    #[fail(display = "the protocol version is incorrect")]
    InvalidProtocolVersion,
    /// Returned when a passed parameter is invalid
    #[fail(display = "a parameter is invalid")]
    InvalidParameter,
    /// Returned when a file is not found
    #[fail(display = "a file was not found")]
    FileNotFound,
    /// Returned when the called method was busy
    ///
    /// No action was performed
    #[fail(display = "method busy")]
    Busy,
    /// Returned when the called object was in an
    /// invalid state
    #[fail(display = "object in invalid state")]
    InvalidState,
    /// Returned when the name is invalid
    #[fail(display = "name is invalid")]
    InvalidName,
    /// Returned when the email is invalid
    #[fail(display = "email is invalid")]
    InvalidEmail,
    /// Returned when the name is not unique
    #[fail(display = "name is not unique")]
    DuplicateName,
    /// Returned when access is denied
    #[fail(display = "access denied")]
    AccessDenied,
    /// Returned when the operation timed out
    #[fail(display = "operation timed")]
    Timeout,
    /// Returned when the user is VAC2 banned
    #[fail(display = "VAC2 banned")]
    Banned,
    /// Returned when the account is not found
    #[fail(display = "account not found")]
    AccountNotFound,
    /// Returned when the passed steam id is invalid
    #[fail(display = "steamID is invalid")]
    InvalidSteamID,
    /// Returned when the requested service in unavailable
    #[fail(display = "requested service is unavailable")]
    ServiceUnavailable,
    /// Returned when the user is not logged on
    #[fail(display = "user not logged on")]
    NotLoggedOn,
    /// Returned when the request is pending (e.g. in progress/waiting)
    #[fail(display = "request is pending")]
    Pending,
    /// Returned when encryption or decryption fails
    #[fail(display = "encryption/decryption failed")]
    EncryptionFailure,
    /// Returned when you have insufficient privilege to perform
    /// the action
    #[fail(display = "insufficient privilege")]
    InsufficientPrivilege,
    /// Returned when you have hit the API limits
    #[fail(display = "limit exceeded")]
    LimitExceeded,
    /// Returned when the user's access has been revoked (e.g. revoked
    /// guess passes)
    #[fail(display = "access revoked")]
    Revoked,
    /// Returned when the user's access has expired
    #[fail(display = "access expired")]
    Expired,
    /// Returned when the licence/guest pass has already been redeemed
    #[fail(display = "licence/guest pass already redeemed")]
    AlreadyRedeemed,
    /// Returned when the requested action is a duplicate and has
    /// already occurred.
    ///
    /// The action will be ignored
    #[fail(display = "request is a duplicate")]
    DuplicateRequest,
    /// Returned when all the games in the guest pass are already
    /// owned by the user
    #[fail(display = "all games requested already owned")]
    AlreadyOwned,
    /// Returned when the ip address is not found
    #[fail(display = "ip address not found")]
    IPNotFound,
    /// Returned when the change failed to write to the data store
    #[fail(display = "failed to write change")]
    PersistFailed,
    /// Returned when the operation failed to acquire the access lock
    #[fail(display = "failed to acquire access lock")]
    LockingFailed,
    /// Undocumented
    #[fail(display = "logon session replaced")]
    LogonSessionReplaced,
    /// Undocumented
    #[fail(display = "connect failed")]
    ConnectFailed,
    /// Undocumented
    #[fail(display = "handshake failed")]
    HandshakeFailed,
    /// Undocumented
    #[fail(display = "IO failure")]
    IOFailure,
    /// Undocumented
    #[fail(display = "remote disconnect")]
    RemoteDisconnect,
    /// Returned when the requested shopping cart wasn't found
    #[fail(display = "failed to find the requested shopping cart")]
    ShoppingCartNotFound,
    /// Returned when the user blocks an action
    #[fail(display = "action blocked")]
    Blocked,
    /// Returned when the target user is ignoring the sender
    #[fail(display = "target is ignoring sender")]
    Ignored,
    /// Returned when nothing matching the request is found
    #[fail(display = "no matches found")]
    NoMatch,
    /// Undocumented
    #[fail(display = "account disabled")]
    AccountDisabled,
    /// Returned when the service isn't accepting content changes at
    /// this moment
    #[fail(display = "service is read only")]
    ServiceReadOnly,
    /// Returned when the account doesn't have value so the feature
    /// isn't available
    #[fail(display = "account not featured")]
    AccountNotFeatured,
    /// Allowed to take this action but only because the requester is
    /// an admin
    #[fail(display = "administrator ok")]
    AdministratorOK,
    /// Returned when there is a version mismatch in content transmitted
    /// within the steam protocol
    #[fail(display = "version mismatch with transmitted content")]
    ContentVersion,
    /// Returned when the current CM cannot service the user's request.
    ///
    /// The user should try another.
    #[fail(display = "CM cannot service user")]
    TryAnotherCM,
    /// Returned when the user is already logged in elsewhere and the
    /// cached credential login failed.
    #[fail(display = "user already logged in, cached login failed")]
    PasswordRequiredToKickSession,
    /// Returned when the user is already logged in elsewhere, you
    /// must wait before trying again
    #[fail(display = "user already logged in, please wait")]
    AlreadyLoggedInElsewhere,
    /// Returned when a long running operation (e.g. download) is
    /// suspended/paused.
    #[fail(display = "operation suspended/paused")]
    Suspended,
    /// Returned when an operation is cancelled
    #[fail(display = "operation cancelled")]
    Cancelled,
    /// Returned when an operation is cancelled due to data corruption
    #[fail(display = "operation cancelled due to data corruption")]
    DataCorruption,
    /// Returned when an operation is cancelled due to running out of disk
    /// space
    #[fail(display = "operation cancelled due to the disk being full")]
    DiskFull,
    /// Returned when a remote call or an IPC call failed
    #[fail(display = "remote/IPC call failed")]
    RemoteCallFailed,
    /// Returned when a password could not be verified as its unset
    /// server side
    #[fail(display = "cannot verify unset password")]
    PasswordUnset,
    /// Returned when the external account is not linked to a steam
    /// account
    #[fail(display = "external account not linked to steam")]
    ExternalAccountUnlinked,
    /// Returned when the PSN ticket is invalid
    #[fail(display = "PSN ticket invalid")]
    PSNTicketInvalid,
    /// Returned when the external account is already linked to a steam
    /// account
    #[fail(display = "external account already linked")]
    ExternalAccountAlreadyLinked,
    /// Returned when sync cannot resume due to a file conflict
    #[fail(display = "sync conflict between remote and local files")]
    RemoteFileConflict,
    /// Returned when the requested new password is not legal
    #[fail(display = "new password is illegal")]
    IllegalPassword,
    /// Returned when the new value is the same as the previous value
    #[fail(display = "new value is the same as old value")]
    SameAsPreviousValue,
    /// Returned when the account logon is denied to 2nd factor authentication
    /// failure
    #[fail(display = "2nd factor authentication failed")]
    AccountLogonDenied,
    /// Returned when the requested new password is the same as the
    /// previous password
    #[fail(display = "cannot use old password")]
    CannotUseOldPassword,
    /// Returned when logging in is denied due to an invalid auth code
    #[fail(display = "invalid login auth code")]
    InvalidLoginAuthCode,
    /// Returned when logging in fails due to no email being set for 2nd
    /// factor authentication
    #[fail(display = "no email for 2nd factor authentication")]
    AccountLogonDeniedNoMail,
    /// Undocumented
    #[fail(display = "hardware not capable of IPT")]
    HardwareNotCapableOfIPT,
    /// Undocumented
    #[fail(display = "IPT init error")]
    IPTInitError,
    /// Returned when a operation fails due to parental control restrictions
    /// for a user
    #[fail(display = "restricted due to parental controls")]
    ParentalControlRestricted,
    /// Returned when a facebook query returns an error
    #[fail(display = "facebook query failed")]
    FacebookQueryError,
    /// Returned when account login is denied due to an expired auth code
    #[fail(display = "login denied due to exipred auth code")]
    ExpiredLoginAuthCode,
    /// Undocumented
    #[fail(display = "IP login restriction failed")]
    IPLoginRestrictionFailed,
    /// Undocumented
    #[fail(display = "account locked down")]
    AccountLockedDown,
    /// Undocumented
    #[fail(display = "account logon denied verified email required")]
    AccountLogonDeniedVerifiedEmailRequired,
    /// Undocumented
    #[fail(display = "no matching URL")]
    NoMatchingURL,
    /// Returned when something fails to parse/has a missing field
    #[fail(display = "bad response")]
    BadResponse,
    /// Returned when a user cannot complete the action until they
    /// re-enter their password
    #[fail(display = "password re-entry required")]
    RequirePasswordReEntry,
    /// Returned when an entered value is outside the acceptable range
    #[fail(display = "value is out of range")]
    ValueOutOfRange,
    /// Returned when an error happens that the steamworks API didn't
    /// expect to happen
    #[fail(display = "unexpected error")]
    UnexpectedError,
    /// Returned when the requested service is disabled
    #[fail(display = "service disabled")]
    Disabled,
    /// Returned when the set of files submitted to the CEG server
    /// are not valid
    #[fail(display = "submitted files to CEG are invalid")]
    InvalidCEGSubmission,
    /// Returned when the device being used is not allowed to perform
    /// this action
    #[fail(display = "device is restricted from action")]
    RestrictedDevice,
    /// Returned when an action is prevented due to region restrictions
    #[fail(display = "region restrictions prevented action")]
    RegionLocked,
    /// Returned when an action failed due to a temporary rate limit
    #[fail(display = "temporary rate limit exceeded")]
    RateLimitExceeded,
    /// Returned when a account needs to use a two-factor code to login
    #[fail(display = "two-factor authetication required for login")]
    AccountLoginDeniedNeedTwoFactor,
    /// Returned when the item attempting to be accessed has been deleted
    #[fail(display = "item deleted")]
    ItemDeleted,
    /// Returned when the account login failed and you should throttle the
    /// response to the possible attacker
    #[fail(display = "account login denied, throttled")]
    AccountLoginDeniedThrottle,
    /// Returned when the two factor code provided mismatched the expected
    /// one
    #[fail(display = "two-factor code mismatched")]
    TwoFactorCodeMismatch,
    /// Returned when the two factor activation code mismatched the expected
    /// one
    #[fail(display = "two-factor activation code mismatched")]
    TwoFactorActivationCodeMismatch,
    /// Returned when the account has been associated with multiple partners
    #[fail(display = "account associated to multiple partners")]
    AccountAssociatedToMultiplePartners,
    /// Returned when the data wasn't modified
    #[fail(display = "data not modified")]
    NotModified,
    /// Returned when the account doesn't have a mobile device associated with
    /// it
    #[fail(display = "no mobile device associated with account")]
    NoMobileDevice,
    /// Returned when the current time is out of range or tolerance
    #[fail(display = "time not synced correctly")]
    TimeNotSynced,
    /// Returned when the sms code failed to validate
    #[fail(display = "sms code validation failed")]
    SmsCodeFailed,
    /// Returned when too many accounts are accessing the requested
    /// resource
    #[fail(display = "account limit exceeded for resource")]
    AccountLimitExceeded,
    /// Returned when there have been too many changes to the account
    #[fail(display = "account activity limit exceeded")]
    AccountActivityLimitExceeded,
    /// Returned when there have been too many changes to the phone
    #[fail(display = "phone activity limited exceeded")]
    PhoneActivityLimitExceeded,
    /// Returned when the refund can not be sent to the payment method
    /// and the steam wallet must be used
    #[fail(display = "must refund to wallet instead of payment method")]
    RefundToWallet,
    /// Returned when steam failed to send an email
    #[fail(display = "email sending failed")]
    EmailSendFailure,
    /// Returned when an action cannot be performed until the payment
    /// has settled
    #[fail(display = "action cannot be performed until payment has settled")]
    NotSettled,
    /// Returned when the user needs to provide a valid captcha
    #[fail(display = "valid captcha required")]
    NeedCaptcha,
    /// Returned when the game server login token owned by the token's owner
    /// been banned
    #[fail(display = "game server login token has been banned")]
    GSLTDenied,
    /// Returned when the game server owner has been denied for other reasons
    /// (account lock, community ban, vac ban, missing phone)
    #[fail(display = "game server owner denied")]
    GSOwnerDenied,
    /// Returned when the type of item attempted to be acted on is invalid
    #[fail(display = "invalid item type")]
    InvalidItemType,
    /// Returned when the IP address has been banned for taking this action
    #[fail(display = "IP banned from action")]
    IPBanned,
    /// Returned when the game server login token has expired
    ///
    /// It can be reset for use
    #[fail(display = "game server login token expired")]
    GSLTExpired,
    /// Returned when the user does not have the wallet funds to complete
    /// the action
    #[fail(display = "insufficient wallet funds for action")]
    InsufficientFunds,
    /// Returned when there are too many of the requested action pending
    /// already
    #[fail(display = "too many actions pending")]
    TooManyPending,
    /// Returned when there is no site licenses found
    #[fail(display = "no site licenses found")]
    NoSiteLicensesFound,
    /// Returned when WG could not send a response because it exceeded the
    /// max network send size
    #[fail(display = "WG network send size exceeded")]
    WGNetworkSendExceeded,
    /// Returned when the account is not mutually friends with the user
    #[fail(display = "account not friends")]
    AccountNotFriends,
    /// Returned when the user is limited
    #[fail(display = "user limited")]
    LimitedUserAccount,
}

impl From<sys::EResult> for SteamError {
    fn from(r: sys::EResult) -> Self {
        match r {
            sys::EResult_k_EResultOK => panic!("EResult_k_EResultOK isn't an error"),
            sys::EResult_k_EResultFail => SteamError::Generic,
            sys::EResult_k_EResultNoConnection => SteamError::NoConnection,
            sys::EResult_k_EResultInvalidPassword => SteamError::InvalidPassword,
            sys::EResult_k_EResultLoggedInElsewhere => SteamError::LoggedInElsewhere,
            sys::EResult_k_EResultInvalidProtocolVer => SteamError::InvalidProtocolVersion,
            sys::EResult_k_EResultInvalidParam => SteamError::InvalidParameter,
            sys::EResult_k_EResultFileNotFound => SteamError::FileNotFound,
            sys::EResult_k_EResultBusy => SteamError::Busy,
            sys::EResult_k_EResultInvalidState => SteamError::InvalidState,
            sys::EResult_k_EResultInvalidName => SteamError::InvalidName,
            sys::EResult_k_EResultInvalidEmail => SteamError::InvalidEmail,
            sys::EResult_k_EResultDuplicateName => SteamError::DuplicateName,
            sys::EResult_k_EResultAccessDenied => SteamError::AccessDenied,
            sys::EResult_k_EResultTimeout => SteamError::Timeout,
            sys::EResult_k_EResultBanned => SteamError::Banned,
            sys::EResult_k_EResultAccountNotFound => SteamError::AccountNotFound,
            sys::EResult_k_EResultInvalidSteamID => SteamError::InvalidSteamID,
            sys::EResult_k_EResultServiceUnavailable => SteamError::ServiceUnavailable,
            sys::EResult_k_EResultNotLoggedOn => SteamError::NotLoggedOn,
            sys::EResult_k_EResultPending => SteamError::Pending,
            sys::EResult_k_EResultEncryptionFailure => SteamError::EncryptionFailure,
            sys::EResult_k_EResultInsufficientPrivilege => SteamError::InsufficientPrivilege,
            sys::EResult_k_EResultLimitExceeded => SteamError::LimitExceeded,
            sys::EResult_k_EResultRevoked => SteamError::Revoked,
            sys::EResult_k_EResultExpired => SteamError::Expired,
            sys::EResult_k_EResultAlreadyRedeemed => SteamError::AlreadyRedeemed,
            sys::EResult_k_EResultDuplicateRequest => SteamError::DuplicateRequest,
            sys::EResult_k_EResultAlreadyOwned => SteamError::AlreadyOwned,
            sys::EResult_k_EResultIPNotFound => SteamError::IPNotFound,
            sys::EResult_k_EResultPersistFailed => SteamError::PersistFailed,
            sys::EResult_k_EResultLockingFailed => SteamError::LockingFailed,
            sys::EResult_k_EResultLogonSessionReplaced => SteamError::LogonSessionReplaced,
            sys::EResult_k_EResultConnectFailed => SteamError::ConnectFailed,
            sys::EResult_k_EResultHandshakeFailed => SteamError::HandshakeFailed,
            sys::EResult_k_EResultIOFailure => SteamError::IOFailure,
            sys::EResult_k_EResultRemoteDisconnect => SteamError::RemoteDisconnect,
            sys::EResult_k_EResultShoppingCartNotFound => SteamError::ShoppingCartNotFound,
            sys::EResult_k_EResultBlocked => SteamError::Blocked,
            sys::EResult_k_EResultIgnored => SteamError::Ignored,
            sys::EResult_k_EResultNoMatch => SteamError::NoMatch,
            sys::EResult_k_EResultAccountDisabled => SteamError::AccountDisabled,
            sys::EResult_k_EResultServiceReadOnly => SteamError::ServiceReadOnly,
            sys::EResult_k_EResultAccountNotFeatured => SteamError::AccountNotFeatured,
            sys::EResult_k_EResultAdministratorOK => SteamError::AdministratorOK,
            sys::EResult_k_EResultContentVersion => SteamError::ContentVersion,
            sys::EResult_k_EResultTryAnotherCM => SteamError::TryAnotherCM,
            sys::EResult_k_EResultPasswordRequiredToKickSession => SteamError::PasswordRequiredToKickSession,
            sys::EResult_k_EResultAlreadyLoggedInElsewhere => SteamError::AlreadyLoggedInElsewhere,
            sys::EResult_k_EResultSuspended => SteamError::Suspended,
            sys::EResult_k_EResultCancelled => SteamError::Cancelled,
            sys::EResult_k_EResultDataCorruption => SteamError::DataCorruption,
            sys::EResult_k_EResultDiskFull => SteamError::DiskFull,
            sys::EResult_k_EResultRemoteCallFailed => SteamError::RemoteCallFailed,
            sys::EResult_k_EResultPasswordUnset => SteamError::PasswordUnset,
            sys::EResult_k_EResultExternalAccountUnlinked => SteamError::ExternalAccountUnlinked,
            sys::EResult_k_EResultPSNTicketInvalid => SteamError::PSNTicketInvalid,
            sys::EResult_k_EResultExternalAccountAlreadyLinked => SteamError::ExternalAccountAlreadyLinked,
            sys::EResult_k_EResultRemoteFileConflict => SteamError::RemoteFileConflict,
            sys::EResult_k_EResultIllegalPassword => SteamError::IllegalPassword,
            sys::EResult_k_EResultSameAsPreviousValue => SteamError::SameAsPreviousValue,
            sys::EResult_k_EResultAccountLogonDenied => SteamError::AccountLogonDenied,
            sys::EResult_k_EResultCannotUseOldPassword => SteamError::CannotUseOldPassword,
            sys::EResult_k_EResultInvalidLoginAuthCode => SteamError::InvalidLoginAuthCode,
            sys::EResult_k_EResultAccountLogonDeniedNoMail => SteamError::AccountLogonDeniedNoMail,
            sys::EResult_k_EResultHardwareNotCapableOfIPT => SteamError::HardwareNotCapableOfIPT,
            sys::EResult_k_EResultIPTInitError => SteamError::IPTInitError,
            sys::EResult_k_EResultParentalControlRestricted => SteamError::ParentalControlRestricted,
            sys::EResult_k_EResultFacebookQueryError => SteamError::FacebookQueryError,
            sys::EResult_k_EResultExpiredLoginAuthCode => SteamError::ExpiredLoginAuthCode,
            sys::EResult_k_EResultIPLoginRestrictionFailed => SteamError::IPLoginRestrictionFailed,
            sys::EResult_k_EResultAccountLockedDown => SteamError::AccountLockedDown,
            sys::EResult_k_EResultAccountLogonDeniedVerifiedEmailRequired => SteamError::AccountLogonDeniedVerifiedEmailRequired,
            sys::EResult_k_EResultNoMatchingURL => SteamError::NoMatchingURL,
            sys::EResult_k_EResultBadResponse => SteamError::BadResponse,
            sys::EResult_k_EResultRequirePasswordReEntry => SteamError::RequirePasswordReEntry,
            sys::EResult_k_EResultValueOutOfRange => SteamError::ValueOutOfRange,
            sys::EResult_k_EResultUnexpectedError => SteamError::UnexpectedError,
            sys::EResult_k_EResultDisabled => SteamError::Disabled,
            sys::EResult_k_EResultInvalidCEGSubmission => SteamError::InvalidCEGSubmission,
            sys::EResult_k_EResultRestrictedDevice => SteamError::RestrictedDevice,
            sys::EResult_k_EResultRegionLocked => SteamError::RegionLocked,
            sys::EResult_k_EResultRateLimitExceeded => SteamError::RateLimitExceeded,
            sys::EResult_k_EResultAccountLoginDeniedNeedTwoFactor => SteamError::AccountLoginDeniedNeedTwoFactor,
            sys::EResult_k_EResultItemDeleted => SteamError::ItemDeleted,
            sys::EResult_k_EResultAccountLoginDeniedThrottle => SteamError::AccountLoginDeniedThrottle,
            sys::EResult_k_EResultTwoFactorCodeMismatch => SteamError::TwoFactorCodeMismatch,
            sys::EResult_k_EResultTwoFactorActivationCodeMismatch => SteamError::TwoFactorActivationCodeMismatch,
            sys::EResult_k_EResultAccountAssociatedToMultiplePartners => SteamError::AccountAssociatedToMultiplePartners,
            sys::EResult_k_EResultNotModified => SteamError::NotModified,
            sys::EResult_k_EResultNoMobileDevice => SteamError::NoMobileDevice,
            sys::EResult_k_EResultTimeNotSynced => SteamError::TimeNotSynced,
            sys::EResult_k_EResultSmsCodeFailed => SteamError::SmsCodeFailed,
            sys::EResult_k_EResultAccountLimitExceeded => SteamError::AccountLimitExceeded,
            sys::EResult_k_EResultAccountActivityLimitExceeded => SteamError::AccountActivityLimitExceeded,
            sys::EResult_k_EResultPhoneActivityLimitExceeded => SteamError::PhoneActivityLimitExceeded,
            sys::EResult_k_EResultRefundToWallet => SteamError::RefundToWallet,
            sys::EResult_k_EResultEmailSendFailure => SteamError::EmailSendFailure,
            sys::EResult_k_EResultNotSettled => SteamError::NotSettled,
            sys::EResult_k_EResultNeedCaptcha => SteamError::NeedCaptcha,
            sys::EResult_k_EResultGSLTDenied => SteamError::GSLTDenied,
            sys::EResult_k_EResultGSOwnerDenied => SteamError::GSOwnerDenied,
            sys::EResult_k_EResultInvalidItemType => SteamError::InvalidItemType,
            sys::EResult_k_EResultIPBanned => SteamError::IPBanned,
            sys::EResult_k_EResultGSLTExpired => SteamError::GSLTExpired,
            sys::EResult_k_EResultInsufficientFunds => SteamError::InsufficientFunds,
            sys::EResult_k_EResultTooManyPending => SteamError::TooManyPending,
            sys::EResult_k_EResultNoSiteLicensesFound => SteamError::NoSiteLicensesFound,
            sys::EResult_k_EResultWGNetworkSendExceeded => SteamError::WGNetworkSendExceeded,
            sys::EResult_k_EResultAccountNotFriends => SteamError::AccountNotFriends,
            sys::EResult_k_EResultLimitedUserAccount => SteamError::LimitedUserAccount,
            _ => unreachable!(),
        }
    }
}