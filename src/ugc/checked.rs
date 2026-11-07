//! Checked Workshop operations share cancellation and native query ownership.
use super::*;
use std::mem::{offset_of, size_of, MaybeUninit};

mod wire;

/// A recoverable failure in a checked Workshop operation.
///
/// Native `false` does not identify a specific Steam error. SDK status failures
/// retain the numeric code, including codes unknown to this version of the crate.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum UgcError {
    #[error("invalid Workshop parameter: {0}")]
    InvalidInput(&'static str),
    #[error("Workshop path could not be prepared: {0}")]
    PathIo(#[source] std::io::Error),
    #[error("Steam rejected {0}")]
    NativeRejected(&'static str),
    #[error("Steam did not start the asynchronous call")]
    InvalidCall,
    #[error("Steam could not deliver the asynchronous result")]
    Transport,
    #[error("invalid Workshop result: {0}")]
    InvalidPayload(&'static str),
    #[error("Steam returned result code {code}")]
    Steam {
        code: u32,
        known_error: Option<SteamError>,
        published_file_id: Option<PublishedFileId>,
    },
    #[error("Workshop result index {index} is outside {returned} entries")]
    OutOfBounds { index: u32, returned: u32 },
    #[error("Workshop result {index} could not be read")]
    ItemReadFailed { index: u32 },
    #[error("Workshop count {count} exceeds capacity {capacity}")]
    CapacityExceeded { count: usize, capacity: usize },
    #[error("cannot allocate storage for {requested} Workshop entries or bytes")]
    AllocationFailed { requested: usize },
    #[error("unknown {field} value {value}")]
    UnknownValue { field: &'static str, value: u32 },
}

impl UgcError {
    fn into_legacy(self) -> SteamError {
        match self {
            Self::Steam {
                known_error: Some(e),
                ..
            } => e,
            Self::Transport => SteamError::IOFailure,
            Self::InvalidInput(_) | Self::InvalidCall => SteamError::InvalidParameter,
            Self::PathIo(e) if e.kind() == std::io::ErrorKind::NotFound => SteamError::FileNotFound,
            Self::PathIo(_) => SteamError::IOFailure,
            _ => SteamError::Generic,
        }
    }
}

pub(super) fn legacy_call<T: 'static, F>(
    cb: F,
    start: impl FnOnce(
        Box<dyn FnOnce(Result<T, UgcError>) + Send>,
    ) -> Result<CallResultHandle, UgcError>,
) where
    F: FnOnce(Result<T, SteamError>) + Send + 'static,
{
    let callback = Arc::new(Mutex::new(Some(cb)));
    let completion = callback.clone();
    match start(Box::new(move |r| {
        let cb = completion.lock().unwrap().take();
        if let Some(cb) = cb {
            cb(r.map_err(UgcError::into_legacy));
        }
    })) {
        Ok(handle) => handle.detach(),
        Err(error) => {
            let cb = callback.lock().unwrap().take();
            if let Some(cb) = cb {
                cb(Err(error.into_legacy()));
            }
        }
    }
}

fn status(code: u32, item: Option<PublishedFileId>) -> Result<(), UgcError> {
    if code == sys::EResult::k_EResultOK as u32 {
        Ok(())
    } else {
        Err(UgcError::Steam {
            code,
            known_error: SteamError::try_from(i64::from(code)).ok(),
            published_file_id: item,
        })
    }
}

fn cstring(value: &str, parameter: &'static str) -> Result<CString, UgcError> {
    CString::new(value).map_err(|_| UgcError::InvalidInput(parameter))
}

fn apply_string(
    value: &str,
    parameter: &'static str,
    native: impl FnOnce(*const c_char) -> bool,
) -> Result<(), UgcError> {
    let value = cstring(value, parameter)?;
    accepted(native(value.as_ptr()), parameter)
}

fn accepted(ok: bool, operation: &'static str) -> Result<(), UgcError> {
    if ok {
        Ok(())
    } else {
        Err(UgcError::NativeRejected(operation))
    }
}

fn register_checked<F>(
    inner: &Inner,
    call: sys::SteamAPICall_t,
    size: usize,
    id: i32,
    callback: F,
    cleanup: Option<Box<dyn FnOnce() + Send>>,
) -> Result<CallResultHandle, UgcError>
where
    F: FnOnce(Result<*const u8, UgcError>) + Send + 'static,
{
    if call == sys::k_uAPICallInvalid {
        if let Some(cleanup) = cleanup {
            cleanup();
        }
        return Err(UgcError::InvalidCall);
    }
    Ok(register_raw_call_result(
        inner,
        call,
        size,
        Some(id),
        move |ptr, failure| {
            callback(match failure {
                None => Ok(ptr.cast_const().cast()),
                Some(CallResultFailure::Transport) => Err(UgcError::Transport),
                Some(CallResultFailure::Retrieval) => {
                    Err(UgcError::NativeRejected("GetAPICallResult"))
                }
                Some(CallResultFailure::InvalidPayload) => {
                    Err(UgcError::InvalidPayload("callback type or size"))
                }
            });
        },
        cleanup,
    ))
}

/// The item and operation reported by Steam after adding/removing a favorite.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FavoriteChangeResult {
    pub published_file_id: PublishedFileId,
    pub was_add_request: bool,
}

/// Steam's vote state. Failed reads are errors, not an unvoted state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UserItemVoteResult {
    pub published_file_id: PublishedFileId,
    pub voted_up: bool,
    pub voted_down: bool,
    pub vote_skipped: bool,
}

/// The item and vote direction returned by Steam.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SetUserItemVoteResult {
    pub published_file_id: PublishedFileId,
    pub vote_up: bool,
}

