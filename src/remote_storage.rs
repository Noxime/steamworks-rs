use super::*;
#[cfg(test)]
use serial_test::serial;
use steamworks_sys::{EResult, SteamAPICall_t, SteamAPI_ISteamRemoteStorage_UGCDownload};

/// Access to the steam remote storage interface
pub struct RemoteStorage<Manager> {
    pub(crate) rs: *mut sys::ISteamRemoteStorage,
    pub(crate) util: *mut sys::ISteamUtils,
    pub(crate) inner: Arc<Inner<Manager>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PublishedFileVisibility {
    Public,
    FriendsOnly,
    Private,
    Unlisted,
}

impl From<sys::ERemoteStoragePublishedFileVisibility> for PublishedFileVisibility {
    fn from(visibility: sys::ERemoteStoragePublishedFileVisibility) -> Self {
        match visibility {
            sys::ERemoteStoragePublishedFileVisibility::k_ERemoteStoragePublishedFileVisibilityPublic => PublishedFileVisibility::Public,
            sys::ERemoteStoragePublishedFileVisibility::k_ERemoteStoragePublishedFileVisibilityFriendsOnly => PublishedFileVisibility::FriendsOnly,
            sys::ERemoteStoragePublishedFileVisibility::k_ERemoteStoragePublishedFileVisibilityPrivate => PublishedFileVisibility::Private,
            sys::ERemoteStoragePublishedFileVisibility::k_ERemoteStoragePublishedFileVisibilityUnlisted => PublishedFileVisibility::Unlisted,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
/// The outcome of downloading UGC from a leaderboard, this type is exclusively used for `ugc_read`
pub struct LeaderboardUGCDownload {
    file_name: String,
    file_length: steamworks_sys::int32,
    ugc_handle: UGCHandle_t,
}

impl Into<sys::ERemoteStoragePublishedFileVisibility> for PublishedFileVisibility {
    fn into(self) -> sys::ERemoteStoragePublishedFileVisibility {
        match self {
            PublishedFileVisibility::Public => sys::ERemoteStoragePublishedFileVisibility::k_ERemoteStoragePublishedFileVisibilityPublic,
            PublishedFileVisibility::FriendsOnly => sys::ERemoteStoragePublishedFileVisibility::k_ERemoteStoragePublishedFileVisibilityFriendsOnly,
            PublishedFileVisibility::Private => sys::ERemoteStoragePublishedFileVisibility::k_ERemoteStoragePublishedFileVisibilityPrivate,
            PublishedFileVisibility::Unlisted => sys::ERemoteStoragePublishedFileVisibility::k_ERemoteStoragePublishedFileVisibilityUnlisted,
        }
    }
}

impl<Manager> Clone for RemoteStorage<Manager> {
    fn clone(&self) -> Self {
        RemoteStorage {
            inner: self.inner.clone(),
            rs: self.rs,
            util: self.util,
        }
    }
}

impl<Manager> RemoteStorage<Manager> {
    /// Toggles whether the steam cloud is enabled for the application
    pub fn set_cloud_enabled_for_app(&self, enabled: bool) {
        unsafe {
            sys::SteamAPI_ISteamRemoteStorage_SetCloudEnabledForApp(self.rs, enabled);
        }
    }

    pub fn raw(&self) -> *mut sys::ISteamRemoteStorage {
        self.rs
    }

    /// Returns whether the steam cloud is enabled for the application
    ///
    /// # Note
    ///
    /// This is independent from the account wide setting
    pub fn is_cloud_enabled_for_app(&self) -> bool {
        unsafe { sys::SteamAPI_ISteamRemoteStorage_IsCloudEnabledForApp(self.rs) }
    }

    /// Returns whether the steam cloud is enabled for the account
    ///
    /// # Note
    ///
    /// This is independent from the application setting
    pub fn is_cloud_enabled_for_account(&self) -> bool {
        unsafe { sys::SteamAPI_ISteamRemoteStorage_IsCloudEnabledForAccount(self.rs) }
    }

    /// Returns information about all files in the cloud storage
    pub fn files(&self) -> Vec<SteamFileInfo> {
        unsafe {
            let count = sys::SteamAPI_ISteamRemoteStorage_GetFileCount(self.rs);
            if count == -1 {
                return Vec::new();
            }
            let mut files = Vec::with_capacity(count as usize);
            for idx in 0..count {
                let mut size = 0;
                let name = CStr::from_ptr(sys::SteamAPI_ISteamRemoteStorage_GetFileNameAndSize(
                    self.rs, idx, &mut size,
                ));
                files.push(SteamFileInfo {
                    name: name.to_string_lossy().into_owned(),
                    size: size as u64,
                })
            }

            files
        }
    }

    pub fn download_ugc<F>(&self, leaderboard_entry: &LeaderboardEntry, cb: F)
    where F: FnOnce(Result<LeaderboardUGCDownload, EResult>) + 'static + Send,{
        // TODO(PRE FLIGHT CHECK): test this function as it has now been moved.
        unsafe {

            let ugc_handle = leaderboard_entry.ugc;

            let res = SteamAPI_ISteamRemoteStorage_UGCDownload(self.rs, ugc_handle, 1);

            register_call_result::<sys::RemoteStorageDownloadUGCResult_t, _, _>(&self.inner, res, 1100 + 4, move |a, _b| {
                let filename_cstr = CStr::from_ptr(a.m_pchFileName.as_ptr());
                let filename_owned_string = filename_cstr.to_str().unwrap().to_string();

                cb( // TODO: for some reason, `_b` can be true or false and still a good result type is received, the only reason I can seem to find is that the inner from client makes this work but the inner from remote storage makes it say it failed
                    match a.m_eResult {
                        EResult::k_EResultOK => {
                            Ok(LeaderboardUGCDownload {
                                file_length: a.m_nSizeInBytes,
                                file_name: filename_owned_string,
                                ugc_handle,
                            })
                        }
                        _ => {
                            Err(a.m_eResult)
                        }
                    }
                );
            });
        }
    }

    pub fn ugc_read(&self, leaderboard_ugc_download: LeaderboardUGCDownload) -> String {
        // TODO(PRE FLIGHT CHECK): test this function as it has now been moved.
        unsafe {

            let cstr = CString::new((0..leaderboard_ugc_download.file_name.len()).map(|val| 'a' as u8).collect::<Vec<_>>()).unwrap();

            let _ = SteamAPI_ISteamRemoteStorage_UGCRead(self.rs, leaderboard_ugc_download.ugc_handle, cstr.as_ptr() as *mut c_void, leaderboard_ugc_download.file_length, 0, EUGCReadAction::k_EUGCRead_ContinueReadingUntilFinished);

            cstr.into_string().unwrap()
        }
    }


    pub fn file_share<F>(&self,file_name: impl Into<String>, file_content: impl Into<String> ,cb: F) -> bool // TODO: Into<String> might not be the best type here, not sure yet
    where F: FnOnce(Result<LeaderboardUGC, EResult>) + 'static + Send, {
        unsafe {

            // TODO: maybe replace these with enum error types and change the function signature to output a result as well instead of a boolean?
            let Ok(str) = CString::new(file_name.into().as_bytes().to_vec()) else {
                return false;
            };

            let Ok(str_content) = CString::new(file_content.into()) else {
                return false;
            };

            let file_write_successful = sys::SteamAPI_ISteamRemoteStorage_FileWrite(self.raw(), str.as_ptr(), str_content.as_ptr() as *const c_void, str_content.as_bytes().len() as i32);

            if file_write_successful {
                let file_share_result = sys::SteamAPI_ISteamRemoteStorage_FileShare(self.raw(), str.as_ptr());

                register_call_result::<sys::RemoteStorageFileShareResult_t,_,_>(&self.inner, file_share_result, 1100+4, move |a, b| {

                    match a.m_eResult {
                        EResult::k_EResultOK => {
                            cb(Ok(LeaderboardUGC{handle: a.m_hFile}));
                        }
                        err => {
                            cb(Err(err))
                        }
                    }
                });
            }

            file_write_successful
        }

    }



    /// Returns a handle to a steam cloud file
    ///
    /// The file does not have to exist.
    pub fn file(&self, name: &str) -> SteamFile<Manager> {
        SteamFile {
            rs: self.rs,
            util: self.util,
            _inner: self.inner.clone(),
            name: CString::new(name).unwrap(),
        }
    }
}

/// A handle for a possible steam cloud file
pub struct SteamFile<Manager> {
    pub(crate) rs: *mut sys::ISteamRemoteStorage,
    pub(crate) util: *mut sys::ISteamUtils,
    pub(crate) _inner: Arc<Inner<Manager>>,
    name: CString,
}

impl<Manager> SteamFile<Manager> {
    /// Deletes the file locally and remotely.
    ///
    /// Returns whether a file was actually deleted
    pub fn delete(&self) -> bool {
        unsafe { sys::SteamAPI_ISteamRemoteStorage_FileDelete(self.rs, self.name.as_ptr()) }
    }
    /// Deletes the file remotely whilst keeping it locally.
    ///
    /// Returns whether a file was actually forgotten
    pub fn forget(&self) -> bool {
        unsafe { sys::SteamAPI_ISteamRemoteStorage_FileForget(self.rs, self.name.as_ptr()) }
    }

