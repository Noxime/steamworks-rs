use std::net::SocketAddrV4;
use std::str::FromStr;
use std::thread::sleep;
use std::time::{Duration, Instant};
use steamworks::CallbackResult::{
    GSClientApprove, GSClientDeny, GSClientGroupStatus, GSClientKick, P2PSessionRequest,
    SteamServersConnected,
};
use steamworks::ServerMode;

const BUFFER_SIZE: usize = 1500;

fn main() {
    let bind_address =
        SocketAddrV4::from_str("0.0.0.0:54321").expect("Could not parse bind address");

    // Client::init is not required with Server::Init as long as you set the app id via steam_appid.txt or env var
    unsafe {
        std::env::set_var("SteamAppId", "480");
    }
    let (server, _) = steamworks::Server::init(
        *bind_address.ip(), // in reality, this should be the external ip of the server
        bind_address.port() - 1,
        bind_address.port(), // For some games, this port is actually the "main" port (and the game_port is unused)
        ServerMode::AuthenticationAndSecure,
        "123456",
    )
    .expect("Could not register server with Steam Master server list");

    server.set_server_name("Rusty Server");
    server.set_dedicated_server(true);
    server.set_max_players(10);

    server.enable_heartbeats(true);
    server.log_on_anonymous();

    println!("{:?}", server.steam_id());

    let networking = server.networking();
    let networking_messages = server.networking_messages();

    networking_messages.session_request_callback(|req| {
        println!("Session request event");
        req.accept();
    });

    let mut buffer = vec![0; BUFFER_SIZE];

    loop {
        let start = Instant::now();

        server.process_callbacks(|event| match event {
            SteamServersConnected(..) => {
                println!("Steam Servers Connected");
            }
            P2PSessionRequest(request) => {
                println!(
                    "Received a new Server P2P session request for {:?}",
                    request.remote
                );
                networking.accept_p2p_session(request.remote);
            }
            GSClientApprove(info) => {
                println!("GSClientApprove {:?}", info);
            }
            GSClientDeny(denial) => {
                println!("GSClientDeny {:?}", denial);
            }
            GSClientKick(kick) => {
                println!("GSClientKick {:?}", kick);
            }
            GSClientGroupStatus(status) => {
                println!("GSClientGroupStatus {:?}", status);
            }
            _ => {
                println!("Unhandled event");
            }
        });

        let size = networking.is_p2p_packet_available_on_channel(0);
        if (size.is_some()) {
            let (sender, size) = networking
                .read_p2p_packet_from_channel(&mut buffer, 2)
                .expect("Could not read P2P packet");
            println!(
                "recv from: {:?}, size: {:?}, data: {:X?}",
                sender, size, &buffer
            );
        }

        let difference = Duration::from_secs_f32(1f32 / 60f32).checked_sub(start.elapsed());
        if (difference != None) {
            sleep(difference.unwrap());
        } else {
            println!("Event loop lagging!")
        }
    }
}