/// A local enumeration, not an atomic or authoritative server snapshot.
#[derive(Clone, Debug)]
pub struct SubscribedItemsRead {
    pub items: Vec<PublishedFileId>,
    pub count_before: u32,
    pub copied: u32,
    pub count_after: u32,
}

/// An owned ID-only page. Individual read failures fail the whole page.
#[derive(Clone, Debug)]
pub struct UgcIdPage {
    pub items: Vec<PublishedFileId>,
    pub returned_count: u32,
    pub total_count: u32,
    pub was_cached: bool,
}

impl UGC {
    /// Starts an item creation. Keep the handle to receive completion.
    /// Cancelling delivery does not undo creation; retain application identity
    /// and recovery state until the native result is known.
    pub fn create_item_with_handle<F>(
        &self,
        app_id: AppId,
        file_type: FileType,
        cb: F,
    ) -> Result<CallResultHandle, UgcError>
    where
        F: FnOnce(Result<(PublishedFileId, bool), UgcError>) + Send + 'static,
    {
        let call =
            unsafe { sys::SteamAPI_ISteamUGC_CreateItem(self.ugc, app_id.0, file_type.into()) };
        register_checked(
            &self.inner,
            call,
            size_of::<sys::CreateItemResult_t>(),
            sys::CreateItemResult_t_k_iCallback as i32,
            move |r| cb(r.and_then(|p| unsafe { wire::created(p) })),
            None,
        )
    }

    /// Subscribes and retains the item ID actually returned by Steam.
    pub fn subscribe_item_with_result<F>(
        &self,
        item: PublishedFileId,
        cb: F,
    ) -> Result<CallResultHandle, UgcError>
    where
        F: FnOnce(Result<PublishedFileId, UgcError>) + Send + 'static,
    {
        let call = unsafe { sys::SteamAPI_ISteamUGC_SubscribeItem(self.ugc, item.0) };
        register_checked(
            &self.inner,
            call,
            size_of::<sys::RemoteStorageSubscribePublishedFileResult_t>(),
            sys::RemoteStorageSubscribePublishedFileResult_t_k_iCallback as i32,
            move |r| cb(r.and_then(|p| unsafe { wire::subscribed(p) })),
            None,
        )
    }

    /// Unsubscribes and retains the item ID actually returned by Steam.
    pub fn unsubscribe_item_with_result<F>(
        &self,
        item: PublishedFileId,
        cb: F,
    ) -> Result<CallResultHandle, UgcError>
    where
        F: FnOnce(Result<PublishedFileId, UgcError>) + Send + 'static,
    {
        let call = unsafe { sys::SteamAPI_ISteamUGC_UnsubscribeItem(self.ugc, item.0) };
        register_checked(
            &self.inner,
            call,
            size_of::<sys::RemoteStorageUnsubscribePublishedFileResult_t>(),
            sys::RemoteStorageUnsubscribePublishedFileResult_t_k_iCallback as i32,
            move |r| cb(r.and_then(|p| unsafe { wire::unsubscribed(p) })),
            None,
        )
    }

    /// Adds a favorite. Completion reports Steam's item ID and operation direction.
    pub fn add_item_to_favorites<F>(
        &self,
        app_id: AppId,
        item: PublishedFileId,
        cb: F,
    ) -> Result<CallResultHandle, UgcError>
    where
        F: FnOnce(Result<FavoriteChangeResult, UgcError>) + Send + 'static,
    {
        let call =
            unsafe { sys::SteamAPI_ISteamUGC_AddItemToFavorites(self.ugc, app_id.0, item.0) };
        self.favorite_result(call, cb)
    }

