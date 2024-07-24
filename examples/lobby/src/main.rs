use std::sync::mpsc;

use steamworks::*;

fn main() {
    let (client, single) = Client::init().unwrap();

    let matchmaking = client.matchmaking();

    let (sender_create_lobby, receiver_create_lobby) = mpsc::channel();

    // let mut lobby_id_state: Option<LobbyId> = None;

    matchmaking.create_lobby(LobbyType::Private, 4, move |result| match result {
        Ok(lobby_id) => {
            sender_create_lobby.send(lobby_id).unwrap();
            println!("Created lobby: [{}]", lobby_id.raw())
        }
        Err(err) => panic!("Error: {}", err),
    });

    client.register_callback(move |message: LobbyChatMsg| {
        println!("Lobby chat message received: {:?}", message);
    });

    loop {
        single.run_callbacks();

        if let Ok(lobby_id) = receiver_create_lobby.try_recv() {
            // lobby_id_state = Some(lobby_id);

            println!("Sending message to lobby chat...");
            matchmaking
                .send_lobby_chat_message(lobby_id, &[0, 1, 2, 3, 4])
                .expect("Failed to send chat message to lobby");
        }
    }
}
