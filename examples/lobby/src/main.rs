use std::sync::mpsc;

use steamworks::*;

fn main() {
    let client = Client::init().unwrap();

    let matchmaking = client.matchmaking();

    let (sender_create_lobby, receiver_create_lobby) = mpsc::channel();
    let (sender_lobby_chat_msg, receiver_lobby_chat_msg) = mpsc::channel();

    matchmaking.create_lobby(LobbyType::Private, 4, move |result| match result {
        Ok(lobby_id) => {
            sender_create_lobby.send(lobby_id).unwrap();
            println!("Created lobby: [{}]", lobby_id.raw())
        }
        Err(err) => panic!("Error: {}", err),
    });

    client.register_callback(move |message: LobbyChatMsg| {
        println!("Lobby chat message received: {:?}", message);
        sender_lobby_chat_msg.send(message).unwrap();
    });

    loop {
        client.run_callbacks();

        if let Ok(lobby_id) = receiver_create_lobby.try_recv() {
            println!("Sending message to lobby chat...");
            matchmaking
                .send_lobby_chat_message(lobby_id, &[0, 1, 2, 3, 4, 5])
                .expect("Failed to send chat message to lobby");
        }

        if let Ok(message) = receiver_lobby_chat_msg.try_recv() {
            let mut buffer = vec![0; 256];
            let buffer = matchmaking.get_lobby_chat_entry(
                message.lobby,
                message.chat_id,
                buffer.as_mut_slice(),
            );
            println!("Message buffer: [{:?}]", buffer);
        }
    }
}