    /// Removes a favorite. Cancelling delivery does not undo the operation.
    pub fn remove_item_from_favorites<F>(
        &self,
        app_id: AppId,
        item: PublishedFileId,
        cb: F,
    ) -> Result<CallResultHandle, UgcError>
    where
        F: FnOnce(Result<FavoriteChangeResult, UgcError>) + Send + 'static,
    {
        let call =
            unsafe { sys::SteamAPI_ISteamUGC_RemoveItemFromFavorites(self.ugc, app_id.0, item.0) };
        self.favorite_result(call, cb)
    }

    fn favorite_result<F>(
        &self,
        call: sys::SteamAPICall_t,
        cb: F,
    ) -> Result<CallResultHandle, UgcError>
    where
        F: FnOnce(Result<FavoriteChangeResult, UgcError>) + Send + 'static,
    {
        register_checked(
            &self.inner,
            call,
            size_of::<sys::UserFavoriteItemsListChanged_t>(),
            sys::UserFavoriteItemsListChanged_t_k_iCallback as i32,
            move |r| cb(r.and_then(|p| unsafe { wire::favorite(p) })),
            None,
        )
    }

    /// Reads the current user's vote. Keep the handle alive for completion.
    pub fn get_user_item_vote<F>(
        &self,
        item: PublishedFileId,
        cb: F,
    ) -> Result<CallResultHandle, UgcError>
    where
        F: FnOnce(Result<UserItemVoteResult, UgcError>) + Send + 'static,
    {
        let call = unsafe { sys::SteamAPI_ISteamUGC_GetUserItemVote(self.ugc, item.0) };
        register_checked(
            &self.inner,
            call,
            size_of::<sys::GetUserItemVoteResult_t>(),
            sys::GetUserItemVoteResult_t_k_iCallback as i32,
            move |r| cb(r.and_then(|p| unsafe { wire::vote(p) })),
            None,
        )
    }

    /// Sets an up/down vote. Steam does not expose an unvote through this call.
    pub fn set_user_item_vote<F>(
        &self,
        item: PublishedFileId,
        vote_up: bool,
        cb: F,
    ) -> Result<CallResultHandle, UgcError>
    where
        F: FnOnce(Result<SetUserItemVoteResult, UgcError>) + Send + 'static,
    {
        let call = unsafe { sys::SteamAPI_ISteamUGC_SetUserItemVote(self.ugc, item.0, vote_up) };
        register_checked(
            &self.inner,
            call,
            size_of::<sys::SetUserItemVoteResult_t>(),
            sys::SetUserItemVoteResult_t_k_iCallback as i32,
            move |r| cb(r.and_then(|p| unsafe { wire::set_vote(p) })),
            None,
        )
    }

    /// Bounded, single-pass local enumeration including count-change evidence.
    /// Equal counts do not prove atomicity or server authority. No implicit retry.
    pub fn subscribed_items_checked(
        &self,
        include_disabled: bool,
        max_items: usize,
    ) -> Result<SubscribedItemsRead, UgcError> {
        unsafe {
            let before = sys::SteamAPI_ISteamUGC_GetNumSubscribedItems(self.ugc, include_disabled);
            read_subscriptions(
                before,
                max_items,
                |ids| {
                    sys::SteamAPI_ISteamUGC_GetSubscribedItems(
                        self.ugc,
                        ids.as_mut_ptr(),
                        ids.len() as u32,
                        include_disabled,
                    )
                },
                || sys::SteamAPI_ISteamUGC_GetNumSubscribedItems(self.ugc, include_disabled),
            )
        }
    }

    /// Reads the install path with a caller-selected byte capacity.
    /// Native rejection does not distinguish unavailable content from a small buffer.
    pub fn item_install_info_with_capacity(
        &self,
        item: PublishedFileId,
        capacity: usize,
    ) -> Result<InstallInfo, UgcError> {
        let capacity32 =
            u32::try_from(capacity).map_err(|_| UgcError::InvalidInput("path capacity"))?;
        if capacity == 0 {
            return Err(UgcError::InvalidInput("path capacity"));
        }
        let mut folder = Vec::new();
        folder
            .try_reserve_exact(capacity)
            .map_err(|_| UgcError::AllocationFailed {
                requested: capacity,
            })?;
        folder.resize(capacity, 0 as c_char);
        let (mut size, mut timestamp) = (0, 0);
        accepted(
            unsafe {
                sys::SteamAPI_ISteamUGC_GetItemInstallInfo(
                    self.ugc,
                    item.0,
                    &mut size,
                    folder.as_mut_ptr(),
                    capacity32,
                    &mut timestamp,
                )
            },
            "GetItemInstallInfo",
        )?;
        Ok(InstallInfo {
            folder: wire::text(&folder, "install path")?,
            size_on_disk: size,
            timestamp,
        })
    }
}

