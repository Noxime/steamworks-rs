#[cfg(feature = "rebuild-bindings")]
extern crate bindgen;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::env;
    use std::fs::{self};
    use std::path::{Path, PathBuf};

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let sdk_loc = if let Ok(sdk_loc) = env::var("STEAM_SDK_LOCATION") {
        Path::new(&sdk_loc).to_path_buf()
    } else {
        let mut path = PathBuf::new();
        path.push(env::var("CARGO_MANIFEST_DIR").unwrap());
        path.push("lib");
        path.push("steam");
        path
    };
    println!("cargo:rerun-if-env-changed=STEAM_SDK_LOCATION");

    let triple = env::var("TARGET").unwrap();
    let mut lib = "steam_api";
    let mut link_path = sdk_loc.join("redistributable_bin");
    if triple.contains("windows") {
        if !triple.contains("i686") {
            lib = "steam_api64";
            link_path.push("win64");
        }
    } else if triple.contains("linux") {
        if triple.contains("i686") {
            link_path.push("linux32");
        } else {
            link_path.push("linux64");
        }
    } else if triple.contains("darwin") {
        link_path.push("osx");
    } else {
        panic!("Unsupported OS");
    };

    if triple.contains("windows") {
        let dll_file = format!("{}.dll", lib);
        let lib_file = format!("{}.lib", lib);
        fs::copy(link_path.join(&dll_file), out_path.join(dll_file))?;
        fs::copy(link_path.join(&lib_file), out_path.join(lib_file))?;
    } else if triple.contains("darwin") {
        fs::copy(
            link_path.join("libsteam_api.dylib"),
            out_path.join("libsteam_api.dylib"),
        )?;
    } else if triple.contains("linux") {
        fs::copy(
            link_path.join("libsteam_api.so"),
            out_path.join("libsteam_api.so"),
        )?;
    }

    println!("cargo:rustc-link-search={}", out_path.display());
    println!("cargo:rustc-link-lib=dylib={}", lib);

    #[cfg(feature = "rebuild-bindings")]
    {
        let target_os = if triple.contains("windows") {
            "windows"
        } else if triple.contains("darwin") {
            "macos"
        } else if triple.contains("linux") {
            "linux"
        } else {
            panic!("Unsupported OS");
        };
        let binding_path = Path::new(&format!("src/{}_bindings.rs", target_os)).to_owned();
        let bindings = bindgen::Builder::default()
            .header(
                sdk_loc
                    .join("public/steam/steam_api_flat.h")
                    .to_string_lossy(),
            )
            .header(
                sdk_loc
                    .join("public/steam/steam_gameserver.h")
                    .to_string_lossy(),
            )
            .clang_arg("-xc++")
            .clang_arg("-std=c++11")
            .clang_arg(format!("-I{}", sdk_loc.join("public").display()))
            .rustfmt_bindings(true)
            .default_enum_style(bindgen::EnumVariation::Rust {
                non_exhaustive: true,
            })
            .generate()
            .expect("Unable to generate bindings");

        bindings
            .write_to_file(binding_path)
            .expect("Couldn't write bindings!");
    }

    Ok(())
}
