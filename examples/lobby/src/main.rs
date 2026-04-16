use std::sync::mpsc;

use steamworks::*;

fn main() {
    let client = Client::init().unwrap();

    let matchmaking = client.matchmaking();

    let (sender_create_lobby, receiver_create_lobby) = mpsc::channel();

    matchmaking.create_lobby(LobbyType::Private, 4, move |result| match result {
        Ok(lobby_id) => {
            sender_create_lobby.send(lobby_id).unwrap();
            println!("Created lobby: [{}]", lobby_id.raw())
        }
        Err(err) => panic!("Error: {}", err),
    });

    loop {
        client.process_callbacks(|event| {
            if let CallbackResult::LobbyChatMsg(message) = event {
                println!("Lobby chat message received: {:?}", message);
                let mut buffer = vec![0; 256];
                let buffer = matchmaking.get_lobby_chat_entry(
                    message.lobby,
                    message.chat_id,
                    buffer.as_mut_slice(),
                );
                println!("Message buffer: [{:?}]", buffer);
            }
        });

        if let Ok(lobby_id) = receiver_create_lobby.try_recv() {
            println!("Sending message to lobby chat...");
            matchmaking
                .send_lobby_chat_message(lobby_id, &[0, 1, 2, 3, 4, 5])
                .expect("Failed to send chat message to lobby");
        }
    }
}