fn read_subscriptions(
    before: u32,
    maximum: usize,
    copy: impl FnOnce(&mut [u64]) -> u32,
    after: impl FnOnce() -> u32,
) -> Result<SubscribedItemsRead, UgcError> {
    if before as usize > maximum {
        return Err(UgcError::CapacityExceeded {
            count: before as usize,
            capacity: maximum,
        });
    }
    let mut data = Vec::new();
    data.try_reserve_exact(before as usize)
        .map_err(|_| UgcError::AllocationFailed {
            requested: before as usize,
        })?;
    data.resize(before as usize, 0);
    let copied = copy(&mut data);
    if copied as usize > data.len() {
        return Err(UgcError::InvalidPayload("subscription count"));
    }
    data.truncate(copied as usize);
    Ok(SubscribedItemsRead {
        items: data.into_iter().map(PublishedFileId).collect(),
        count_before: before,
        copied,
        count_after: after(),
    })
}

macro_rules! query_value_setters {
    ($($name:ident($ty:ty) => $native:ident;)*) => { impl QueryHandle { $(
        #[doc = concat!("Checked variant of `", stringify!($native), "`. Returns native rejection instead of ignoring it.")]
        pub fn $name(self, value: $ty) -> Result<Self, UgcError> {
            let handle = self.handle.ok_or(UgcError::InvalidInput("consumed query"))?;
            accepted(unsafe { sys::$native(self.ugc, handle, value) }, stringify!($native))?;
            Ok(self)
        }
    )* } };
}

query_value_setters! {
    try_set_return_only_ids(bool) => SteamAPI_ISteamUGC_SetReturnOnlyIDs;
    try_set_return_metadata(bool) => SteamAPI_ISteamUGC_SetReturnMetadata;
    try_set_return_long_description(bool) => SteamAPI_ISteamUGC_SetReturnLongDescription;
    try_set_allow_cached_response(u32) => SteamAPI_ISteamUGC_SetAllowCachedResponse;
}

macro_rules! query_string_setters {
    ($($name:ident => $native:ident;)*) => { impl QueryHandle { $(
        #[doc = concat!("Checked variant of `", stringify!($native), "`. Reports embedded NUL and native rejection.")]
        pub fn $name(self, value: &str) -> Result<Self, UgcError> {
            let handle = self.handle.ok_or(UgcError::InvalidInput("consumed query"))?;
            apply_string(value, stringify!($native), |value| unsafe { sys::$native(self.ugc, handle, value) })?;
            Ok(self)
        }
    )* } };
}

query_string_setters! {
    try_add_required_tag => SteamAPI_ISteamUGC_AddRequiredTag;
    try_add_excluded_tag => SteamAPI_ISteamUGC_AddExcludedTag;
    try_set_search_text => SteamAPI_ISteamUGC_SetSearchText;
}

struct TagStrings(Vec<CString>);
impl TagStrings {
    fn new<S: AsRef<str>>(tags: &[S], allow_empty: bool) -> Result<Self, UgcError> {
        if !allow_empty && tags.is_empty() {
            return Err(UgcError::InvalidInput("empty tag group"));
        }
        i32::try_from(tags.len()).map_err(|_| UgcError::InvalidInput("tag count"))?;
        Ok(Self(
            tags.iter()
                .map(|s| cstring(s.as_ref(), "tag"))
                .collect::<Result<_, _>>()?,
        ))
    }

    fn with_raw<T>(&self, f: impl FnOnce(&sys::SteamParamStringArray_t) -> T) -> T {
        let mut ptrs: Vec<_> = self.0.iter().map(|s| s.as_ptr()).collect();
        f(&sys::SteamParamStringArray_t {
            m_ppStrings: ptrs.as_mut_ptr(),
            m_nNumStrings: ptrs.len() as i32,
        })
    }
}

