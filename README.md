# steamworks

[![crates.io](https://img.shields.io/crates/v/steamworks.svg)](https://crates.io/crates/steamworks)
[![Documentation](https://docs.rs/steamworks/badge.svg)](https://docs.rs/steamworks)
![License](https://img.shields.io/crates/l/steamworks.svg)

This crate provides Rust bindings to the [Steamworks SDK](https://partner.steamgames.com/doc/sdk).

## Usage

Add the following to your `Cargo.toml`:

```toml
[dependencies]
steamworks = { git = "https://github.com/Jackson0ne/steamworks-rs" }
```

| Crate  | SDK   | MSRV   |
| ------ | ----- | ------ |
| git    | 1.57  | 1.56.1 |
| 0.10.0 | 1.54  | 1.56.1 |
| 0.9.0  | 1.53a | 1.56.1 |

## Example

In addition to the standard functionality of `steamworks-rs` demonstrated in the [original Readme](https://github.com/Noxime/steamworks-rs#example), I've added various achievement-based Steamworks functions:

```rust
use steamworks::{Client,AppId};

fn main() {
    let (client,single) = Client::init_app(AppId(4000)).unwrap();
    let name = "GMA_BALLEATER";

    // Potentially called via `Client::init_app,
    // so may not be necessary
    client.user_stats().request_current_stats();

    client.user_stats().request_global_achievement_percentages(move|result| {
        if !result.is_err() {
            let user_stats = client.user_stats();
            let achievement = user_stats.achievement(name);

            let ach_percent = achievement.get_achievement_achieved_percent().unwrap();

            let ach_name = achievement.get_achievement_display_attribute("name").unwrap();
            let ach_desc = achievement.get_achievement_display_attribute("desc").unwrap();
            let ach_hidden = achievement.get_achievement_display_attribute("hidden").unwrap().parse::<u32>().unwrap();

            println!(
                "Name: {:?}\nDesc: {:?}\nPercent: {:?}\nHidden?: {:?}",
                ach_name,
                ach_desc,
                ach_percent,
                ach_hidden != 0
            );

            let _ach_icon_handle = achievement.get_achievement_icon().expect("Failed getting achievement icon RGBA buffer");
        } else {
            println!("Error fetching achievement percentage for {}",name);
        }
    });

    for _ in 0..50 {
        single.run_callbacks();
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
```

## Features

`serde`: This feature enables serialization and deserialization of some types with `serde`.

## License

This crate is dual-licensed under [Apache](./LICENSE-APACHE) and [MIT](./LICENSE-MIT).

## Help, I can't run my game!

If you are seeing errors like `STATUS_DLL_NOT_FOUND`, `Image not found` etc. You are likely missing the Steamworks SDK Redistributable files. Steamworks-rs loads the SDK dynamically, so the libraries need to exist somewhere the operating system can find them. This is likely next to your game binary (.exe on windows). You can find the required files in the SDK release ZIP, under `lib\steam\redistributable_bin`. See #63 for further details
