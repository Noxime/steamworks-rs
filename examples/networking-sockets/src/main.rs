use std::collections::HashMap;

use macroquad::prelude::*;
use steamworks::{
    networking_types::{NetworkingIdentity, SendFlags},
    FriendFlags,
};

// 480 is Spacewar!, the Steamworks SDK example app.
const APP_ID: u32 = 480;

#[macroquad::main("steamworks-rs-networking-sockets")]
async fn main() {
    let client =
        steamworks::Client::init_app(APP_ID).expect("Steam is not running or has not been detected");

    // Get the API interfaces
    let friends = client.friends();
    let messages = client.networking_sockets();

}