impl QueryHandle {
    pub(super) fn fetch_legacy<F>(self, cb: F)
    where
        F: for<'a> FnOnce(Result<QueryResults<'a>, SteamError>) + Send + 'static,
    {
        let callback = Arc::new(Mutex::new(Some(cb)));
        let completion = callback.clone();
        match self.fetch_with_handle(move |r| {
            let cb = completion.lock().unwrap().take();
            if let Some(cb) = cb {
                cb(r.map_err(UgcError::into_legacy));
            }
        }) {
            Ok(handle) => handle.detach(),
            Err(error) => {
                let cb = callback.lock().unwrap().take();
                if let Some(cb) = cb {
                    cb(Err(error.into_legacy()));
                }
            }
        }
    }
    /// Requires at least one tag in this group, and at least one from every group.
    /// Must be configured before sending. Empty groups and NUL strings are errors.
    pub fn try_add_required_tag_group<S: AsRef<str>>(self, tags: &[S]) -> Result<Self, UgcError> {
        let strings = TagStrings::new(tags, false)?;
        let handle = self
            .handle
            .ok_or(UgcError::InvalidInput("consumed query"))?;
        accepted(
            strings.with_raw(|raw| unsafe {
                sys::SteamAPI_ISteamUGC_AddRequiredTagGroup(self.ugc, handle, raw)
            }),
            "AddRequiredTagGroup",
        )?;
        Ok(self)
    }
}

macro_rules! update_string_setters {
    ($($name:ident => $native:ident;)*) => { impl UpdateHandle { $(
        #[doc = concat!("Checked variant of `", stringify!($native), "`. Reports embedded NUL and native rejection.")]
        pub fn $name(self, value: &str) -> Result<Self, UgcError> {
            apply_string(value, stringify!($native), |value| unsafe { sys::$native(self.ugc, self.handle, value) })?;
            Ok(self)
        }
    )* } };
}

update_string_setters! {
    try_title => SteamAPI_ISteamUGC_SetItemTitle;
    try_description => SteamAPI_ISteamUGC_SetItemDescription;
    try_language => SteamAPI_ISteamUGC_SetItemUpdateLanguage;
    try_metadata => SteamAPI_ISteamUGC_SetItemMetadata;
}

fn path_string(path: &Path) -> Result<CString, UgcError> {
    let path = path.canonicalize().map_err(UgcError::PathIo)?;
    cstring(
        path.to_str()
            .ok_or(UgcError::InvalidInput("path encoding"))?,
        "path",
    )
}

impl UpdateHandle {
    pub(super) fn submit_legacy<F>(self, change_note: Option<&str>, cb: F) -> UpdateWatchHandle
    where
        F: FnOnce(Result<(PublishedFileId, bool), SteamError>) + Send + 'static,
    {
        let fallback = UpdateWatchHandle {
            ugc: self.ugc,
            _inner: self.inner.clone(),
            handle: self.handle,
        };
        let callback = Arc::new(Mutex::new(Some(cb)));
        let completion = callback.clone();
        // Preserve the legacy API's documented/baseline treatment of NUL notes.
        // The checked submit path rejects these instead of silently omitting them.
        let note = change_note.filter(|s| !s.as_bytes().contains(&0));
        match self.try_submit_with_handle(note, move |r| {
            let cb = completion.lock().unwrap().take();
            if let Some(cb) = cb {
                cb(r.map_err(UgcError::into_legacy));
            }
        }) {
            Ok((watch, handle)) => {
                handle.detach();
                watch
            }
            Err(error) => {
                let cb = callback.lock().unwrap().take();
                if let Some(cb) = cb {
                    cb(Err(error.into_legacy()));
                }
                fallback
            }
        }
    }
    /// Sets visibility, reporting native rejection.
    pub fn try_visibility(self, visibility: PublishedFileVisibility) -> Result<Self, UgcError> {
        accepted(
            unsafe {
                sys::SteamAPI_ISteamUGC_SetItemVisibility(self.ugc, self.handle, visibility.into())
            },
            "SetItemVisibility",
        )?;
        Ok(self)
    }

    /// Sets tags. An empty list retains Steam's clearing semantics.
    pub fn try_tags<S: AsRef<str>>(
        self,
        tags: &[S],
        allow_admin_tags: bool,
    ) -> Result<Self, UgcError> {
        let strings = TagStrings::new(tags, true)?;
        accepted(
            strings.with_raw(|raw| unsafe {
                sys::SteamAPI_ISteamUGC_SetItemTags(self.ugc, self.handle, raw, allow_admin_tags)
            }),
            "SetItemTags",
        )?;
        Ok(self)
    }

    /// Prepares a content directory without panic or lossy path substitution.
    pub fn try_content_path(self, path: &Path) -> Result<Self, UgcError> {
        let path = path_string(path)?;
        accepted(
            unsafe { sys::SteamAPI_ISteamUGC_SetItemContent(self.ugc, self.handle, path.as_ptr()) },
            "SetItemContent",
        )?;
        Ok(self)
    }

