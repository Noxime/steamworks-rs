# 🏆 Achievements Example
*By [Jackson0ne](https://github.com/Jackson0ne)*

This example outlines how to use the following achievement functions from the Steamworks API within `steamworks-rs`:

`get_achievement_achieved_percent()`
-

> *⚠ This function **requires** a successful callback to be received from `request_global_achievement_percentages(...)` before it will return any data!*

Returns the percentage of users who have unlocked the specified achievement.

To use, you'll first need to call `client.user_stats().request_global_achievement_percentages(...)`, and obtain the result from within the returned callback.

#### Example:

```rust
use steamworks::{Client,AppId};

fn main() {
    let (client,single) = Client::init_app(AppId(4000)).unwrap();
    let name = "GMA_BALLEATER";

    client.user_stats().request_global_achievement_percentages(move|result| {
        if !result.is_err() {
            let user_stats = client.user_stats();
            let achievement = user_stats.achievement(name);

            let ach_percent = achievement.get_achievement_achieved_percent().unwrap();
        } else {
            eprintln!("Error fetching achievement percentage for {}",name);
        }
    });

    for _ in 0..50 {
        single.run_callbacks();
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
```

`get_achievement_display_attribute("name" | "desc" | "hidden")`
-

Returns a string for the result of the specified attribute type. Accepted values are:

- `"name"`: The friendly (*non-API*) name of the achievement
- `"desc"`: The achievement description
- `"hidden"`: Whether the achievement is hidden (`"1"`) or not (`"0"`).

> *As the returned value is always a string, the `"hidden"` value will need to be parsed for comparison!*

#### Example:

```rust
use steamworks::{Client,AppId};

fn main() {
    let (client,single) = Client::init_app(AppId(4000)).unwrap();
    let name = "GMA_BALLEATER";

    let user_stats = client.user_stats();
    let achievement = user_stats.achievement(name);

    let ach_name = achievement.get_achievement_display_attribute("name").unwrap();
    let ach_desc = achievement.get_achievement_display_attribute("desc").unwrap();
    let ach_hidden = achievement.get_achievement_display_attribute("hidden").unwrap().parse::<u32>().unwrap();

    println!(
        "Name: {:?}\nDesc: {:?}\nHidden?: {:?}",
        ach_name,
        ach_desc,
        ach_hidden != 0
    );
}
```

`get_achievement_icon()`
-

Returns a `Vec<u8>` buffer containing the image data for the specified achievement.


- The icon is always returned as `64px` x `64px`.
- The version of the icon that is downloaded - i.e. locked (*grey*) vs unlocked (*colour*) - is dependent on whether the achievement has been unlocked or not at the time the function is called.

> *As far as I can tell, there's no parameter to request a specific version!*

To convert the buffer into an image, you can use an external crate to convert the `Vec<u8>` (`Uint8Array`) into a file (such as `.jpg`) and save it to disk - there's plenty of [Rust crates](https://crates.io/crates/image) or [NPM libraries](https://www.npmjs.com/package/jpeg-js) that can do this.

#### Example:

```rust
use steamworks::{Client,AppId};

fn main() {
    let (client,single) = Client::init_app(AppId(4000)).unwrap();
    let name = "GMA_BALLEATER";

    let user_stats = client.user_stats();
    let achievement = user_stats.achievement(name);

    let _ach_icon_handle = achievement.get_achievement_icon().expect("Failed getting achievement icon RGBA buffer");
}
```

`get_num_achievements()`
-

Returns the number of achievements for the current AppId.

> *Returns `0` if the current AppId has no achievements.*

#### Example:

```rust
use steamworks::{Client,AppId};

fn main() {
    let (client,single) = Client::init_app(AppId(4000)).unwrap();

    let num = client.user_stats().get_num_achievements().expect("Failed to get number of achievements");

    println!("{}",num);
}
```

`get_achievement_names()`
-

Returns a `Vec<String>` containing the API names of all achievements for the current AppId.

> *The returned string value will be empty if the specified index is invalid.*

#### Example:

```rust
use steamworks::{Client,AppId};

fn main() {
    let (client,single) = Client::init_app(AppId(4000)).unwrap();
    let name = "GMA_BALLEATER";

    let names = client.user_stats().get_achievement_names().expect("Failed to get achievement names");
}
```