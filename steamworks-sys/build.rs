extern crate cc;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

#[derive(Deserialize)]
struct SteamApi {
    typedefs: Vec<SteamTypedef>,
    structs: Vec<SteamStruct>,
    enums: Vec<SteamEnum>,
}
#[derive(Deserialize)]
struct SteamTypedef {
    typedef: String,
    #[serde(rename = "type")]
    ty: String,
}

#[derive(Deserialize)]
struct SteamEnum {
    enumname: String,
    values: Vec<SteamEnumValue>,
}


#[derive(Deserialize)]
struct SteamEnumValue {
    name: String,
    value: String,
}

#[derive(Deserialize)]
struct SteamStruct {
    #[serde(rename = "struct")]
    struct_: String,
    fields: Vec<SteamField>,
}

#[derive(Deserialize)]
struct SteamField {
    fieldname: String,
    fieldtype: String,
}

#[cfg(feature = "docs-only")]
fn main() {}

#[cfg(not(feature = "docs-only"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::env;
    use std::path::{Path, PathBuf};
    use std::fmt::Write as _;
    use std::fs::{self, File};
    use std::borrow::Cow;

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let sdk_loc = env::var("STEAM_SDK_LOCATION")
        .expect("STEAM_SDK_LOCATION must be set");
    let sdk_loc = Path::new(&sdk_loc);

    let triple = env::var("TARGET").unwrap();
    let mut lib = "steam_api";
    let mut packing = 8;
    let path = if triple.contains("windows") {
        if triple.contains("i686") {
            sdk_loc.join("redistributable_bin/")
        } else {
            lib = "steam_api64";
            sdk_loc.join("redistributable_bin/win64")
        }
    } else if triple.contains("linux") {
        packing = 4;
        if triple.contains("i686") {
            sdk_loc.join("redistributable_bin/linux32")
        } else {
            sdk_loc.join("redistributable_bin/linux64")
        }
    } else if triple.contains("darwin") {
        packing = 4;
        sdk_loc.join("redistributable_bin/osx64")
    } else {
        panic!("Unsupported OS");
    };
    println!("cargo:rustc-link-search={}", path.display());
    println!("cargo:rustc-link-lib=dylib={}", lib);

    // Steamworks uses packed structs making them hard to work
    // with normally
    let steam_api: SteamApi = serde_json::from_reader(File::open(sdk_loc.join("public/steam/steam_api.json"))?)?;

    let mut bindings = r##"
use libc::*;
"##.to_owned();

    fn c_to_rust<'a>(fty: &'a str) -> Option<(&'a str, Cow<'a, str>)> {
        // Generics
        if fty == "T *" {
            return None;
        }
        let fty = {
            if fty.contains("enum") {
                if fty.contains("::") {
                    return None;
                }
                fty.trim_start_matches("enum ")
            } else if fty.contains("struct") {
                fty.trim_start_matches("struct ")
            } else if fty == "_Bool" {
                "bool"
            } else {
                fty
            }
        };
        let fty_rust = if fty == "const char*" || fty ==  "*const char" || fty ==  "const char *" {
            "*const c_char".into()
        } else if fty == "char*" {
            "*mut c_char".into()
        } else if fty == "const char **" {
            "*mut *const c_char".into()
        } else if fty.ends_with("*") {
            if fty.starts_with("const") {
                let trimmed = fty.trim_start_matches("const ").trim_end_matches("*").trim();
                format!("*const {}", c_to_rust(trimmed)?.1).into()
            } else {
                let trimmed = fty.trim_end_matches("*").trim();
                format!("*mut {}", c_to_rust(trimmed)?.1).into()
            }
        } else if fty.contains("[") {
            let open_square = fty.char_indices().find(|ic| ic.1 == '[').unwrap().0;
            let close_square = fty.char_indices().find(|ic| ic.1 == ']').unwrap().0;
            format!(
                "[{}; {}]",
                c_to_rust(&fty[..open_square].trim())?.1,
                &fty[open_square + 1..close_square],
            ).into()
        } else if fty.contains("(") {
            eprintln!("Unsupported field type function pointer: {:?}", fty);
            return None;
        } else {
            match fty {
                "int" => "c_int".into(),
                "float" => "c_float".into(),
                "double" => "c_double".into(),
                "void" => "c_void".into(),
                "uint8" => "u8".into(),
                "int8" => "i8".into(),
                "uint16" => "u16".into(),
                "int16" => "i16".into(),
                "uint32" => "u32".into(),
                "int32" => "i32".into(),
                "uint64" => "u64".into(),
                "int64" => "i64".into(),
                "lint64" => "i64".into(),
                "ulint64" => "u64".into(),
                "intp" => "isize".into(),
                "uintp" => "usize".into(),
                "class CSteamID" => "CSteamID".into(),
                "class CGameID" => "CGameID".into(),
                "char" => "c_char".into(),
                val if val.contains("class") => return None,
                val => val.into(),
            }
        };

        Some((fty, fty_rust))
    }

    for def in steam_api.typedefs {
        if def.typedef.chars().next().map_or(true, |v| v.is_lowercase()) {
            continue;
        }

        if let Some(rty) = c_to_rust(&def.ty) {
            if def.typedef == rty.1 || def.typedef.contains("::") {
                continue;
            }
            writeln!(bindings, r#"
#[repr(transparent)]
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct {}(pub {});"#, def.typedef, rty.1)?;
        } else {
            eprintln!("Could not typedef {:?}", def.typedef);
        }
    }

    for e in steam_api.enums {
        if e.enumname.contains("::") {
            continue;
        }
        writeln!(bindings, r#"
#[repr(C)]
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum {} {{"#, e.enumname)?;

        for v in e.values {
            // Known duplicate
            if v.name == "k_EWorkshopFileTypeFirst" {
                continue;
            }
            writeln!(bindings, r#"    {} = {},"#, v.name.trim_start_matches("k_"), v.value)?;
        }

        writeln!(bindings, r#"}}"#)?;

    }

    'structs:
    for s in steam_api.structs {
        if s.struct_ == "CSteamID" || s.struct_ == "CGameID" || s.struct_.contains("::") {
            continue;
        }
        // TODO: Remove special case for SteamUGCDetails_t
        let derive = if !s.fields.iter().any(|v|
            v.fieldtype == "float"
            || v.fieldtype == "double"
            || v.fieldtype == "struct SteamUGCDetails_t"
            || v.fieldtype.contains('['))
        {
            "#[derive(Clone, Copy, PartialEq, Eq, Hash)]"
        } else {
            "#[derive(Clone, Copy)]"
        };
        let mut s_builder = String::new();
        writeln!(s_builder, r#"
#[repr(C, packed({}))]
{}
pub struct {} {{"#, packing, derive, s.struct_)?;

        for f in s.fields {
            if let Some(rty) = c_to_rust(&f.fieldtype) {
                writeln!(s_builder, "    pub {}: {},", f.fieldname, rty.1)?;
            } else {
                continue 'structs;
            }
        }

        writeln!(s_builder, r#"}}"#)?;

        bindings.push_str(&s_builder);
    }

    // fs::write("/tmp/steam-bindings.rs", &bindings)?;
    fs::write(out_path.join("bindings.rs"), bindings)?;

    cc::Build::new()
        .cpp(true)
        .include(sdk_loc.join("public/steam"))
        .file("src/lib.cpp")
        .compile("steamrust");

    Ok(())
}
