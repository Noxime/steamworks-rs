use macroquad::prelude::*;
use std::time::Instant;
use steamworks::{
    networking_sockets::{ListenSocket, NetConnection, NetworkingSockets},
    networking_types::{
        ListenSocketEvent, NetworkingConnectionState, NetworkingIdentity, SendFlags,
    },
    Client as SteamClient, FriendFlags, Friends, SteamId,
};

pub enum State {
    MainMenu(MainMenuState),
    Server(ServerState),
    Client(ClientState),
}

impl State {
    pub fn new(client: SteamClient) -> Self {
        Self::MainMenu(MainMenuState::new(client))
    }

    /// Process the state internally and render it.
    pub fn process_render(self) -> Self {
        match self {
            Self::MainMenu(main_menu) => main_menu.process_render(),
            Self::Server(server) => server.process_render(),
            Self::Client(client) => client.process_render(),
        }
    }
}

pub struct MainMenuState {
    steam: SteamClient,
    steam_friends: Friends,
    friends: Vec<(String, SteamId)>,
    next_update: Instant,
}

const START_Y: f32 = 40.0;
const FONT_SIZE: f32 = 32.0;
const PER_LINE: f32 = 50.0;
const MARGIN_X: f32 = 30.0;

impl MainMenuState {
    fn new(client: SteamClient) -> Self {
        MainMenuState {
            steam_friends: client.friends(),
            friends: Vec::new(),
            next_update: Instant::now(),
            steam: client,
        }
    }

    fn process_render(mut self) -> State {
        self.steam.run_callbacks();
        let now = Instant::now();
        if now >= self.next_update {
            // update friend list every so often
            self.friends.clear();

            for friend in self.steam_friends.get_friends(FriendFlags::IMMEDIATE) {
                if let Some(played_game) = friend.game_played() {
                    if played_game.game.app_id().0 == super::APP_ID {
                        self.friends.push((friend.name(), friend.id()));
                    }
                }
            }
            self.friends.sort_by_key(|f| f.1.raw());

            self.next_update = now + std::time::Duration::from_secs(1);
        }

        clear_background(DARKPURPLE);

        let mut pos_y = START_Y;
        draw_text("[S] to start a server", MARGIN_X, pos_y, FONT_SIZE, WHITE);
        pos_y += PER_LINE;
        draw_text(
            "list of friends to connect:",
            MARGIN_X,
            pos_y,
            FONT_SIZE,
            WHITE,
        );
        pos_y += PER_LINE;

        for (i, friend) in self.friends.iter().take(4).enumerate() {
            let text = format!(
                "[{}] to try connect to {} (id {})",
                i + 1,
                friend.0,
                friend.1.raw()
            );
            draw_text(&*text, MARGIN_X, pos_y, FONT_SIZE, WHITE);
            pos_y += PER_LINE;
        }

        let join_key_codes = &[
            (KeyCode::Key1, KeyCode::Kp1),
            (KeyCode::Key2, KeyCode::Kp2),
            (KeyCode::Key3, KeyCode::Kp3),
            (KeyCode::Key4, KeyCode::Kp4),
        ];

        if is_key_down(KeyCode::S) {
            return State::Server(ServerState::new(self.steam));
        }

        for (i, (k1, k2)) in join_key_codes.iter().enumerate() {
            if !is_key_down(*k1) && !is_key_down(*k2) {
                continue;
            }
            let Some(friend) = self.friends.get(i) else {
                continue;
            };

            return State::Client(ClientState::new(self.steam, friend.1));
        }

        State::MainMenu(self)
    }
}

pub struct ServerState {
    steam: SteamClient,
    listen_socket: ListenSocket,
    remotes: Vec<(NetworkingIdentity, NetConnection, (f32, f32))>,
}

impl ServerState {
    pub fn new(steam: SteamClient) -> Self {
        println!("starting server");
        let networking_sockets = steam.networking_sockets();

        // not necessary, call this before you know you will need this to reduce initial connection delay,
        // but it is done automatically if needed
        steam.networking_utils().init_relay_network_access();
        let listen_socket = networking_sockets
            .create_listen_socket_p2p(0, vec![])
            .expect("failed to create listener");

        Self {
            steam,
            listen_socket,
            remotes: vec![],
        }
    }