    /// Returns whether a file exists
    pub fn exists(&self) -> bool {
        unsafe { sys::SteamAPI_ISteamRemoteStorage_FileExists(self.rs, self.name.as_ptr()) }
    }

    /// Returns whether a file is persisted in the steam cloud
    pub fn is_persisted(&self) -> bool {
        unsafe { sys::SteamAPI_ISteamRemoteStorage_FilePersisted(self.rs, self.name.as_ptr()) }
    }

    // Returns the timestamp of the file
    pub fn timestamp(&self) -> i64 {
        unsafe { sys::SteamAPI_ISteamRemoteStorage_GetFileTimestamp(self.rs, self.name.as_ptr()) }
    }

    pub fn write(self) -> SteamFileWriter<Manager> {
        unsafe {
            let handle =
                sys::SteamAPI_ISteamRemoteStorage_FileWriteStreamOpen(self.rs, self.name.as_ptr());
            SteamFileWriter { file: self, handle }
        }
    }

    pub fn read(self) -> SteamFileReader<Manager> {
        unsafe {
            SteamFileReader {
                offset: 0,
                size: sys::SteamAPI_ISteamRemoteStorage_GetFileSize(self.rs, self.name.as_ptr())
                    as usize,
                file: self,
            }
        }
    }
}
/// A write handle for a steam cloud file
pub struct SteamFileWriter<Manager> {
    file: SteamFile<Manager>,
    handle: sys::UGCFileWriteStreamHandle_t,
}

impl<Manager> std::io::Write for SteamFileWriter<Manager> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        unsafe {
            if sys::SteamAPI_ISteamRemoteStorage_FileWriteStreamWriteChunk(
                self.file.rs,
                self.handle,
                buf.as_ptr() as *const _,
                buf.len() as _,
            ) {
                Ok(buf.len())
            } else {
                Err(std::io::ErrorKind::Other.into())
            }
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<Manager> Drop for SteamFileWriter<Manager> {
    fn drop(&mut self) {
        unsafe {
            sys::SteamAPI_ISteamRemoteStorage_FileWriteStreamClose(self.file.rs, self.handle);
        }
    }
}

/// A read handle for a steam cloud file
pub struct SteamFileReader<Manager> {
    file: SteamFile<Manager>,
    offset: usize,
    size: usize,
}

impl<Manager> std::io::Read for SteamFileReader<Manager> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        use std::cmp::min;
        if buf.is_empty() || self.size - self.offset == 0 {
            return Ok(0);
        }
        let len = min(buf.len(), self.size - self.offset);
        unsafe {
            let api_call = sys::SteamAPI_ISteamRemoteStorage_FileReadAsync(
                self.file.rs,
                self.file.name.as_ptr(),
                self.offset as _,
                len as _,
            );

            let mut failed = false;
            while !sys::SteamAPI_ISteamUtils_IsAPICallCompleted(
                self.file.util,
                api_call,
                &mut failed,
            ) {
                std::thread::yield_now();
            }
            if failed {
                return Err(std::io::ErrorKind::Other.into());
            }
            let mut callback: sys::RemoteStorageFileReadAsyncComplete_t = std::mem::zeroed();
            sys::SteamAPI_ISteamUtils_GetAPICallResult(
                self.file.util,
                api_call,
                (&mut callback) as *mut _ as *mut _,
                std::mem::size_of::<sys::RemoteStorageFileReadAsyncComplete_t>() as _,
                1332,
                &mut failed,
            );

            if callback.m_eResult != sys::EResult::k_EResultOK {
                return Err(std::io::ErrorKind::Other.into());
            }
            let size = callback.m_cubRead as usize;
            sys::SteamAPI_ISteamRemoteStorage_FileReadAsyncComplete(
                self.file.rs,
                callback.m_hFileReadAsync,
                buf.as_mut_ptr() as *mut _,
                callback.m_cubRead,
            );

            self.offset += size;
            Ok(size)
        }
    }
}

impl<Manager> std::io::Seek for SteamFileReader<Manager> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        match pos {
            std::io::SeekFrom::Current(o) => {
                if self.offset as isize + o as isize >= self.size as isize {
                    return Err(std::io::ErrorKind::InvalidInput.into());
                }
                self.offset = (self.offset as isize + o as isize) as usize;
            }
            std::io::SeekFrom::End(o) => {
                if o as isize >= self.size as isize {
                    return Err(std::io::ErrorKind::InvalidInput.into());
                }
                self.offset = (self.size as isize - 1 - o as isize) as usize;
            }
            std::io::SeekFrom::Start(o) => {
                if o as usize >= self.size {
                    return Err(std::io::ErrorKind::InvalidInput.into());
                }
                self.offset = o as usize;
            }
        }
        Ok(self.offset as u64)
    }
}

/// Name and size information about a file in the steam cloud
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SteamFileInfo {
    /// The file name
    pub name: String,
    /// The size of the file in bytes
    pub size: u64,
}

#[test]
#[serial]
fn test_cloud() {
    use std::io::{Read, Write};
    let client = Client::init().unwrap();

    let rs = client.remote_storage();
    println!("Listing files:");
    for f in rs.files() {
        println!("{:?}", f);
    }

    {
        let test = rs.file("test.txt");
        let mut w = test.write();
        write!(w, "Testing").unwrap();
    }

    println!("Listing files:");
    for f in rs.files() {
        println!("{:?}", f);
    }

    let mut output = String::new();
    let test = rs.file("test.txt");
    test.read().read_to_string(&mut output).unwrap();
    println!("Got: {:?}", output);

    assert_eq!(output, "Testing");
}
