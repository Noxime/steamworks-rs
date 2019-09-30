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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::env;
    use std::path::{Path, PathBuf};
    use std::fmt::Write as _;
    use std::fs::{self, File};
    use std::borrow::Cow;

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let mut sdk_loc = env::var("CARGO_MANIFEST_DIR")
        .expect("Error getting SDK directory");
    sdk_loc.push_str("/../steamworks-sdk");

    let sdk_loc = Path::new(&sdk_loc);

    let triple = env::var("TARGET").unwrap();
    let mut lib = "steam_api";
    let mut packing = 8;
    let mut link_path = sdk_loc.join("redistributable_bin");
    if triple.contains("windows") {
        if !triple.contains("i686") {
            lib = "steam_api64";
            link_path.push("win64");
        }
    } else if triple.contains("linux") {
        packing = 4;
        if triple.contains("i686") {
            link_path.push("linux32");
        } else {
            link_path.push("linux64");
        }
    } else if triple.contains("darwin") {
        packing = 4;
        link_path.push("osx");
    } else {
        panic!("Unsupported OS");
    };
    println!("cargo:rustc-link-search={}", link_path.display());
    println!("cargo:rustc-link-lib=dylib={}", lib);

    // Steamworks uses packed structs making them hard to work
    // with normally
    let steam_api_json_loc = sdk_loc.join("public/steam/steam_api.json");
    let file = File::open(&steam_api_json_loc).expect(&format!("open {:?}", steam_api_json_loc));
    let steam_api: SteamApi = serde_json::from_reader(file)?;

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

    fs::write(out_path.join("bindings.rs"), bindings)?;

    if triple.contains("windows") {
        let file_name = format!("{}.dll", lib);
        fs::copy(link_path.join(&file_name), out_path.join(file_name))?;
    } else if triple.contains("darwin") {
        fs::copy(link_path.join("libsteam_api.dylib"), out_path.join("libsteam_api.dylib"))?;
    } else if triple.contains("linux") {
        fs::copy(link_path.join("libsteam_api.so"), out_path.join("libsteam_api.so"))?;
    }

    let mut compiler = cc::Build::new();
    compiler
        .cpp(true)
        .include(sdk_loc.join("public/steam"))
        .file("src/lib.cpp");
    if triple.contains("darwin") || triple.contains("linux") {
        compiler.flag("-std=c++11");
    }
    compiler.compile("steamrust");

    Ok(())
}
