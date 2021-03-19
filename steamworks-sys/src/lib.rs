#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

#[cfg(not(feature = "docs-only"))]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(feature = "docs-only")]
include!("bindings.rs");