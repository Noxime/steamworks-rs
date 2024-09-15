use std::collections::HashMap;

use macroquad::prelude::*;
use steamworks::{
    networking_types::{NetworkingIdentity, SendFlags},
    FriendFlags,
};

#[macroquad::main("steamworks-rs")]
async fn main() {
    // 480 is Spacewar!, the Steamworks SDK example app.
    let client =
        steamworks::Client::init_app(480).expect("Steam is not running or has not been detected");

    // Get the API interfaces
    let friends = client.friends();
    let messages = client.networking_messages();

    // Even though NetworkingMessages appears as ad-hoc API, it's internally session based. We must accept any incoming
    // messages before communicating with the peer.
    messages.session_request_callback(move |req| {
        println!("Accepting session request from {:?}", req.remote());
        assert!(req.accept());
    });

    // Install a callback to debug print failed peer connections
    messages.session_failed_callback(|info| {
        eprintln!("Session failed: {info:#?}");
    });

    // Keep track of all players
    let mut peers = HashMap::new();

    loop {
        // Poll the internal callbacks
        client.run_callbacks();

        clear_background(BLACK);

        set_camera(&Camera2D::from_display_rect(Rect::new(
            -1.0, 1.0, 2.0, -2.0,
        )));

        // Draw us at our mouse position
        let me = mouse_position_local();
        draw_circle(me.x, me.y, 0.1, GREEN);

        // Send our mouse position to all friends
        for friend in friends.get_friends(FriendFlags::IMMEDIATE) {
            let identity = NetworkingIdentity::new_steam_id(friend.id());

            // Convert our position to bytes
            let mut data = [0; 8];
            data[0..4].copy_from_slice(&me.x.to_le_bytes());
            data[4..8].copy_from_slice(&me.y.to_le_bytes());

            let _ =
                messages.send_message_to_user(identity, SendFlags::UNRELIABLE_NO_DELAY, &data, 0);
        }

        // Receive messages from the network
        for message in messages.receive_messages_on_channel(0, 100) {
            let peer = message.identity_peer();
            let data = message.data();

            // Convert peer position from bytes
            let peer_x =
                f32::from_le_bytes(data[0..4].try_into().expect("Someone sent bad message"));
            let peer_y =
                f32::from_le_bytes(data[4..8].try_into().expect("Someone sent bad message"));

            peers.insert(peer.debug_string(), (peer_x, peer_y));
        }

        // Draw all peers
        for peer in peers.values() {
            draw_circle(peer.0, peer.1, 0.1, RED);
        }

        next_frame().await;
    }
}