    /// Prepares a preview path without panic or lossy path substitution.
    pub fn try_preview_path(self, path: &Path) -> Result<Self, UgcError> {
        let path = path_string(path)?;
        accepted(
            unsafe { sys::SteamAPI_ISteamUGC_SetItemPreview(self.ugc, self.handle, path.as_ptr()) },
            "SetItemPreview",
        )?;
        Ok(self)
    }

    /// Submits an update and returns separate progress and result-delivery handles.
    /// Keep content/preview files alive until completion or SDK shutdown, even if
    /// result delivery is cancelled. Invalid change notes do not silently disappear.
    pub fn try_submit_with_handle<F>(
        self,
        change_note: Option<&str>,
        cb: F,
    ) -> Result<(UpdateWatchHandle, CallResultHandle), UgcError>
    where
        F: FnOnce(Result<(PublishedFileId, bool), UgcError>) + Send + 'static,
    {
        let note = change_note.map(|s| cstring(s, "change note")).transpose()?;
        let call = unsafe {
            sys::SteamAPI_ISteamUGC_SubmitItemUpdate(
                self.ugc,
                self.handle,
                note.as_ref().map_or(std::ptr::null(), |s| s.as_ptr()),
            )
        };
        let result = register_checked(
            &self.inner,
            call,
            size_of::<sys::SubmitItemUpdateResult_t>(),
            sys::SubmitItemUpdateResult_t_k_iCallback as i32,
            move |r| cb(r.and_then(|p| unsafe { wire::submitted(p) })),
            None,
        )?;
        Ok((
            UpdateWatchHandle {
                ugc: self.ugc,
                _inner: self.inner,
                handle: self.handle,
            },
            result,
        ))
    }
}

impl UpdateWatchHandle {
    /// Reads progress with a raw-value error for states unknown to this crate.
    pub fn progress_checked(&self) -> Result<(UpdateStatus, u64, u64), UgcError> {
        let (mut processed, mut total) = (0, 0);
        let state =
            unsafe { wire::update_progress_raw(self.ugc, self.handle, &mut processed, &mut total) };
        Ok((wire::progress(state)?, processed, total))
    }
}

pub(super) unsafe fn decode_download(p: *const u8) -> DownloadItemResult {
    wire::download(p)
}

/// The lease contains no Arc<Inner>: pending work must not keep its registry alive.
pub(super) struct QueryLease {
    ugc: *mut sys::ISteamUGC,
    handle: sys::UGCQueryHandle_t,
    release: unsafe extern "C" fn(*mut sys::ISteamUGC, sys::UGCQueryHandle_t) -> bool,
}
// Like UGC's existing native access, release is not restricted to a pump thread.
// The registry drains these leases before the SDK manager is dropped.
unsafe impl Send for QueryLease {}
unsafe impl Sync for QueryLease {}
impl Drop for QueryLease {
    fn drop(&mut self) {
        unsafe {
            (self.release)(self.ugc, self.handle);
        }
    }
}

impl QueryHandle {
    /// Sends a query with cancellable delivery and cleanup independent of user code.
    /// Results are borrowed for the callback; cancelled queries remain leased until
    /// their native result is consumed or the SDK shuts down.
    pub fn fetch_with_handle<F>(mut self, cb: F) -> Result<CallResultHandle, UgcError>
    where
        F: for<'a> FnOnce(Result<QueryResults<'a>, UgcError>) + Send + 'static,
    {
        let handle = self
            .handle
            .take()
            .ok_or(UgcError::InvalidInput("consumed query"))?;
        let lease = Arc::new(QueryLease {
            ugc: self.ugc,
            handle,
            release: sys::SteamAPI_ISteamUGC_ReleaseQueryUGCRequest,
        });
        // Only the pending registry owns the native lease before dispatch.
        // A cancelled user closure can be dropped concurrently with shutdown.
        let delivery_lease = Arc::downgrade(&lease);
        let call = unsafe { sys::SteamAPI_ISteamUGC_SendQueryUGCRequest(self.ugc, handle) };
        register_checked(
            &self.inner,
            call,
            size_of::<sys::SteamUGCQueryCompleted_t>(),
            sys::SteamUGCQueryCompleted_t_k_iCallback as i32,
            move |r| {
                cb(r.and_then(|p| {
                    let lease = delivery_lease
                        .upgrade()
                        .ok_or(UgcError::InvalidPayload("query lease"))?;
                    let meta = unsafe { wire::query(p, lease.handle)? };
                    Ok(QueryResults {
                        ugc: lease.ugc,
                        handle: lease.handle,
                        num_results_returned: meta.0,
                        num_results_total: meta.1,
                        was_cached: meta.2,
                        _phantom: Default::default(),
                        _lease: lease,
                    })
                }))
            },
            Some(Box::new(move || drop(lease))),
        )
    }

    /// Returns an owned, strict ID-only page including counts and cache status.
    pub fn fetch_ids_checked<F>(self, cb: F) -> Result<CallResultHandle, UgcError>
    where
        F: FnOnce(Result<UgcIdPage, UgcError>) + Send + 'static,
    {
        self.try_set_return_only_ids(true)?
            .fetch_with_handle(move |r| {
                cb(r.and_then(|page| {
                    // SetReturnOnlyIDs is supported by paged queries, not detail
                    // requests. Bound allocation to that SDK page contract.
                    if page.returned_results() > RESULTS_PER_PAGE {
                        return Err(UgcError::InvalidPayload("ID page size"));
                    }
                    let items = (0..page.returned_results())
                        .map(|i| page.get_id_checked(i))
                        .collect::<Result<_, _>>()?;
                    Ok(UgcIdPage {
                        items,
                        returned_count: page.returned_results(),
                        total_count: page.total_results(),
                        was_cached: page.was_cached(),
                    })
                }))
            })
    }
}

