#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::sys;
use std::convert::TryFrom;

/// Covers errors that can be returned by the steamworks API
///
/// Documentation is based on official documentation which doesn't
/// always explain when an error could be returned or its meaning.
#[derive(Copy, Clone, Debug, Error, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SteamError {
    /// Returned if the steamworks API fails to initialize.
    #[error("failed to init the steamworks API")]
    InitFailed,
    /// Returned if the steamworks API fails to perform an action
    #[error("a generic failure from the steamworks API")]
    Generic,
    /// Returned when steam fails performing a network request
    #[error("there isn't a network connection to steam or it failed to connect")]
    NoConnection,
    /// Return when the password or ticked used is invalid
    #[error("password or ticket is invalid")]
    InvalidPassword,
    /// Returned when the user is already logged in at another location
    #[error("user logged in elsewhere")]
    LoggedInElsewhere,
    /// Returned when the protocol version is incorrect
    #[error("the protocol version is incorrect")]
    InvalidProtocolVersion,
    /// Returned when a passed parameter is invalid
    #[error("a parameter is invalid")]
    InvalidParameter,
    /// Returned when a file is not found
    #[error("a file was not found")]
    FileNotFound,
    /// Returned when the called method was busy
    ///
    /// No action was performed
    #[error("method busy")]
    Busy,
    /// Returned when the called object was in an
    /// invalid state
    #[error("object in invalid state")]
    InvalidState,
    /// Returned when the name is invalid
    #[error("name is invalid")]
    InvalidName,
    /// Returned when the email is invalid
    #[error("email is invalid")]
    InvalidEmail,
    /// Returned when the name is not unique
    #[error("name is not unique")]
    DuplicateName,
    /// Returned when access is denied
    #[error("access denied")]
    AccessDenied,
    /// Returned when the operation timed out
    #[error("operation timed out")]
    Timeout,
    /// Returned when the user is VAC2 banned
    #[error("VAC2 banned")]
    Banned,
    /// Returned when the account is not found
    #[error("account not found")]
    AccountNotFound,
    /// Returned when the passed steam id is invalid
    #[error("steamID is invalid")]
    InvalidSteamID,
    /// Returned when the requested service in unavailable
    #[error("requested service is unavailable")]
    ServiceUnavailable,
    /// Returned when the user is not logged on
    #[error("user not logged on")]
    NotLoggedOn,
    /// Returned when the request is pending (e.g. in progress/waiting)
    #[error("request is pending")]
    Pending,
    /// Returned when encryption or decryption fails
    #[error("encryption/decryption failed")]
    EncryptionFailure,
    /// Returned when you have insufficient privilege to perform
    /// the action
    #[error("insufficient privilege")]
    InsufficientPrivilege,
    /// Returned when you have hit the API limits
    #[error("limit exceeded")]
    LimitExceeded,
    /// Returned when the user's access has been revoked (e.g. revoked
    /// guess passes)
    #[error("access revoked")]
    Revoked,
    /// Returned when the user's access has expired
    #[error("access expired")]
    Expired,
    /// Returned when the licence/guest pass has already been redeemed
    #[error("licence/guest pass already redeemed")]
    AlreadyRedeemed,
    /// Returned when the requested action is a duplicate and has
    /// already occurred.
    ///
    /// The action will be ignored
    #[error("request is a duplicate")]
    DuplicateRequest,
    /// Returned when all the games in the guest pass are already
    /// owned by the user
    #[error("all games requested already owned")]
    AlreadyOwned,
    /// Returned when the ip address is not found
    #[error("ip address not found")]
    IPNotFound,
    /// Returned when the change failed to write to the data store
    #[error("failed to write change")]
    PersistFailed,
    /// Returned when the operation failed to acquire the access lock
    #[error("failed to acquire access lock")]
    LockingFailed,
    /// Undocumented
    #[error("logon session replaced")]
    LogonSessionReplaced,
    /// Undocumented
    #[error("connect failed")]
    ConnectFailed,
    /// Undocumented
    #[error("handshake failed")]
    HandshakeFailed,
    /// Undocumented
    #[error("IO failure")]
    IOFailure,
    /// Undocumented
    #[error("remote disconnect")]
    RemoteDisconnect,
    /// Returned when the requested shopping cart wasn't found
    #[error("failed to find the requested shopping cart")]
    ShoppingCartNotFound,
    /// Returned when the user blocks an action
    #[error("action blocked")]
    Blocked,
    /// Returned when the target user is ignoring the sender
    #[error("target is ignoring sender")]
    Ignored,
    /// Returned when nothing matching the request is found
    #[error("no matches found")]
    NoMatch,
    /// Undocumented
    #[error("account disabled")]
    AccountDisabled,
    /// Returned when the service isn't accepting content changes at
    /// this moment
    #[error("service is read only")]
    ServiceReadOnly,
    /// Returned when the account doesn't have value so the feature
    /// isn't available
    #[error("account not featured")]
    AccountNotFeatured,
    /// Allowed to take this action but only because the requester is
    /// an admin
    #[error("administrator ok")]
    AdministratorOK,
    /// Returned when there is a version mismatch in content transmitted
    /// within the steam protocol
    #[error("version mismatch with transmitted content")]
    ContentVersion,
    /// Returned when the current CM cannot service the user's request.
    ///
    /// The user should try another.
    #[error("CM cannot service user")]
    TryAnotherCM,
    /// Returned when the user is already logged in elsewhere and the
    /// cached credential login failed.
    #[error("user already logged in, cached login failed")]
    PasswordRequiredToKickSession,
    /// Returned when the user is already logged in elsewhere, you
    /// must wait before trying again
    #[error("user already logged in, please wait")]
    AlreadyLoggedInElsewhere,
    /// Returned when a long running operation (e.g. download) is
    /// suspended/paused.
    #[error("operation suspended/paused")]
    Suspended,
    /// Returned when an operation is cancelled
    #[error("operation cancelled")]
    Cancelled,
    /// Returned when an operation is cancelled due to data corruption
    #[error("operation cancelled due to data corruption")]
    DataCorruption,
    /// Returned when an operation is cancelled due to running out of disk
    /// space
    #[error("operation cancelled due to the disk being full")]
    DiskFull,
    /// Returned when a remote call or an IPC call failed
    #[error("remote/IPC call failed")]
    RemoteCallFailed,
    /// Returned when a password could not be verified as its unset
    /// server side
    #[error("cannot verify unset password")]
    PasswordUnset,
    /// Returned when the external account is not linked to a steam
    /// account
    #[error("external account not linked to steam")]
    ExternalAccountUnlinked,
    /// Returned when the PSN ticket is invalid
    #[error("PSN ticket invalid")]
    PSNTicketInvalid,
    /// Returned when the external account is already linked to a steam
    /// account
    #[error("external account already linked")]
    ExternalAccountAlreadyLinked,
    /// Returned when sync cannot resume due to a file conflict
    #[error("sync conflict between remote and local files")]
    RemoteFileConflict,
    /// Returned when the requested new password is not legal
    #[error("new password is illegal")]
    IllegalPassword,
    /// Returned when the new value is the same as the previous value
    #[error("new value is the same as old value")]
    SameAsPreviousValue,
    /// Returned when the account logon is denied to 2nd factor authentication
    /// failure
    #[error("2nd factor authentication failed")]
    AccountLogonDenied,
    /// Returned when the requested new password is the same as the
    /// previous password
    #[error("cannot use old password")]
    CannotUseOldPassword,
    /// Returned when logging in is denied due to an invalid auth code
    #[error("invalid login auth code")]
    InvalidLoginAuthCode,
    /// Returned when logging in fails due to no email being set for 2nd
    /// factor authentication
    #[error("no email for 2nd factor authentication")]
    AccountLogonDeniedNoMail,
    /// Undocumented
    #[error("hardware not capable of IPT")]
    HardwareNotCapableOfIPT,
    /// Undocumented
    #[error("IPT init error")]
    IPTInitError,
    /// Returned when a operation fails due to parental control restrictions
    /// for a user
    #[error("restricted due to parental controls")]
    ParentalControlRestricted,
    /// Returned when a facebook query returns an error
    #[error("facebook query failed")]
    FacebookQueryError,
    /// Returned when account login is denied due to an expired auth code
    #[error("login denied due to exipred auth code")]
    ExpiredLoginAuthCode,
    /// Undocumented
    #[error("IP login restriction failed")]
    IPLoginRestrictionFailed,
    /// Undocumented
    #[error("account locked down")]
    AccountLockedDown,
    /// Undocumented
    #[error("account logon denied verified email required")]
    AccountLogonDeniedVerifiedEmailRequired,
    /// Undocumented
    #[error("no matching URL")]
    NoMatchingURL,
    /// Returned when something fails to parse/has a missing field
    #[error("bad response")]
    BadResponse,
    /// Returned when a user cannot complete the action until they
    /// re-enter their password
    #[error("password re-entry required")]
    RequirePasswordReEntry,
    /// Returned when an entered value is outside the acceptable range
    #[error("value is out of range")]
    ValueOutOfRange,
    /// Returned when an error happens that the steamworks API didn't
    /// expect to happen
    #[error("unexpected error")]
    UnexpectedError,
    /// Returned when the requested service is disabled
    #[error("service disabled")]
    Disabled,
    /// Returned when the set of files submitted to the CEG server
    /// are not valid
    #[error("submitted files to CEG are invalid")]
    InvalidCEGSubmission,
    /// Returned when the device being used is not allowed to perform
    /// this action
    #[error("device is restricted from action")]
    RestrictedDevice,
    /// Returned when an action is prevented due to region restrictions
    #[error("region restrictions prevented action")]
    RegionLocked,
    /// Returned when an action failed due to a temporary rate limit
    #[error("temporary rate limit exceeded")]
    RateLimitExceeded,
    /// Returned when a account needs to use a two-factor code to login
    #[error("two-factor authetication required for login")]
    AccountLoginDeniedNeedTwoFactor,
    /// Returned when the item attempting to be accessed has been deleted
    #[error("item deleted")]
    ItemDeleted,
    /// Returned when the account login failed and you should throttle the
    /// response to the possible attacker
    #[error("account login denied, throttled")]
    AccountLoginDeniedThrottle,
    /// Returned when the two factor code provided mismatched the expected
    /// one
    #[error("two-factor code mismatched")]
    TwoFactorCodeMismatch,
    /// Returned when the two factor activation code mismatched the expected
    /// one
    #[error("two-factor activation code mismatched")]
    TwoFactorActivationCodeMismatch,
    /// Returned when the account has been associated with multiple partners
    #[error("account associated to multiple partners")]
    AccountAssociatedToMultiplePartners,
    /// Returned when the data wasn't modified
    #[error("data not modified")]
    NotModified,
    /// Returned when the account doesn't have a mobile device associated with
    /// it
    #[error("no mobile device associated with account")]
    NoMobileDevice,
    /// Returned when the current time is out of range or tolerance
    #[error("time not synced correctly")]
    TimeNotSynced,
    /// Returned when the sms code failed to validate
    #[error("sms code validation failed")]
    SmsCodeFailed,
    /// Returned when too many accounts are accessing the requested
    /// resource
    #[error("account limit exceeded for resource")]
    AccountLimitExceeded,
    /// Returned when there have been too many changes to the account
    #[error("account activity limit exceeded")]
    AccountActivityLimitExceeded,
    /// Returned when there have been too many changes to the phone
    #[error("phone activity limited exceeded")]
    PhoneActivityLimitExceeded,
    /// Returned when the refund can not be sent to the payment method
    /// and the steam wallet must be used
    #[error("must refund to wallet instead of payment method")]
    RefundToWallet,
    /// Returned when steam failed to send an email
    #[error("email sending failed")]
    EmailSendFailure,
    /// Returned when an action cannot be performed until the payment
    /// has settled
    #[error("action cannot be performed until payment has settled")]
    NotSettled,
    /// Returned when the user needs to provide a valid captcha
    #[error("valid captcha required")]
    NeedCaptcha,
    /// Returned when the game server login token owned by the token's owner
    /// been banned
    #[error("game server login token has been banned")]
    GSLTDenied,
    /// Returned when the game server owner has been denied for other reasons
    /// (account lock, community ban, vac ban, missing phone)
    #[error("game server owner denied")]
    GSOwnerDenied,
    /// Returned when the type of item attempted to be acted on is invalid
    #[error("invalid item type")]
    InvalidItemType,
    /// Returned when the IP address has been banned for taking this action
    #[error("IP banned from action")]
    IPBanned,
    /// Returned when the game server login token has expired
    ///
    /// It can be reset for use
    #[error("game server login token expired")]
    GSLTExpired,
    /// Returned when the user does not have the wallet funds to complete
    /// the action
    #[error("insufficient wallet funds for action")]
    InsufficientFunds,
    /// Returned when there are too many of the requested action pending
    /// already
    #[error("too many actions pending")]
    TooManyPending,
    /// Returned when there is no site licenses found
    #[error("no site licenses found")]
    NoSiteLicensesFound,
    /// Returned when WG could not send a response because it exceeded the
    /// max network send size
    #[error("WG network send size exceeded")]
    WGNetworkSendExceeded,
}

