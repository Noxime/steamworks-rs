#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

#[cfg(target_os = "windows")]
include!("windows_bindings.rs");

#[cfg(target_os = "macos")]
include!("macos_bindings.rs");

#[cfg(target_os = "linux")]
include!("linux_bindings.rs");

pub enum SteamApiPlatform {
    Linux32,
    Linux64,
    MacOS,
    Win64,
}

impl SteamApiPlatform {
    #[cfg(all(target_os = "linux", target_arch = "x86"))]
    pub const THIS: Self = SteamApiPlatform::Linux32;
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    pub const THIS: Self = SteamApiPlatform::Linux64;
    #[cfg(target_os = "macos")]
    pub const THIS: Self = SteamApiPlatform::MacOS;
    #[cfg(target_os = "windows")]
    pub const THIS: Self = SteamApiPlatform::Win64;

    /// Returns the platform-specific library name for the Steam API.
    pub fn as_lib_name(&self) -> &'static str {
        match self {
            SteamApiPlatform::Linux32 => "linux32/libsteam_api.so",
            SteamApiPlatform::Linux64 => "linux64/libsteam_api.so",
            SteamApiPlatform::MacOS => "osx/libsteam_api.dylib",
            SteamApiPlatform::Win64 => "win64/steam_api64.dll",
        }
    }
}

impl SteamApi {
    /// Open the Steam API library from the default location.
    pub unsafe fn open() -> Result<Self, ::libloading::Error> {
        Self::new(SteamApiPlatform::THIS.as_lib_name())
    }
}

#[cfg(test)]
#[cfg(target_os = "windows")]
mod windows_tests {
    use super::SteamApi;

    #[test]
    fn load_absolute() {
        unsafe {
            SteamApi::new("lib/steam/redistributable_bin/win64/steam_api64.dll")
                .expect("Failed to load Steamworks SDK");
        }
    }

    #[test]
    fn load_relative() {
        unsafe {
            SteamApi::open().expect("Failed to load Steamworks SDK");
        }
    }

    #[test]
    fn load_wrong() {
        // This should fail because we are on windows and trying to load a macOS dylib
        unsafe {
            assert!(SteamApi::new("lib/steam/redistributable_bin/osx/libsteam_api.dylib").is_err());
        }
    }
}
