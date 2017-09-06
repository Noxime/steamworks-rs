
use std::env;
use std::path::Path;

fn main() {
    let sdk_loc = env::var("STEAM_SDK_LOCATION")
        .expect("STEAM_SDK_LOCATION must be set");
    let sdk_loc = Path::new(&sdk_loc);

    let triple = env::var("TARGET").unwrap();
    let mut lib = "steam_api";
    let path = if triple.contains("windows") {
        if triple.contains("i686") {
            sdk_loc.join("redistributable_bin/")
        } else {
            lib = "steam_api64";
            sdk_loc.join("redistributable_bin/win64")
        }
    } else if triple.contains("linux") {
        if triple.contains("i686") {
            sdk_loc.join("redistributable_bin/linux32")
        } else {
            sdk_loc.join("redistributable_bin/linux64")
        }
    } else if triple.contains("darwin") {
        sdk_loc.join("redistributable_bin/osx64")
    } else {
        panic!("Unsupported OS");
    };
    println!("cargo:rustc-link-search={}", path.display());
    println!("cargo:rustc-link-lib=dylib={}", lib);
}