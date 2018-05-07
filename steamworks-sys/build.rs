extern crate bindgen;
extern crate cc;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

#[derive(Deserialize)]
struct SteamApi {
    structs: Vec<SteamStruct>,
    enums: Vec<SteamEnum>,
}

#[derive(Deserialize)]
struct SteamEnum {
    enumname: String,
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
fn main() {
    use std::env;
    use std::path::{Path, PathBuf};
    use std::io::Write;
    use std::fmt::Write as FWrite;
    use std::fs::File;

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
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

    let mut builder = bindgen::builder()
        .header("wrapper.hpp")
        .clang_arg(format!("-I{}", sdk_loc.join("public/steam").display()))
        .ctypes_prefix("libc")
        .rustfmt_bindings(false);

    // Steamworks uses packed structs making them hard to work
    // with normally
    let steam_api: SteamApi = serde_json::from_reader(File::open(sdk_loc.join("public/steam/steam_api.json")).unwrap())
        .unwrap();

    let mut cpp_wrapper = String::from(r#"
#include <steam_api.h>
#include <steam_gameserver.h>
#include <stdint.h>
extern "C" {
    "#);

    fn field_type_fix(fty: &str) -> (&str, String){
        let fty = {
            if fty.contains("enum") {
                fty.trim_left_matches("enum ")
            } else if fty.contains("struct") {
                fty.trim_left_matches("struct ")
            } else if fty == "_Bool" {
                "bool"
            } else {
                fty
            }
        };
        let fty_rust = if fty == "const char*" || fty ==  "*const char" || fty ==  "const char *" {
            "*const libc::c_char".to_owned()
        } else if fty == "char*" {
            "*mut libc::c_char".to_owned()
        } else if fty == "const char **" {
            "*mut *const libc::c_char".to_owned()
        }else if fty.ends_with("*") {
            if fty.starts_with("const") {
                let trimmed = fty.trim_left_matches("const ").trim_right_matches("*");
                format!("*const {}", trimmed)
            } else {
                let trimmed = fty.trim_right_matches("*");
                format!("*mut {}", trimmed)
            }
        } else if fty.contains("[") {
            panic!("Unsupported field type array")
        } else if fty == "class CSteamID" {
            "u64".to_owned()
        } else if fty == "class CGameID" {
            "u64".to_owned()
        } else if fty == "int" {
            "libc::c_int".to_owned()
        } else if fty == "float" {
            "libc::c_float".to_owned()
        } else if fty == "double" {
            "libc::c_double".to_owned()
        } else {
            fty.to_owned()
        };
        (fty, fty_rust)
    }

    for &SteamStruct{struct_: ref ty, ref fields} in &steam_api.structs {
        if ty.contains("::") || !ty.ends_with("_t") || fields.iter().any(|v| v.fieldtype.contains('['))
            || ty.chars().next().map_or(true, |v| v.is_lowercase())
            || ty.starts_with("GSStats")
        {
            continue;
        }
        builder = builder.whitelist_type(ty)
                         .opaque_type(ty);

        // Make a raw constructor
        writeln!(cpp_wrapper, r#"{ty} __rust_helper_raw__{ty}() {{
            {ty} created_type = {{}};
            return created_type;
        }}"#, ty = ty).unwrap();
        builder = builder.raw_line(format!(r#"
            extern "C" {{
                fn __rust_helper_raw__{ty}() -> {ty};
            }}
            pub unsafe fn create_empty_{ty}() -> {ty} {{
                __rust_helper_raw__{ty}()
            }}
        "#, ty = ty));

        // Make a typed constructor
        let mut typed_constr_extern = String::new();
        let mut typed_constr_wrap = String::new();
        write!(cpp_wrapper, r#"{ty} __rust_helper_typed__{ty}("#, ty = ty).unwrap();
        write!(typed_constr_extern, r#"extern "C" {{
            fn __rust_helper_typed__{ty}("#, ty = ty).unwrap();
        write!(typed_constr_wrap, r#"pub unsafe fn create_{ty}("#, ty = ty).unwrap();
        for (idx, &SteamField{fieldname: ref fname, fieldtype: ref fty}) in fields.iter().enumerate() {
            let (fty, fty_rust) = field_type_fix(fty);
            write!(cpp_wrapper, "{} {}", fty, fname).unwrap();
            write!(typed_constr_extern, "{}: {},", fname, fty_rust).unwrap();
            write!(typed_constr_wrap, "{}: {},", fname, fty_rust).unwrap();
            if idx != fields.len() - 1 {
                cpp_wrapper.push(',');
            }
        }

        write!(cpp_wrapper, r#") {{
            {ty} created_type = {{}};
        "#, ty = ty).unwrap();
        write!(typed_constr_extern, r#") -> {ty};
        }}"#, ty = ty).unwrap();
        write!(typed_constr_wrap, r#") -> {ty} {{
            __rust_helper_typed__{ty}("#, ty = ty).unwrap();
        for &SteamField{fieldname: ref fname, ..} in fields.iter() {
            write!(cpp_wrapper, "created_type.{fname} = {fname};", fname = fname).unwrap();
            write!(typed_constr_wrap, "{},", fname).unwrap();
        }
        writeln!(cpp_wrapper, r#"
            return created_type;
        }}"#).unwrap();
        writeln!(typed_constr_wrap, r#")
        }}"#).unwrap();

        builder = builder.raw_line(typed_constr_extern);
        builder = builder.raw_line(typed_constr_wrap);


        for &SteamField{fieldname: ref fname, fieldtype: ref fty} in fields.iter() {
            let (fty, fty_rust) = field_type_fix(fty);
            builder = builder.whitelist_type(fty);
            // Generate getters/setters for fields
            if fty == "class CSteamID" {
                writeln!(cpp_wrapper, r#"
                uint64_t __rust_helper_getter__{ty}_{fname}(const {ty}* from) {{
                    return from->{fname}.ConvertToUint64();
                }}
                void __rust_helper_setter__{ty}_{fname}({ty}* to, uint64_t val) {{
                    to->{fname}.SetFromUint64(val);
                }}
                "#, ty = ty, fname = fname).unwrap();
            } else if fty == "class CGameID" {
                writeln!(cpp_wrapper, r#"
                uint64_t __rust_helper_getter__{ty}_{fname}(const {ty}* from) {{
                    return from->{fname}.ToUint64();
                }}
                void __rust_helper_setter__{ty}_{fname}({ty}* to, uint64_t val) {{
                    to->{fname}.Set(val);
                }}
                "#, ty = ty, fname = fname).unwrap();
            } else {
                writeln!(cpp_wrapper, r#"
                {fty} __rust_helper_getter__{ty}_{fname}(const {ty}* from) {{
                    return from->{fname};
                }}
                void __rust_helper_setter__{ty}_{fname}({ty}* to, {fty} val) {{
                    to->{fname} = val;
                }}
                "#, ty = ty, fty = fty, fname = fname).unwrap();
            };

            builder = builder.raw_line(format!(r#"
                extern "C" {{
                    fn __rust_helper_getter__{ty}_{fname}(from: *const {ty}) -> {fty_rust};
                    fn __rust_helper_setter__{ty}_{fname}(from: *mut {ty}, val: {fty_rust});
                }}
                impl {ty} {{
                    pub unsafe fn get_{fname}(&self) -> {fty_rust} {{
                        __rust_helper_getter__{ty}_{fname}(self)
                    }}
                    pub unsafe fn set_{fname}(&mut self, val: {fty_rust}) {{
                        __rust_helper_setter__{ty}_{fname}(self, val)
                    }}
                }}
            "#, ty = ty, fty_rust = fty_rust, fname = fname))
        }
    }
    for e in steam_api.enums {
        builder = builder.whitelist_type(e.enumname);
    }
    builder = builder.whitelist_type("EServerMode");

    cpp_wrapper.push_str("}");

    File::create(out_path.join("steam_gen.cpp"))
        .unwrap()
        .write_all(cpp_wrapper.as_bytes())
        .unwrap();

    // panic!("{}", out_path.join("steam_gen.cpp").display());
    cc::Build::new()
        .cpp(true)
        .include(sdk_loc.join("public/steam"))
        .file("src/lib.cpp")
        .file(out_path.join("steam_gen.cpp"))
        .compile("steamrust");

    let bindings = builder
        .generate()
        .unwrap();
    // panic!("{}", bindings.to_string());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}