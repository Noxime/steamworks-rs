use super::*;
use crate::cstring;
#[cfg(test)]
use serial_test::serial;

/// Access to the steam remote storage interface
pub struct RemoteStorage {
    pub(crate) rs: *mut sys::ISteamRemoteStorage,
    pub(crate) util: *mut sys::ISteamUtils,
    pub(crate) inner: Arc<Inner>,
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

impl Clone for RemoteStorage {
    fn clone(&self) -> Self {
        RemoteStorage {
            inner: self.inner.clone(),
            rs: self.rs,
            util: self.util,
        }
    }
}

impl RemoteStorage {
    /// Toggles whether the steam cloud is enabled for the application
    pub fn set_cloud_enabled_for_app(&self, enabled: bool) {
        unsafe {
            sys::SteamAPI_ISteamRemoteStorage_SetCloudEnabledForApp(self.rs, enabled);
        }
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

    /// Returns a handle to a steam cloud file
    ///
    /// The file does not have to exist.
    ///
    /// If `name` contains a NUL byte, an empty file name is used instead and the
    /// resulting handle will not refer to any file.
    pub fn file(&self, name: &str) -> SteamFile {
        SteamFile {
            rs: self.rs,
            util: self.util,
            _inner: self.inner.clone(),
            name: cstring(name).unwrap_or_default(),
        }
    }
}

bitflags! {
    /// Platform flags used with [`SteamFile::set_sync_platforms`] to restrict
    /// syncing to specific operating systems.
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[repr(C)]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    pub struct RemoteStoragePlatforms: u32 {
        const WINDOWS = sys::ERemoteStoragePlatform::k_ERemoteStoragePlatformWindows.0 as _;
        const MACOS = sys::ERemoteStoragePlatform::k_ERemoteStoragePlatformOSX.0 as _;
        const PS3 = sys::ERemoteStoragePlatform::k_ERemoteStoragePlatformPS3.0 as _;
        const LINUX = sys::ERemoteStoragePlatform::k_ERemoteStoragePlatformLinux.0 as _;
        const SWITCH = sys::ERemoteStoragePlatform::k_ERemoteStoragePlatformSwitch.0 as _;
        const ANDROID = sys::ERemoteStoragePlatform::k_ERemoteStoragePlatformAndroid.0 as _;
        const IOS = sys::ERemoteStoragePlatform::k_ERemoteStoragePlatformIOS.0 as _;
    }
}

impl From<RemoteStoragePlatforms> for sys::ERemoteStoragePlatform {
    fn from(platforms: RemoteStoragePlatforms) -> Self {
        sys::ERemoteStoragePlatform(platforms.bits() as _)
    }
}

/// A handle for a possible steam cloud file
pub struct SteamFile {
    pub(crate) rs: *mut sys::ISteamRemoteStorage,
    pub(crate) util: *mut sys::ISteamUtils,
    pub(crate) _inner: Arc<Inner>,
    name: CString,
}

impl SteamFile {
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

    /// Returns the timestamp of the file
    pub fn timestamp(&self) -> i64 {
        unsafe { sys::SteamAPI_ISteamRemoteStorage_GetFileTimestamp(self.rs, self.name.as_ptr()) }
    }

    /// Set which platforms the file should be available on
    pub fn set_sync_platforms(&self, platforms: RemoteStoragePlatforms) {
        unsafe {
            sys::SteamAPI_ISteamRemoteStorage_SetSyncPlatforms(
                self.rs,
                self.name.as_ptr(),
                platforms.into(),
            );
        }
    }

    /// Returns the platforms the file is available on
    pub fn get_sync_platforms(&self) -> RemoteStoragePlatforms {
        let bits = unsafe {
            sys::SteamAPI_ISteamRemoteStorage_GetSyncPlatforms(self.rs, self.name.as_ptr())
        };
        RemoteStoragePlatforms::from_bits_truncate(bits.0 as _)
    }

    pub fn write(self) -> SteamFileWriter {
        unsafe {
            let handle =
                sys::SteamAPI_ISteamRemoteStorage_FileWriteStreamOpen(self.rs, self.name.as_ptr());
            SteamFileWriter { file: self, handle }
        }
    }

    pub fn read(self) -> SteamFileReader {
        unsafe {
            SteamFileReader {
                offset: 0,
                size: sys::SteamAPI_ISteamRemoteStorage_GetFileSize(self.rs, self.name.as_ptr())
                    as usize,
                file: self,
            }
        }
    }

    pub fn share(&self, cb: impl FnOnce(Result<u64, SteamError>) + 'static + Send) {
        let api_call =
            unsafe { sys::SteamAPI_ISteamRemoteStorage_FileShare(self.rs, self.name.as_ptr()) };
        unsafe {
            register_call_result::<sys::RemoteStorageFileShareResult_t, _>(
                &self._inner,
                api_call,
                move |v, io_error| {
                    if io_error {
                        cb(Err(SteamError::IOFailure));
                        return;
                    }

                    cb(to_steam_result(v.m_eResult).map(|()| v.m_hFile))
                },
            )
        }
    }
}

/// A write handle for a steam cloud file
pub struct SteamFileWriter {
    file: SteamFile,
    handle: sys::UGCFileWriteStreamHandle_t,
}

impl std::io::Write for SteamFileWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        unsafe {
            if sys::SteamAPI_ISteamRemoteStorage_FileWriteStreamWriteChunk(
                self.file.rs,
                self.handle,
                buf.as_ptr().cast(),
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

impl Drop for SteamFileWriter {
    fn drop(&mut self) {
        unsafe {
            sys::SteamAPI_ISteamRemoteStorage_FileWriteStreamClose(self.file.rs, self.handle);
        }
    }
}

/// A read handle for a steam cloud file
pub struct SteamFileReader {
    file: SteamFile,
    offset: usize,
    size: usize,
}

impl std::io::Read for SteamFileReader {
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

            if to_steam_result(callback.m_eResult).is_err() {
                return Err(std::io::ErrorKind::Other.into());
            }
            let size = callback.m_cubRead as usize;
            sys::SteamAPI_ISteamRemoteStorage_FileReadAsyncComplete(
                self.file.rs,
                callback.m_hFileReadAsync,
                buf.as_mut_ptr().cast(),
                callback.m_cubRead,
            );

            self.offset += size;
            Ok(size)
        }
    }
}

impl std::io::Seek for SteamFileReader {
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

#[cfg(test)]
mod cstring_nul_degrade_tests {
    use super::*;

    #[test]
    fn steam_file_nul_name_uses_empty_name() {
        let rs = RemoteStorage {
            rs: std::ptr::null_mut(),
            util: std::ptr::null_mut(),
            inner: test_cstring_inner(),
        };
        assert!(rs.file("name\0").name.to_bytes().is_empty());
    }

    fn test_cstring_inner() -> std::sync::Arc<crate::Inner> {
        use std::collections::HashMap;
        let inner = std::sync::Arc::new(crate::Inner {
            manager: crate::Manager::Client,
            callbacks: crate::Callbacks {
                callbacks: std::sync::Mutex::new(HashMap::new()),
                call_results: std::sync::Mutex::new(HashMap::new()),
            },
            networking_sockets_data: std::sync::Mutex::new(crate::NetworkingSocketsData {
                sockets: HashMap::new(),
                independent_connections: HashMap::new(),
                connection_callback: std::sync::Weak::new(),
            }),
        });
        // Leak one strong reference: dropping Inner fires SteamAPI_Shutdown via
        // Manager::drop, and these tests never initialize the Steam API.
        std::mem::forget(std::sync::Arc::clone(&inner));
        inner
    }
}