impl QueryResults<'_> {
    fn check_index(&self, index: u32) -> Result<(), UgcError> {
        if index >= self.num_results_returned {
            Err(UgcError::OutOfBounds {
                index,
                returned: self.num_results_returned,
            })
        } else {
            Ok(())
        }
    }

    /// Reads one complete detail record, retaining failures instead of skipping it.
    /// Use `get_id_checked` with ID-only queries.
    pub fn get_checked(&self, index: u32) -> Result<QueryResult, UgcError> {
        self.check_index(index)?;
        let mut details = MaybeUninit::<sys::SteamUGCDetails_t>::zeroed();
        if !unsafe {
            sys::SteamAPI_ISteamUGC_GetQueryUGCResult(
                self.ugc,
                self.handle,
                index,
                details.as_mut_ptr(),
            )
        } {
            return Err(UgcError::ItemReadFailed { index });
        }
        unsafe { wire::details(details.as_ptr().cast()) }
    }

    /// Reads only the ID. Does not inspect unspecified ID-only detail fields.
    pub fn get_id_checked(&self, index: u32) -> Result<PublishedFileId, UgcError> {
        self.check_index(index)?;
        let mut details = MaybeUninit::<sys::SteamUGCDetails_t>::zeroed();
        if !unsafe {
            sys::SteamAPI_ISteamUGC_GetQueryUGCResult(
                self.ugc,
                self.handle,
                index,
                details.as_mut_ptr(),
            )
        } {
            return Err(UgcError::ItemReadFailed { index });
        }
        Ok(PublishedFileId(unsafe {
            std::ptr::addr_of!((*details.as_ptr()).m_nPublishedFileId).read_unaligned()
        }))
    }

    /// Visits every index, preserving its error rather than filtering it out.
    pub fn iter_checked(&self) -> impl Iterator<Item = Result<QueryResult, UgcError>> + '_ {
        (0..self.returned_results()).map(|i| self.get_checked(i))
    }
}

