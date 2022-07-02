#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

#[cfg(target_os = "windows")]
include!("windows_bindings.rs");

#[cfg(target_os = "macos")]
include!("macos_bindings.rs");

#[cfg(target_os = "linux")]
include!("linux_bindings.rs");
