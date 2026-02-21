use std::collections::HashMap;

use macroquad::prelude::*;
use steamworks::{
    networking_types::{NetworkingIdentity, SendFlags},
    FriendFlags, Friends,
};

mod states;

// 480 is Spacewar!, the Steamworks SDK example app.
const APP_ID: u32 = 480;

#[macroquad::main("steamworks-rs-networking-sockets")]
async fn main() {
    prevent_quit();
    let client = steamworks::Client::init_app(APP_ID)
        .expect("Steam is not running or has not been detected");

    request_new_screen_size(1280.0, 720.0);

    let mut state = states::State::new(client.clone());

    loop {
        state = state.process_render();
        if is_quit_requested() {
            break;
        }
        next_frame().await;
    }

    // there is a nasty bug where the network thread keeps running just a tiny bit after Drop / Close has been called,
    // and unfortunately if we shutdown Steam *right* after shutting down the NetConnection, we may get use-after-free,
    // various race conditions and sigsev.
    // Since there is no way to wait until all the threads are cleaned up, the only thing we can do is wait
    // a few milliseconds to give time to steam to clean everything up... not pretty but what choice do we have?
    drop(state);
    std::thread::sleep(std::time::Duration::from_millis(50));
    drop(client);
}
