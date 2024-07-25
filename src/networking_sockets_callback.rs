use crate::networking_sockets::NetConnection;
use crate::networking_types::{
    AppNetConnectionEnd, NetConnectionEnd, NetConnectionStatusChanged, NetworkingConnectionState,
};
use crate::{register_callback, CallbackHandle, Inner};
use std::sync::{Arc, Weak};
use steamworks_sys as sys;
use sys::ISteamNetworkingSockets;

/// All independent connections (to a remote host) and listening sockets share the same Callback for
/// `NetConnectionStatusChangedCallback`. This function either returns the existing handle, or creates a new
/// handler.
pub(crate) fn get_or_create_connection_callback<Manager: 'static>(
    inner: Arc<Inner<Manager>>,
    sockets: *mut ISteamNetworkingSockets,
) -> Arc<CallbackHandle<Manager>> {
    let mut network_socket_data = inner.networking_sockets_data.lock().unwrap();
    if let Some(callback) = network_socket_data.connection_callback.upgrade() {
        callback
    } else {
        let handler = ConnectionCallbackHandler {
            inner: Arc::downgrade(&inner),
            sockets,
        };
        let callback = unsafe {
            register_callback(&inner, move |event: NetConnectionStatusChanged| {
                handler.callback(event);
            })
        };

        let callback = Arc::new(callback);
        network_socket_data.connection_callback = Arc::downgrade(&callback);
        callback
    }
}

pub(crate) struct ConnectionCallbackHandler<Manager> {
    inner: Weak<Inner<Manager>>,
    sockets: *mut ISteamNetworkingSockets,
}

unsafe impl<Manager> Send for ConnectionCallbackHandler<Manager> {}
unsafe impl<Manager> Sync for ConnectionCallbackHandler<Manager> {}

impl<Manager: 'static> ConnectionCallbackHandler<Manager> {
    pub(crate) fn callback(&self, event: NetConnectionStatusChanged) {
        if let Some(socket) = event.connection_info.listen_socket() {
            self.listen_socket_callback(socket, event);
        } else {
            self.independent_connection_callback(event);
        }
    }

    fn listen_socket_callback(
        &self,
        socket_handle: sys::HSteamListenSocket,
        event: NetConnectionStatusChanged,
    ) {
        if let Some(inner) = self.inner.upgrade() {
            let data = inner.networking_sockets_data.lock().unwrap();
            if let Some((socket, sender)) = data
                .sockets
                .get(&socket_handle)
                .and_then(|(socket, sender)| socket.upgrade().map(|socket| (socket, sender)))
            {
                let connection_handle = event.connection;
                let state = event.connection_info.state().expect("invalid state");
                if let Ok(event) = event.into_listen_socket_event(socket) {
                    if let Err(_err) = sender.send(event) {
                        // If the main socket was dropped, but the inner socket still exists, reject all new connections,
                        // as there's no way to accept them.
                        if let NetworkingConnectionState::Connecting = state {
                            self.reject_connection(connection_handle);
                        }
                    }
                } else {
                    // Ignore events that couldn't be converted
                }
            }
        }
    }

    fn reject_connection(&self, connection_handle: sys::HSteamNetConnection) {
        if let Some(inner) = self.inner.upgrade() {
            NetConnection::new_internal(connection_handle, self.sockets, inner.clone()).close(
                NetConnectionEnd::App(AppNetConnectionEnd::generic_normal()).into(),
                Some("no new connections will be accepted"),
                false,
            );
        }
    }

    fn independent_connection_callback(&self, _event: NetConnectionStatusChanged) {
        // TODO: Handle event for independent connections
    }
}
