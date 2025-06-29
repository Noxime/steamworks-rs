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
If you are seeing errors like `STATUS_DLL_NOT_FOUND`, `Image not found` etc. You are likely missing the Steamworks SDK Redistributable files. Steamworks-rs loads the SDK dynamically, so the libraries need to exist somewhere the operating system can find them. This is likely next to your game binary (.exe on windows). You can find the required files in the SDK release ZIP, under `lib\steam\redistributable_bin`. See #63 for further details