impl From<sys::EResult> for SteamError {
    fn from(r: sys::EResult) -> Self {
        match r {
            sys::EResult::k_EResultOK => panic!("EResult::k_EResultOK isn't an error"),
            sys::EResult::k_EResultFail => SteamError::Generic,
            sys::EResult::k_EResultNoConnection => SteamError::NoConnection,
            sys::EResult::k_EResultInvalidPassword => SteamError::InvalidPassword,
            sys::EResult::k_EResultLoggedInElsewhere => SteamError::LoggedInElsewhere,
            sys::EResult::k_EResultInvalidProtocolVer => SteamError::InvalidProtocolVersion,
            sys::EResult::k_EResultInvalidParam => SteamError::InvalidParameter,
            sys::EResult::k_EResultFileNotFound => SteamError::FileNotFound,
            sys::EResult::k_EResultBusy => SteamError::Busy,
            sys::EResult::k_EResultInvalidState => SteamError::InvalidState,
            sys::EResult::k_EResultInvalidName => SteamError::InvalidName,
            sys::EResult::k_EResultInvalidEmail => SteamError::InvalidEmail,
            sys::EResult::k_EResultDuplicateName => SteamError::DuplicateName,
            sys::EResult::k_EResultAccessDenied => SteamError::AccessDenied,
            sys::EResult::k_EResultTimeout => SteamError::Timeout,
            sys::EResult::k_EResultBanned => SteamError::Banned,
            sys::EResult::k_EResultAccountNotFound => SteamError::AccountNotFound,
            sys::EResult::k_EResultInvalidSteamID => SteamError::InvalidSteamID,
            sys::EResult::k_EResultServiceUnavailable => SteamError::ServiceUnavailable,
            sys::EResult::k_EResultNotLoggedOn => SteamError::NotLoggedOn,
            sys::EResult::k_EResultPending => SteamError::Pending,
            sys::EResult::k_EResultEncryptionFailure => SteamError::EncryptionFailure,
            sys::EResult::k_EResultInsufficientPrivilege => SteamError::InsufficientPrivilege,
            sys::EResult::k_EResultLimitExceeded => SteamError::LimitExceeded,
            sys::EResult::k_EResultRevoked => SteamError::Revoked,
            sys::EResult::k_EResultExpired => SteamError::Expired,
            sys::EResult::k_EResultAlreadyRedeemed => SteamError::AlreadyRedeemed,
            sys::EResult::k_EResultDuplicateRequest => SteamError::DuplicateRequest,
            sys::EResult::k_EResultAlreadyOwned => SteamError::AlreadyOwned,
            sys::EResult::k_EResultIPNotFound => SteamError::IPNotFound,
            sys::EResult::k_EResultPersistFailed => SteamError::PersistFailed,
            sys::EResult::k_EResultLockingFailed => SteamError::LockingFailed,
            sys::EResult::k_EResultLogonSessionReplaced => SteamError::LogonSessionReplaced,
            sys::EResult::k_EResultConnectFailed => SteamError::ConnectFailed,
            sys::EResult::k_EResultHandshakeFailed => SteamError::HandshakeFailed,
            sys::EResult::k_EResultIOFailure => SteamError::IOFailure,
            sys::EResult::k_EResultRemoteDisconnect => SteamError::RemoteDisconnect,
            sys::EResult::k_EResultShoppingCartNotFound => SteamError::ShoppingCartNotFound,
            sys::EResult::k_EResultBlocked => SteamError::Blocked,
            sys::EResult::k_EResultIgnored => SteamError::Ignored,
            sys::EResult::k_EResultNoMatch => SteamError::NoMatch,
            sys::EResult::k_EResultAccountDisabled => SteamError::AccountDisabled,
            sys::EResult::k_EResultServiceReadOnly => SteamError::ServiceReadOnly,
            sys::EResult::k_EResultAccountNotFeatured => SteamError::AccountNotFeatured,
            sys::EResult::k_EResultAdministratorOK => SteamError::AdministratorOK,
            sys::EResult::k_EResultContentVersion => SteamError::ContentVersion,
            sys::EResult::k_EResultTryAnotherCM => SteamError::TryAnotherCM,
            sys::EResult::k_EResultPasswordRequiredToKickSession => {
                SteamError::PasswordRequiredToKickSession
            }
            sys::EResult::k_EResultAlreadyLoggedInElsewhere => SteamError::AlreadyLoggedInElsewhere,
            sys::EResult::k_EResultSuspended => SteamError::Suspended,
            sys::EResult::k_EResultCancelled => SteamError::Cancelled,
            sys::EResult::k_EResultDataCorruption => SteamError::DataCorruption,
            sys::EResult::k_EResultDiskFull => SteamError::DiskFull,
            sys::EResult::k_EResultRemoteCallFailed => SteamError::RemoteCallFailed,
            sys::EResult::k_EResultPasswordUnset => SteamError::PasswordUnset,
            sys::EResult::k_EResultExternalAccountUnlinked => SteamError::ExternalAccountUnlinked,
            sys::EResult::k_EResultPSNTicketInvalid => SteamError::PSNTicketInvalid,
            sys::EResult::k_EResultExternalAccountAlreadyLinked => {
                SteamError::ExternalAccountAlreadyLinked
            }
            sys::EResult::k_EResultRemoteFileConflict => SteamError::RemoteFileConflict,
            sys::EResult::k_EResultIllegalPassword => SteamError::IllegalPassword,
            sys::EResult::k_EResultSameAsPreviousValue => SteamError::SameAsPreviousValue,
            sys::EResult::k_EResultAccountLogonDenied => SteamError::AccountLogonDenied,
            sys::EResult::k_EResultCannotUseOldPassword => SteamError::CannotUseOldPassword,
            sys::EResult::k_EResultInvalidLoginAuthCode => SteamError::InvalidLoginAuthCode,
            sys::EResult::k_EResultAccountLogonDeniedNoMail => SteamError::AccountLogonDeniedNoMail,
            sys::EResult::k_EResultHardwareNotCapableOfIPT => SteamError::HardwareNotCapableOfIPT,
            sys::EResult::k_EResultIPTInitError => SteamError::IPTInitError,
            sys::EResult::k_EResultParentalControlRestricted => {
                SteamError::ParentalControlRestricted
            }
            sys::EResult::k_EResultFacebookQueryError => SteamError::FacebookQueryError,
            sys::EResult::k_EResultExpiredLoginAuthCode => SteamError::ExpiredLoginAuthCode,
            sys::EResult::k_EResultIPLoginRestrictionFailed => SteamError::IPLoginRestrictionFailed,
            sys::EResult::k_EResultAccountLockedDown => SteamError::AccountLockedDown,
            sys::EResult::k_EResultAccountLogonDeniedVerifiedEmailRequired => {
                SteamError::AccountLogonDeniedVerifiedEmailRequired
            }
            sys::EResult::k_EResultNoMatchingURL => SteamError::NoMatchingURL,
            sys::EResult::k_EResultBadResponse => SteamError::BadResponse,
            sys::EResult::k_EResultRequirePasswordReEntry => SteamError::RequirePasswordReEntry,
            sys::EResult::k_EResultValueOutOfRange => SteamError::ValueOutOfRange,
            sys::EResult::k_EResultUnexpectedError => SteamError::UnexpectedError,
            sys::EResult::k_EResultDisabled => SteamError::Disabled,
            sys::EResult::k_EResultInvalidCEGSubmission => SteamError::InvalidCEGSubmission,
            sys::EResult::k_EResultRestrictedDevice => SteamError::RestrictedDevice,
            sys::EResult::k_EResultRegionLocked => SteamError::RegionLocked,
            sys::EResult::k_EResultRateLimitExceeded => SteamError::RateLimitExceeded,
            sys::EResult::k_EResultAccountLoginDeniedNeedTwoFactor => {
                SteamError::AccountLoginDeniedNeedTwoFactor
            }
            sys::EResult::k_EResultItemDeleted => SteamError::ItemDeleted,
            sys::EResult::k_EResultAccountLoginDeniedThrottle => {
                SteamError::AccountLoginDeniedThrottle
            }
            sys::EResult::k_EResultTwoFactorCodeMismatch => SteamError::TwoFactorCodeMismatch,
            sys::EResult::k_EResultTwoFactorActivationCodeMismatch => {
                SteamError::TwoFactorActivationCodeMismatch
            }
            sys::EResult::k_EResultAccountAssociatedToMultiplePartners => {
                SteamError::AccountAssociatedToMultiplePartners
            }
            sys::EResult::k_EResultNotModified => SteamError::NotModified,
            sys::EResult::k_EResultNoMobileDevice => SteamError::NoMobileDevice,
            sys::EResult::k_EResultTimeNotSynced => SteamError::TimeNotSynced,
            sys::EResult::k_EResultSmsCodeFailed => SteamError::SmsCodeFailed,
            sys::EResult::k_EResultAccountLimitExceeded => SteamError::AccountLimitExceeded,
            sys::EResult::k_EResultAccountActivityLimitExceeded => {
                SteamError::AccountActivityLimitExceeded
            }
            sys::EResult::k_EResultPhoneActivityLimitExceeded => {
                SteamError::PhoneActivityLimitExceeded
            }
            sys::EResult::k_EResultRefundToWallet => SteamError::RefundToWallet,
            sys::EResult::k_EResultEmailSendFailure => SteamError::EmailSendFailure,
            sys::EResult::k_EResultNotSettled => SteamError::NotSettled,
            sys::EResult::k_EResultNeedCaptcha => SteamError::NeedCaptcha,
            sys::EResult::k_EResultGSLTDenied => SteamError::GSLTDenied,
            sys::EResult::k_EResultGSOwnerDenied => SteamError::GSOwnerDenied,
            sys::EResult::k_EResultInvalidItemType => SteamError::InvalidItemType,
            sys::EResult::k_EResultIPBanned => SteamError::IPBanned,
            sys::EResult::k_EResultGSLTExpired => SteamError::GSLTExpired,
            sys::EResult::k_EResultInsufficientFunds => SteamError::InsufficientFunds,
            sys::EResult::k_EResultTooManyPending => SteamError::TooManyPending,
            sys::EResult::k_EResultNoSiteLicensesFound => SteamError::NoSiteLicensesFound,
            sys::EResult::k_EResultWGNetworkSendExceeded => SteamError::WGNetworkSendExceeded,
            _ => unreachable!(),
        }
    }
}