#[cfg(test)]
mod checked_ugc_operation_tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    #[test]
    fn bad_string_never_reaches_native_and_native_false_is_recoverable() {
        assert!(matches!(
            apply_string("a\0b", "title", |_| panic!("must not call native")),
            Err(UgcError::InvalidInput("title"))
        ));
        assert!(matches!(
            apply_string("valid", "title", |p| {
                assert_eq!(unsafe { CStr::from_ptr(p) }.to_bytes(), b"valid");
                false
            }),
            Err(UgcError::NativeRejected("title"))
        ));
    }

    #[test]
    fn tag_array_keeps_unicode_strings_and_pointer_array_alive() {
        let tags = TagStrings::new(&["Widget", "天气"], false).unwrap();
        tags.with_raw(|raw| {
            assert_eq!(raw.m_nNumStrings, 2);
            let ptrs = unsafe { std::slice::from_raw_parts(raw.m_ppStrings, 2) };
            assert_eq!(
                unsafe { CStr::from_ptr(ptrs[0]) }.to_str().unwrap(),
                "Widget"
            );
            assert_eq!(unsafe { CStr::from_ptr(ptrs[1]) }.to_str().unwrap(), "天气");
        });
        assert!(TagStrings::new(&["first", "bad\0tag"], false).is_err());
        assert!(TagStrings::new::<&str>(&[], false).is_err());
        assert!(TagStrings::new::<&str>(&[], true).is_ok());
    }

    #[test]
    fn shrinking_subscriptions_are_truncated_without_zero_tail() {
        let r = read_subscriptions(
            4,
            4,
            |data| {
                data[0] = 2;
                data[1] = 3;
                2
            },
            || 2,
        )
        .unwrap();
        assert_eq!(r.items, vec![PublishedFileId(2), PublishedFileId(3)]);
        assert_eq!((r.count_before, r.copied, r.count_after), (4, 2, 2));
    }

    #[test]
    fn growing_subscriptions_report_evidence_without_claiming_completeness() {
        let r = read_subscriptions(
            1,
            8,
            |data| {
                data[0] = 9;
                1
            },
            || 2,
        )
        .unwrap();
        assert_eq!((r.count_before, r.copied, r.count_after), (1, 1, 2));
        assert_eq!(r.items, vec![PublishedFileId(9)]);
    }

    #[test]
    fn subscription_capacity_is_checked_before_native_copy() {
        assert!(matches!(
            read_subscriptions(10, 2, |_| panic!("over budget"), || 0),
            Err(UgcError::CapacityExceeded {
                count: 10,
                capacity: 2
            })
        ));
        assert!(read_subscriptions(1, 1, |_| 2, || 2).is_err());
        assert!(read_subscriptions(
            0,
            0,
            |data| {
                assert!(data.is_empty());
                0
            },
            || 0
        )
        .unwrap()
        .items
        .is_empty());
    }

    #[test]
    fn disabled_and_future_item_state_bits_survive_conversion() {
        let raw = sys::EItemState::k_EItemStateDisabledLocally as u32 | 0x8000;
        let flags = ItemState::from_bits_retain(raw);
        assert!(flags.contains(ItemState::DISABLED_LOCALLY));
        assert_eq!(flags.bits(), raw);
    }

    #[test]
    fn long_paths_are_read_to_their_actual_terminator() {
        let mut data = vec![65 as c_char; 6000];
        data[5999] = 0;
        assert_eq!(wire::text(&data, "path").unwrap().len(), 5999);
        data[5999] = 65;
        assert!(wire::text(&data, "path").is_err());
        assert!(path_string(Path::new("\0bad")).is_err());
    }

    #[test]
    fn invalid_native_call_cleans_resource_without_invoking_callback() {
        let inner = crate::callback::test_inner(Arc::new(AtomicBool::new(false)));
        let cleaned = Arc::new(AtomicBool::new(false));
        let copy = cleaned.clone();
        let r = register_checked(
            &inner,
            0,
            16,
            1,
            |_| panic!("start failed"),
            Some(Box::new(move || copy.store(true, Ordering::SeqCst))),
        );
        assert!(matches!(r, Err(UgcError::InvalidCall)));
        assert!(cleaned.load(Ordering::SeqCst));
        assert!(inner.callbacks.call_results.lock().unwrap().is_empty());
    }

    #[test]
    fn legacy_start_failure_delivers_one_compatible_error() {
        let called = Arc::new(AtomicUsize::new(0));
        let copy = called.clone();
        legacy_call(
            move |r: Result<(), SteamError>| {
                assert!(r.is_err());
                copy.fetch_add(1, Ordering::SeqCst);
            },
            |_cb| Err(UgcError::InvalidCall),
        );
        assert_eq!(called.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn cancelled_query_lease_releases_once_on_native_completion() {
        unsafe extern "C" fn release(p: *mut sys::ISteamUGC, handle: u64) -> bool {
            let state = &*p.cast::<AtomicUsize>();
            state.fetch_add(handle as usize, Ordering::SeqCst);
            true
        }
        let state = Box::new(AtomicUsize::new(0));
        let inner = crate::callback::test_inner(Arc::new(AtomicBool::new(false)));
        let owner = Arc::new(QueryLease {
            ugc: (&*state as *const AtomicUsize).cast_mut().cast(),
            handle: 7,
            release,
        });
        let user = Arc::downgrade(&owner);
        let handle = register_raw_call_result(
            &inner,
            1,
            0,
            Some(12),
            move |_, _| {
                drop(user);
                panic!("cancelled")
            },
            Some(Box::new(move || drop(owner))),
        );
        assert_eq!(handle.cancel(), CancelOutcome::Cancelled);
        assert_eq!(state.load(Ordering::SeqCst), 0);
        inner
            .callbacks
            .take_call_result(1)
            .unwrap()
            .complete(std::ptr::null_mut(), None);
        assert_eq!(state.load(Ordering::SeqCst), 7);
        drop(inner);
        assert_eq!(state.load(Ordering::SeqCst), 7);
    }
}
