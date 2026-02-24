# steamworks
[![crates.io](https://img.shields.io/crates/v/steamworks.svg)](https://crates.io/crates/steamworks)
[![Documentation](https://docs.rs/steamworks/badge.svg)](https://docs.rs/steamworks)
![License](https://img.shields.io/crates/l/steamworks.svg)

This crate provides Rust bindings to the [Steamworks SDK](https://partner.steamgames.com/doc/sdk).

## Usage
Add the following to your `Cargo.toml`:

```toml
[dependencies]
steamworks = "0.12.0"
```

### Extra note for Linux & MacOS linking
Since Steamworks-rs loads the SDK dynamically, you need to tell the linker to look for the dynamic library next to your executable. You can do so by using a build script:

Create a `build.rs` file next to your `Cargo.toml`:

```rust
fn main() {
    #[cfg(target_os = "macos")]
    println!("cargo::rustc-link-arg=-Wl,-rpath,@executable_path");

    #[cfg(target_os = "linux")]
    println!("cargo::rustc-link-arg=-Wl,-rpath,$ORIGIN");
}

```

| Crate  | SDK   | MSRV   |
| ------ | ----- | ------ |
| git    | 1.62  | 1.80.0 |
| 0.12.0 | 1.62  | 1.80.0 |
| 0.11.0 | 1.58a | 1.71.1 |
| 0.10.0 | 1.54  | 1.56.1 |
| 0.9.0  | 1.53a | 1.56.1 |

## Example
You can find more examples in [examples](examples/).
```rust
use steamworks::AppId;
use steamworks::Client;
use steamworks::FriendFlags;
use steamworks::PersonaStateChange;

fn main() {
    let client = Client::init().unwrap();

    let _cb = client.register_callback(|p: PersonaStateChange| {
        println!("Got callback: {:?}", p);
    });

    let utils = client.utils();
    println!("Utils:");
    println!("AppId: {:?}", utils.app_id());
    println!("UI Language: {}", utils.ui_language());

    let apps = client.apps();
    println!("Apps");
    println!("IsInstalled(480): {}", apps.is_app_installed(AppId(480)));
    println!("InstallDir(480): {}", apps.app_install_dir(AppId(480)));
    println!("BuildId: {}", apps.app_build_id());
    println!("AppOwner: {:?}", apps.app_owner());
    println!("Langs: {:?}", apps.available_game_languages());
    println!("Lang: {}", apps.current_game_language());
    println!("Beta: {:?}", apps.current_beta_name());

    let friends = client.friends();
    println!("Friends");
    let list = friends.get_friends(FriendFlags::IMMEDIATE);
    println!("{:?}", list);
    for f in &list {
        println!("Friend: {:?} - {}({:?})", f.id(), f.name(), f.state());
        friends.request_user_information(f.id(), true);
    }

    for _ in 0..50 {
        client.run_callbacks();
        ::std::thread::sleep(::std::time::Duration::from_millis(100));
    }
}
```

## Features
`serde`: This feature enables serialization and deserialization of some types with `serde`.
`image`: This feature allows accessing image data like icons with `image` crate.

## License
This crate is dual-licensed under [Apache](./LICENSE-APACHE) and
[MIT](./LICENSE-MIT), except for the files in [`steamworks-sys/lib/steam/`]

## Help, I can't run my game!
If you are seeing errors like `STATUS_DLL_NOT_FOUND`, `Image not found` etc. You are likely missing the Steamworks SDK Redistributable files. In this case, please make sure you ship the Steamworks dynamic library with your game. If you can not find the required files, you can download the SDK release ZIP, and find them under `lib\steam\redistributable_bin`. See #63 for further details
