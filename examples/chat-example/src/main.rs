use clipboard::{ClipboardContext, ClipboardProvider};
use macroquad::prelude::*;
use macroquad::ui::*;
use std::sync::mpsc;
use steamworks::*;

//MAYBE IT'S NOT GOOD GAME ARCHITECTURE
struct GameState {
    state: Box<States>,
}

enum States {
    Menu(MenuState),
    Chat(ChatState),
}

struct MenuState {
    lobby_input: String,
}

struct ChatState {
    own_id: SteamId,
    current_lobby: LobbyId,
    is_host: bool,
    host_id: SteamId,
    peers: Vec<SteamId>,
    message: String,
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Test".parse().unwrap(),
        window_resizable: false,
        window_width: 800,
        window_height: 600,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let (client, single) = Client::init().unwrap();

    let matchmaking = client.matchmaking();
    let networking = client.networking();

    let mut state = GameState {
        state: Box::new(States::Menu(MenuState {
            lobby_input: String::new(),
        })),
    };

    //For getting values from callback
    let (sender_create_lobby, receiver_create_lobby) = mpsc::channel();
    let (sender_join_lobby, receiver_join_lobby) = mpsc::channel();
    let (sender_accept, receiver_accept) = mpsc::channel();

    //YOU MUST KEEP CALLBACK IN VARIABLE OTHERWISE CALLBACK WILL NOT WORK
    let _request_callback = client.register_callback(move |request: P2PSessionRequest| {
        println!("ACCEPTED PEER");
        sender_accept.send(request.remote).unwrap();
    });

    //Should be in ChatState :)
    let mut messages: Vec<String> = vec![];

    loop {
        single.run_callbacks();
        clear_background(BLACK);

        let local_sender_create_lobby = sender_create_lobby.clone();
        let local_sender_join_lobby = sender_join_lobby.clone();

        match state.state.as_mut() {
            States::Menu(menu_state) => {
                widgets::Group::new(hash!(), vec2(800.0, 600.)).ui(&mut *root_ui(), |ui| {
                    //Creating lobby by button
                    if ui.button(vec2(20.0, 40.0), "Create Lobby") {
                        matchmaking.create_lobby(LobbyType::FriendsOnly, 4, move |lobby| {
                            match lobby {
                                Ok(lobby) => {
                                    local_sender_create_lobby.send(lobby).unwrap();
                                }
                                Err(_) => {}
                            };
                        });
                    }

                    //Try to join in lobby with id from InputField
                    if ui.button(vec2(120.0, 40.0), "Connect") {
                        let lobby_id: Result<u64, _> = menu_state.lobby_input.parse();
                        match lobby_id {
                            Ok(id) => {
                                matchmaking.join_lobby(LobbyId::from_raw(id), move |result| {
                                    if let Ok(lobby) = result {
                                        local_sender_join_lobby.send(lobby).unwrap();
                                    }
                                });
                            }
                            Err(_) => {}
                        }
                    }

                    widgets::Group::new(hash!(), vec2(200.0, 25.0))
                        .position(vec2(20.0, 60.0))
                        .ui(ui, |ui| {
                            ui.input_text(hash!(), "", &mut menu_state.lobby_input);
                        });
                });
            }
            States::Chat(lobby_state) => {
                widgets::Group::new(hash!(), vec2(800.0, 600.)).ui(&mut *root_ui(), |ui| {
                    draw_text_ex(
                        &format!("LobbyID: {}", lobby_state.current_lobby.raw()),
                        20.0,
                        20.0,
                        TextParams::default(),
                    );

                    for (id, message) in messages.iter().enumerate() {
                        draw_text_ex(
                            message,
                            20.0,
                            40.0 + 20.0 * (id as f32),
                            TextParams::default(),
                        );
                    }

                    widgets::Group::new(hash!(), vec2(250.0, 25.0))
                        .position(vec2(20.0, 570.0))
                        .ui(ui, |ui| {
                            ui.input_text(hash!(), "", &mut lobby_state.message);
                        });

                    //Little Client-Server logic
                    //Host is server. Host sends message to all players
                    //Clients send message only to host
                    if ui.button(vec2(300.0, 570.0), "Send") {
                        let message =
                            format!("{}: {}", client.friends().name(), lobby_state.message);
                        if lobby_state.is_host {
                            for peer in lobby_state.peers.iter() {
                                if peer.raw() != lobby_state.own_id.raw() {
                                    println!("SENT {} TO {}", message, peer.raw());
                                    networking.send_p2p_packet(
                                        *peer,
                                        SendType::Reliable,
                                        message.as_bytes(),
                                    );
                                }
                            }
                        } else {
                            let result = networking.send_p2p_packet(
                                lobby_state.host_id,
                                SendType::Reliable,
                                message.as_bytes(),
                            );
                            println!(
                                "SENT {} TO {}. RESULT: {}",
                                message,
                                lobby_state.host_id.raw(),
                                result
                            );
                        }
                        messages.push(message);
                        lobby_state.message = String::new();
                    }
                });

                while let Some(size) = networking.is_p2p_packet_available() {
                    let mut empty_array = vec![0; size];
                    let mut buffer = empty_array.as_mut_slice();
                    if let Some((sender, _)) = networking.read_p2p_packet(&mut buffer) {
                        //Host gets message from one of the players and sends this message to other players
                        if lobby_state.is_host {
                            for peer in lobby_state.peers.iter() {
                                if peer.raw() != lobby_state.own_id.raw()
                                    && peer.raw() != sender.raw()
                                {
                                    networking.send_p2p_packet(
                                        *peer,
                                        SendType::Reliable,
                                        format!(
                                            "{}: {}",
                                            client.friends().name(),
                                            lobby_state.message
                                        )
                                        .as_bytes(),
                                    );
                                }
                            }
                        }

                        if let Ok(message) = String::from_utf8(Vec::from(buffer)) {
                            println!("GOT MESSAGE: {}", message);
                            messages.push(message);
                        }
                    }
                }

                if let Ok(user) = receiver_accept.try_recv() {
                    println!("GET REQUEST FROM {}", user.raw());
                    if lobby_state.is_host {
                        lobby_state.peers.push(user);
                    }
                    networking.accept_p2p_session(user);
                }
            }
        }

        if let Ok(lobby) = receiver_create_lobby.try_recv() {
            println!("CREATED LOBBY WITH ID: {}", lobby.raw());
            let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
            let _ = ctx.set_contents(lobby.raw().to_string());
            state.state = Box::new(States::Chat(ChatState {
                own_id: client.user().steam_id(),
                current_lobby: lobby,
                is_host: true,
                host_id: client.user().steam_id(),
                peers: vec![client.user().steam_id()],
                message: String::new(),
            }));
        }

        if let Ok(lobby) = receiver_join_lobby.try_recv() {
            println!("JOINED TO LOBBY WITH ID: {}", lobby.raw());
            let host_id = matchmaking.lobby_owner(lobby);
            state.state = Box::new(States::Chat(ChatState {
                own_id: client.user().steam_id(),
                current_lobby: lobby,
                is_host: false,
                host_id,
                peers: vec![],
                message: String::new(),
            }));

            //When you connected to lobby you have to send a "ping" message to host
            //After that host will add you into peer list
            networking.send_p2p_packet(
                host_id,
                SendType::Reliable,
                format!("{} JOINED", client.friends().name()).as_bytes(),
            );
        }

        next_frame().await
    }
}