    fn process_render(mut self) -> State {
        self.steam.run_callbacks();
        clear_background(DARKBROWN);
        let txt = format!(
            "server is listening, currently connected to {} clients",
            self.remotes.len()
        );
        draw_text(&*txt, MARGIN_X, START_Y, FONT_SIZE, WHITE);

        while let Some(event) = self.listen_socket.try_receive_event() {
            match event {
                ListenSocketEvent::Connecting(connecting) => {
                    println!(
                        "received event Connecting: {:?} user_data={}",
                        connecting.remote(),
                        connecting.user_data()
                    );
                    connecting.accept().unwrap();
                }
                ListenSocketEvent::Connected(connected) => {
                    println!(
                        "received event Connected: {:?} user_data={}",
                        connected.remote(),
                        connected.user_data()
                    );
                    self.remotes.push((
                        connected.remote(),
                        connected.take_connection(),
                        (0.0, 0.0),
                    ));
                }
                ListenSocketEvent::Disconnected(disconnected) => {
                    let remote = disconnected.remote();
                    println!(
                        "received event Disconnected {:?}: {:?} user_data={}",
                        disconnected.end_reason(),
                        remote,
                        disconnected.user_data()
                    );
                    self.remotes.retain(|c| c.0 != remote);
                }
            }
        }

        for (_net_id, net_conn, (pos_x, pos_y)) in &mut self.remotes {
            net_conn.run_callbacks();

            net_conn.receive_messages_with(|msg| {
                let data = msg.data();
                if data.len() < 8 {
                    return;
                };
                let x = f32::from_le_bytes(data[0..4].try_into().unwrap());
                let y = f32::from_le_bytes(data[4..8].try_into().unwrap());
                *pos_x = x;
                *pos_y = y;
            });

            draw_circle(*pos_x, *pos_y, 5.0, WHITE);
        }
        let our_pos = mouse_position();
        draw_circle(our_pos.0, our_pos.1, 5.0, BLUE);
        State::Server(self)
    }
}

pub struct ClientState {
    steam: SteamClient,
    friend_steam_id: SteamId,
    net_connection: NetConnection,
    our_mouse_pos: (f32, f32),
}

impl ClientState {
    pub fn new(steam: SteamClient, friend_steam_id: SteamId) -> Self {
        let networking_sockets = steam.networking_sockets();

        // not necessary, call this before you know you will need this to reduce initial connection delay
        steam.networking_utils().init_relay_network_access();

        let networking_identity = NetworkingIdentity::new_steam_id(friend_steam_id.clone());
        let net_connection = networking_sockets
            .connect_p2p(networking_identity, 0, vec![])
            .expect("failed to create connection");

        Self {
            steam,
            net_connection,
            friend_steam_id,
            our_mouse_pos: (0.0, 0.0),
        }
    }

    fn process_render(mut self) -> State {
        self.steam.run_callbacks();
        self.net_connection.run_callbacks();

        let info = self.net_connection.info().unwrap();
        let friend_steam_id = self.friend_steam_id.steamid32();
        let (bg_color, txt) = match info.state().unwrap() {
            NetworkingConnectionState::Connecting => (
                BLACK,
                format!("connecting to friend {}...", friend_steam_id),
            ),
            NetworkingConnectionState::FindingRoute => {
                (BLACK, format!("pathing to friend {}...", friend_steam_id))
            }
            NetworkingConnectionState::Connected => (
                DARKBROWN,
                format!("connected to friend {}", friend_steam_id),
            ),
            _ => (
                RED,
                format!(
                    "disconnected from friend {}; reason = {:?}",
                    friend_steam_id,
                    info.end_reason()
                ),
            ),
        };
        clear_background(bg_color);
        let mut pos_y = START_Y;
        draw_text(&*txt, MARGIN_X, pos_y, FONT_SIZE, WHITE);
        pos_y += PER_LINE;
        draw_text("[esc] to go back", MARGIN_X, pos_y, FONT_SIZE, WHITE);

        if is_key_down(KeyCode::Escape) {
            return State::MainMenu(MainMenuState::new(self.steam));
        }

        while let Some(event) = self.net_connection.try_receive_event() {
            println!("received NetConnectionEvent: {:?}", event);
        }

        let local_mouse_pos = mouse_position();
        self.our_mouse_pos = local_mouse_pos;

        let mut data_to_send = [0u8; 8];
        data_to_send[0..4].copy_from_slice(&self.our_mouse_pos.0.to_le_bytes());
        data_to_send[4..8].copy_from_slice(&self.our_mouse_pos.1.to_le_bytes());

        self.net_connection
            .send_message(&data_to_send, SendFlags::RELIABLE_NO_NAGLE)
            .unwrap();

        draw_circle(self.our_mouse_pos.0, self.our_mouse_pos.1, 5.0, BLUE);

        State::Client(self)
    }
}
