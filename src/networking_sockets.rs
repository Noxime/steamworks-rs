use super::*;
use std::net::SocketAddr;
use std::sync::Arc;

/// Access to the steam networking sockets interface
pub struct NetworkingSockets<Manager> {
    pub(crate) sockets: *mut sys::ISteamNetworkingSockets,
    pub(crate) _inner: Arc<Inner<Manager>>,
}

impl<Manager> NetworkingSockets<Manager> {
    pub fn create_listen_socket_ip(
        &self,
        local_address: SocketAddr,
        config: impl IntoIterator<Item = NetworkingConfigEntry>
    ) -> ListenSocket {
        let mut local_address = SteamIpAddr::from(local_address);
        let config: Vec<_> = config.into_iter().map(|x| x.into()).collect();
        unsafe {
            let socket = sys::SteamAPI_ISteamNetworkingSockets_CreateListenSocketIP(
                self.sockets,
                local_address.as_ptr(),
                config.len() as i32,
                config.as_ptr(),
            );
            ListenSocket::from_raw(socket)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr};

    #[test]
    fn test_create_listen_socket_ip() {
        let (client, _single) = Client::init().unwrap();
        let sockets = client.networking_sockets();
        let test = sockets
            .create_listen_socket_ip(SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), 1234), vec![]);
        assert!(!test.is_invalid());
    }
}