impl TryFrom<i64> for SteamError {
    type Error = InvalidErrorCode;

    fn try_from(r: i64) -> Result<Self, Self::Error> {
        let error = match r {
            x if x == sys::EResult::k_EResultFail as i64 => SteamError::Generic,
            x if x == sys::EResult::k_EResultNoConnection as i64 => SteamError::NoConnection,
            x if x == sys::EResult::k_EResultInvalidPassword as i64 => SteamError::InvalidPassword,
            x if x == sys::EResult::k_EResultLoggedInElsewhere as i64 => {
                SteamError::LoggedInElsewhere
            }
            x if x == sys::EResult::k_EResultInvalidProtocolVer as i64 => {
                SteamError::InvalidProtocolVersion
            }
            x if x == sys::EResult::k_EResultInvalidParam as i64 => SteamError::InvalidParameter,
            x if x == sys::EResult::k_EResultFileNotFound as i64 => SteamError::FileNotFound,
            x if x == sys::EResult::k_EResultBusy as i64 => SteamError::Busy,
            x if x == sys::EResult::k_EResultInvalidState as i64 => SteamError::InvalidState,
            x if x == sys::EResult::k_EResultInvalidName as i64 => SteamError::InvalidName,
            x if x == sys::EResult::k_EResultInvalidEmail as i64 => SteamError::InvalidEmail,
            x if x == sys::EResult::k_EResultDuplicateName as i64 => SteamError::DuplicateName,
            x if x == sys::EResult::k_EResultAccessDenied as i64 => SteamError::AccessDenied,
            x if x == sys::EResult::k_EResultTimeout as i64 => SteamError::Timeout,
            x if x == sys::EResult::k_EResultBanned as i64 => SteamError::Banned,
            x if x == sys::EResult::k_EResultAccountNotFound as i64 => SteamError::AccountNotFound,
            x if x == sys::EResult::k_EResultInvalidSteamID as i64 => SteamError::InvalidSteamID,
            x if x == sys::EResult::k_EResultServiceUnavailable as i64 => {
                SteamError::ServiceUnavailable
            }
            x if x == sys::EResult::k_EResultNotLoggedOn as i64 => SteamError::NotLoggedOn,
            x if x == sys::EResult::k_EResultPending as i64 => SteamError::Pending,
            x if x == sys::EResult::k_EResultEncryptionFailure as i64 => {
                SteamError::EncryptionFailure
            }
            x if x == sys::EResult::k_EResultInsufficientPrivilege as i64 => {
                SteamError::InsufficientPrivilege
            }
            x if x == sys::EResult::k_EResultLimitExceeded as i64 => SteamError::LimitExceeded,
            x if x == sys::EResult::k_EResultRevoked as i64 => SteamError::Revoked,
            x if x == sys::EResult::k_EResultExpired as i64 => SteamError::Expired,
            x if x == sys::EResult::k_EResultAlreadyRedeemed as i64 => SteamError::AlreadyRedeemed,
            x if x == sys::EResult::k_EResultDuplicateRequest as i64 => {
                SteamError::DuplicateRequest
            }
            x if x == sys::EResult::k_EResultAlreadyOwned as i64 => SteamError::AlreadyOwned,
            x if x == sys::EResult::k_EResultIPNotFound as i64 => SteamError::IPNotFound,
            x if x == sys::EResult::k_EResultPersistFailed as i64 => SteamError::PersistFailed,
            x if x == sys::EResult::k_EResultLockingFailed as i64 => SteamError::LockingFailed,
            x if x == sys::EResult::k_EResultLogonSessionReplaced as i64 => {
                SteamError::LogonSessionReplaced
            }
            x if x == sys::EResult::k_EResultConnectFailed as i64 => SteamError::ConnectFailed,
            x if x == sys::EResult::k_EResultHandshakeFailed as i64 => SteamError::HandshakeFailed,
            x if x == sys::EResult::k_EResultIOFailure as i64 => SteamError::IOFailure,
            x if x == sys::EResult::k_EResultRemoteDisconnect as i64 => {
                SteamError::RemoteDisconnect
            }
            x if x == sys::EResult::k_EResultShoppingCartNotFound as i64 => {
                SteamError::ShoppingCartNotFound
            }
            x if x == sys::EResult::k_EResultBlocked as i64 => SteamError::Blocked,
            x if x == sys::EResult::k_EResultIgnored as i64 => SteamError::Ignored,
            x if x == sys::EResult::k_EResultNoMatch as i64 => SteamError::NoMatch,
            x if x == sys::EResult::k_EResultAccountDisabled as i64 => SteamError::AccountDisabled,
            x if x == sys::EResult::k_EResultServiceReadOnly as i64 => SteamError::ServiceReadOnly,
            x if x == sys::EResult::k_EResultAccountNotFeatured as i64 => {
                SteamError::AccountNotFeatured
            }
            x if x == sys::EResult::k_EResultAdministratorOK as i64 => SteamError::AdministratorOK,
            x if x == sys::EResult::k_EResultContentVersion as i64 => SteamError::ContentVersion,
            x if x == sys::EResult::k_EResultTryAnotherCM as i64 => SteamError::TryAnotherCM,
            x if x == sys::EResult::k_EResultPasswordRequiredToKickSession as i64 => {
                SteamError::PasswordRequiredToKickSession
            }
            x if x == sys::EResult::k_EResultAlreadyLoggedInElsewhere as i64 => {
                SteamError::AlreadyLoggedInElsewhere
            }
            x if x == sys::EResult::k_EResultSuspended as i64 => SteamError::Suspended,
            x if x == sys::EResult::k_EResultCancelled as i64 => SteamError::Cancelled,
            x if x == sys::EResult::k_EResultDataCorruption as i64 => SteamError::DataCorruption,
            x if x == sys::EResult::k_EResultDiskFull as i64 => SteamError::DiskFull,
            x if x == sys::EResult::k_EResultRemoteCallFailed as i64 => {
                SteamError::RemoteCallFailed
            }
            x if x == sys::EResult::k_EResultPasswordUnset as i64 => SteamError::PasswordUnset,
            x if x == sys::EResult::k_EResultExternalAccountUnlinked as i64 => {
                SteamError::ExternalAccountUnlinked
            }
            x if x == sys::EResult::k_EResultPSNTicketInvalid as i64 => {
                SteamError::PSNTicketInvalid
            }
            x if x == sys::EResult::k_EResultExternalAccountAlreadyLinked as i64 => {
                SteamError::ExternalAccountAlreadyLinked
            }
            x if x == sys::EResult::k_EResultRemoteFileConflict as i64 => {
                SteamError::RemoteFileConflict
            }
            x if x == sys::EResult::k_EResultIllegalPassword as i64 => SteamError::IllegalPassword,
            x if x == sys::EResult::k_EResultSameAsPreviousValue as i64 => {
                SteamError::SameAsPreviousValue
            }
            x if x == sys::EResult::k_EResultAccountLogonDenied as i64 => {
                SteamError::AccountLogonDenied
            }
            x if x == sys::EResult::k_EResultCannotUseOldPassword as i64 => {
                SteamError::CannotUseOldPassword
            }
            x if x == sys::EResult::k_EResultInvalidLoginAuthCode as i64 => {
                SteamError::InvalidLoginAuthCode
            }
            x if x == sys::EResult::k_EResultAccountLogonDeniedNoMail as i64 => {
                SteamError::AccountLogonDeniedNoMail
            }
            x if x == sys::EResult::k_EResultHardwareNotCapableOfIPT as i64 => {
                SteamError::HardwareNotCapableOfIPT
            }
            x if x == sys::EResult::k_EResultIPTInitError as i64 => SteamError::IPTInitError,
            x if x == sys::EResult::k_EResultParentalControlRestricted as i64 => {
                SteamError::ParentalControlRestricted
            }
            x if x == sys::EResult::k_EResultFacebookQueryError as i64 => {
                SteamError::FacebookQueryError
            }
            x if x == sys::EResult::k_EResultExpiredLoginAuthCode as i64 => {
                SteamError::ExpiredLoginAuthCode
            }
            x if x == sys::EResult::k_EResultIPLoginRestrictionFailed as i64 => {
                SteamError::IPLoginRestrictionFailed
            }
            x if x == sys::EResult::k_EResultAccountLockedDown as i64 => {
                SteamError::AccountLockedDown
            }
            x if x == sys::EResult::k_EResultAccountLogonDeniedVerifiedEmailRequired as i64 => {
                SteamError::AccountLogonDeniedVerifiedEmailRequired
            }
            x if x == sys::EResult::k_EResultNoMatchingURL as i64 => SteamError::NoMatchingURL,
            x if x == sys::EResult::k_EResultBadResponse as i64 => SteamError::BadResponse,
            x if x == sys::EResult::k_EResultRequirePasswordReEntry as i64 => {
                SteamError::RequirePasswordReEntry
            }
            x if x == sys::EResult::k_EResultValueOutOfRange as i64 => SteamError::ValueOutOfRange,
            x if x == sys::EResult::k_EResultUnexpectedError as i64 => SteamError::UnexpectedError,
            x if x == sys::EResult::k_EResultDisabled as i64 => SteamError::Disabled,
            x if x == sys::EResult::k_EResultInvalidCEGSubmission as i64 => {
                SteamError::InvalidCEGSubmission
            }
            x if x == sys::EResult::k_EResultRestrictedDevice as i64 => {
                SteamError::RestrictedDevice
            }
            x if x == sys::EResult::k_EResultRegionLocked as i64 => SteamError::RegionLocked,
            x if x == sys::EResult::k_EResultRateLimitExceeded as i64 => {
                SteamError::RateLimitExceeded
            }
            x if x == sys::EResult::k_EResultAccountLoginDeniedNeedTwoFactor as i64 => {
                SteamError::AccountLoginDeniedNeedTwoFactor
            }
            x if x == sys::EResult::k_EResultItemDeleted as i64 => SteamError::ItemDeleted,
            x if x == sys::EResult::k_EResultAccountLoginDeniedThrottle as i64 => {
                SteamError::AccountLoginDeniedThrottle
            }
            x if x == sys::EResult::k_EResultTwoFactorCodeMismatch as i64 => {
                SteamError::TwoFactorCodeMismatch
            }
            x if x == sys::EResult::k_EResultTwoFactorActivationCodeMismatch as i64 => {
                SteamError::TwoFactorActivationCodeMismatch
            }
            x if x == sys::EResult::k_EResultAccountAssociatedToMultiplePartners as i64 => {
                SteamError::AccountAssociatedToMultiplePartners
            }
            x if x == sys::EResult::k_EResultNotModified as i64 => SteamError::NotModified,
            x if x == sys::EResult::k_EResultNoMobileDevice as i64 => SteamError::NoMobileDevice,
            x if x == sys::EResult::k_EResultTimeNotSynced as i64 => SteamError::TimeNotSynced,
            x if x == sys::EResult::k_EResultSmsCodeFailed as i64 => SteamError::SmsCodeFailed,
            x if x == sys::EResult::k_EResultAccountLimitExceeded as i64 => {
                SteamError::AccountLimitExceeded
            }
            x if x == sys::EResult::k_EResultAccountActivityLimitExceeded as i64 => {
                SteamError::AccountActivityLimitExceeded
            }
            x if x == sys::EResult::k_EResultPhoneActivityLimitExceeded as i64 => {
                SteamError::PhoneActivityLimitExceeded
            }
            x if x == sys::EResult::k_EResultRefundToWallet as i64 => SteamError::RefundToWallet,
            x if x == sys::EResult::k_EResultEmailSendFailure as i64 => {
                SteamError::EmailSendFailure
            }
            x if x == sys::EResult::k_EResultNotSettled as i64 => SteamError::NotSettled,
            x if x == sys::EResult::k_EResultNeedCaptcha as i64 => SteamError::NeedCaptcha,
            x if x == sys::EResult::k_EResultGSLTDenied as i64 => SteamError::GSLTDenied,
            x if x == sys::EResult::k_EResultGSOwnerDenied as i64 => SteamError::GSOwnerDenied,
            x if x == sys::EResult::k_EResultInvalidItemType as i64 => SteamError::InvalidItemType,
            x if x == sys::EResult::k_EResultIPBanned as i64 => SteamError::IPBanned,
            x if x == sys::EResult::k_EResultGSLTExpired as i64 => SteamError::GSLTExpired,
            x if x == sys::EResult::k_EResultInsufficientFunds as i64 => {
                SteamError::InsufficientFunds
            }
            x if x == sys::EResult::k_EResultTooManyPending as i64 => SteamError::TooManyPending,
            x if x == sys::EResult::k_EResultNoSiteLicensesFound as i64 => {
                SteamError::NoSiteLicensesFound
            }
            x if x == sys::EResult::k_EResultWGNetworkSendExceeded as i64 => {
                SteamError::WGNetworkSendExceeded
            }
            _ => return Err(InvalidErrorCode),
        };
        Ok(error)
    }
}

#[derive(Debug, Error)]
#[error("error code could not be converted to rust enum")]
pub struct InvalidErrorCode;
