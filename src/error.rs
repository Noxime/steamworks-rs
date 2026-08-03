#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::sys;
use std::{convert::TryFrom, ffi::CStr};

/// Link to a page/id on the official Steamworks API Reference. For doc comments
macro_rules! docs_link {
    ($page:ident/$id:ident) => {
        concat!(
            "_Steamworks API Reference:_ [`",
            stringify!($id),
            "`](https://partner.steamgames.com/doc/api/",
            stringify!($page),
            "#",
            stringify!($id),
            ")"
        )
    };
}

/// Map a SteamError name to its raw binding name. Most are direct mappings, but some aren't.
macro_rules! map_binding_name {
    (Generic) => {
        k_EResultFail
    };
    (InvalidProtocolVersion) => {
        k_EResultInvalidProtocolVer
    };
    (InvalidParameter) => {
        k_EResultInvalidParam
    };
    ($variant:ident) => {
        pastey::paste! { [< k_EResult $variant >] }
    };
}

/// Implementation of SteamError. Macro is a little bit harder to read but avoids us accidentally missing a variant match
macro_rules! steam_error {
    (
        $(
            $(
                #[doc = $doc:expr] // doc comment gets expanded before macro exp, so we match it like this
            )*
            #[error($error:expr)]
            $variant:ident
        ),*
    ) => {
        pastey::paste! {
            /// Covers errors that can be returned by the steamworks API
            ///
            /// Note that all [`EResult`](sys::EResult) values are [`SteamError`]s, as for example
            /// [`k_EResultOK`](sys::EResult::k_EResultOK) is not an error at all.
            ///
            /// Documentation is based on official documentation which doesn't
            /// always explain when an error could be returned or its meaning.
            #[derive(Copy, Clone, Debug, Error, PartialEq, Eq)]
            #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
            pub enum SteamError {
                $(
                    // Link to official doc, then doc comments
                    #[doc = concat!("**\"", $error, "\"** (", docs_link!(steam_api/[<k_EResult $variant>]), ")\n\n")]
                    $(
                        #[doc = $doc]
                    )*
                    #[error($error)]
                    $variant
                ),*
            }
        }


            impl TryFrom<i64> for SteamError {
                type Error = InvalidSteamError;
                fn try_from(raw: i64) -> Result<Self, Self::Error> {
                    use sys::EResult::*;
                    let ok = match raw {
                        x if x == sys::EResult::k_EResultOK as i64 => return Err(InvalidSteamError::Ok),

                        $(
                            x if x == map_binding_name!($variant) as i64 => SteamError::$variant,
                        )*

                        // Unhandled error, either something undocumented or a new error.
                        _ => return Err(InvalidSteamError::Unknown(raw)),
                    };

                    Ok(ok)
                }
            }
    };
}

steam_error! {
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
    #[error("the user is not mutually friends")]
    AccountNotFriends,
    #[error("the user is limited")]
    LimitedUserAccount,
    #[error("item can't be removed")]
    CantRemoveItem,
    #[error("account has been deleted")]
    AccountDeleted,
    #[error("A license for this already exists, but cancelled")]
    ExistingUserCancelledLicense,
    #[error("access is denied because of a community cooldown (probably from support profile data resets)")]
    CommunityCooldown,
    #[error("No launcher was specified, but a launcher was needed to choose correct realm for operation.")]
    NoLauncherSpecified,
    #[error("User must agree to china SSA or global SSA before login")]
    MustAgreeToSSA,
    #[error(
        "The specified launcher type is no longer supported; the user should be directed elsewhere"
    )]
    LauncherMigrated,
    #[error("The user's realm does not match the realm of the requested resource")]
    SteamRealmMismatch,
    #[error("signature check did not match")]
    InvalidSignature,
    #[error("Failed to parse input")]
    ParseFailure,
    #[error("account does not have a verified phone number")]
    NoVerifiedPhone,
    #[error("user device doesn't have enough battery charge currently to complete the action")]
    InsufficientBattery,
    #[error("The operation requires a charger to be plugged in, which wasn't present")]
    ChargerRequired,
    #[error("Cached credential was invalid - user must reauthenticate")]
    CachedCredentialInvalid,
    #[error("The data being accessed is not supported by this API")]
    NotSupported,
    #[error("Reached the maximum size of the family")]
    FamilySizeLimitExceeded,
    #[error("The local data for the offline mode cache is insufficient to login")]
    OfflineAppCacheInvalid,
    #[error("retry the operation later")]
    TryLater
}

// There is no Try<EResult>
impl TryFrom<sys::EResult> for SteamError {
    type Error = InvalidSteamError;
    fn try_from(raw: sys::EResult) -> Result<Self, Self::Error> {
        Self::try_from(raw as i64)
    }
}

/// Unrecognized error code, or code that isn't an error ([`sys::EResult::k_EResultOK`])
#[derive(Debug, Error)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum InvalidSteamError {
    /// Error wasn't an error after all, but Ok
    #[error("error is 'ok'")]
    Ok,
    /// Error code was not recognized as a valid error code by steamworks-rs
    #[error("error code {0} could not be converted to enum")]
    Unknown(i64),
}

/// Error returned in API initialization.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SteamAPIInitError {
    #[error("Some other failure")]
    Generic(String),

    /// Returned when we can't connect to steam, probably because it's not running.
    #[error("We cannot connect to Steam, steam probably isn't running")]
    NoSteamClient(String),

    /// Returned when the version of the steam client is too old to work with this version of the API.
    #[error("Steam client appears to be out of date")]
    VersionMismatch(String),
}

impl SteamAPIInitError {
    pub fn from_result_and_message(
        result: sys::ESteamAPIInitResult,
        message: sys::SteamErrMsg,
    ) -> Self {
        let err_string = unsafe {
            let cstr = CStr::from_ptr(message.as_ptr());
            cstr.to_string_lossy().to_owned().into_owned()
        };

        match result {
            sys::ESteamAPIInitResult::k_ESteamAPIInitResult_FailedGeneric => {
                SteamAPIInitError::Generic(err_string)
            }
            sys::ESteamAPIInitResult::k_ESteamAPIInitResult_NoSteamClient => {
                SteamAPIInitError::NoSteamClient(err_string)
            }
            sys::ESteamAPIInitResult::k_ESteamAPIInitResult_VersionMismatch => {
                SteamAPIInitError::VersionMismatch(err_string)
            }
            _ => unreachable!(),
        }
    }
}
