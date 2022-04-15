# steamworks
[![crates.io](https://img.shields.io/crates/v/steamworks.svg)](https://crates.io/crates/steamworks)
[![Documentation](https://docs.rs/steamworks/badge.svg)](https://docs.rs/steamworks)
![License](https://img.shields.io/crates/l/steamworks.svg)

This crate provides rust friendly bindings to the [steamworks sdk](https://partner.steamgames.com/doc/sdk).


## Usage
Add the following to your `Cargo.toml`:

```toml
[dependencies]
steamworks = "0.9.0"
```

## Example
You can find more examples in [examples](examples).
```rust
use steamworks::AppId;
use steamworks::Client;
use steamworks::FriendFlags;
use steamworks::PersonaStateChange;

fn main() {
    let (client, single) = Client::init().unwrap();

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
        single.run_callbacks();
        ::std::thread::sleep(::std::time::Duration::from_millis(100));
    }
}
```

## Features
`serde`: This feature enables serialization and deserialization of some types with `serde`.

## License
This crate is dual-licensed under [Apache](./LICENSE-APACHE) and [MIT](./LICENSE-MIT).