use eframe::{egui::*, *};
use steamworks::{
    networking_types::{NetworkingIdentity, SendFlags},
    FriendFlags,
};

fn main() -> eframe::Result {
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

    // UI state
    let mut text_field = "Hello, world!".to_string();
    let mut message_history = vec![];

    run_simple_native("steamworks-rs", Default::default(), move |ctx, _| {
        // Run callback periodically, this is usually your main game loop or networking thread
        client.run_callbacks();
        ctx.request_repaint();

        CentralPanel::default().show(ctx, |ui| {
            let text_height = ui.text_style_height(&TextStyle::Body);

            // Get a list of friends who are playing Spacewar!
            let mut friend_list = friends.get_friends(FriendFlags::IMMEDIATE);
            friend_list.retain(|f| f.game_played().map_or(false, |g| g.game.app_id().0 == 480));

            // Show the friend list
            SidePanel::left("friends").show_inside(ui, |ui| {
                ui.heading(format!("Logged in: {}", friends.name()));
                ui.label(format!("Online friends: {}", friend_list.len()));

                // Show the list of friends
                ScrollArea::both().show_rows(ui, text_height, friend_list.len(), |ui, range| {
                    for friend in &friend_list[range] {
                        ui.monospace(friend.name());
                    }
                });
            });

            // Receive any pending messages
            let new_messages = messages.receive_messages_on_channel(0, 10);
            for msg in new_messages {
                println!("Received message #{:?}", msg.message_number());

                let peer = msg.identity_peer();
                let data = std::str::from_utf8(msg.data()).expect("Peer sent invalid UTF-8");

                message_history.push(format!("{peer:?}: {data}"));
            }

            // Show message history
            ui.heading(format!("Chat history ({} messages)", message_history.len()));
            ScrollArea::both().auto_shrink([false, true]).show_rows(
                ui,
                text_height,
                message_history.len(),
                |ui, range| {
                    for msg in &message_history[range] {
                        ui.label(msg);
                    }
                },
            );

            // Text box for inputting a message and a button to send it
            TopBottomPanel::bottom("textbox").show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut text_field).request_focus();

                    // Send message to all friends
                    if ui.button("Send message").clicked() {
                        for friend in &friend_list {
                            println!("Sending to {:?}", friend.id());

                            if let Err(err) = messages.send_message_to_user(
                                NetworkingIdentity::new_steam_id(friend.id()),
                                SendFlags::RELIABLE,
                                text_field.as_bytes(),
                                0,
                            ) {
                                eprintln!("Send error: {err:?}");
                            }
                        }

                        // We can't send message to ourselves, so add it to chat history manually
                        message_history.push(format!("Me: {text_field}"));
                    }
                });
            });
        });
    })
}
