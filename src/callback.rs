use super::*;
use crate::networking_messages::*;
use crate::networking_types::*;
use crate::networking_utils::*;
use crate::screenshots::*;

use crate::sys;

use std::sync::{Arc, Weak};

/// A sum type over all possible callback results
pub enum CallbackResult {
    AuthSessionTicketResponse(AuthSessionTicketResponse),
    DownloadItemResult(DownloadItemResult),
    FloatingGamepadTextInputDismissed(FloatingGamepadTextInputDismissed),
    GameLobbyJoinRequested(GameLobbyJoinRequested),
    GameOverlayActivated(GameOverlayActivated),
    GamepadTextInputDismissed(GamepadTextInputDismissed),
    GameRichPresenceJoinRequested(GameRichPresenceJoinRequested),
    LobbyChatMsg(LobbyChatMsg),
    LobbyChatUpdate(LobbyChatUpdate),
    LobbyCreated(LobbyCreated),
    LobbyDataUpdate(LobbyDataUpdate),
    LobbyEnter(LobbyEnter),
    MicroTxnAuthorizationResponse(MicroTxnAuthorizationResponse),
    NetConnectionStatusChanged(NetConnectionStatusChanged),
    NetworkingMessagesSessionFailed(NetworkingMessagesSessionFailed),
    NetworkingMessagesSessionRequest(NetworkingMessagesSessionRequest),
    P2PSessionConnectFail(P2PSessionConnectFail),
    P2PSessionRequest(P2PSessionRequest),
    PersonaStateChange(PersonaStateChange),
    RelayNetworkStatusCallback(RelayNetworkStatusCallback),
    RemotePlayConnected(RemotePlayConnected),
    RemotePlayDisconnected(RemotePlayDisconnected),
    ScreenshotRequested(ScreenshotRequested),
    ScreenshotReady(ScreenshotReady),
    SteamServerConnectFailure(SteamServerConnectFailure),
    SteamServersConnected(SteamServersConnected),
    SteamServersDisconnected(SteamServersDisconnected),
    TicketForWebApiResponse(TicketForWebApiResponse),
    UserAchievementStored(UserAchievementStored),
    UserAchievementIconFetched(UserAchievementIconFetched),
    UserStatsReceived(UserStatsReceived),
    UserStatsStored(UserStatsStored),
    ValidateAuthTicketResponse(ValidateAuthTicketResponse),
    GSClientApprove(GSClientApprove),
    GSClientDeny(GSClientDeny),
    GSClientKick(GSClientKick),
    GSClientGroupStatus(GSClientGroupStatus),
}

impl CallbackResult {
    pub unsafe fn from_raw(discriminator: i32, data: *mut c_void) -> Option<Self> {
        Some(match discriminator {
            NetConnectionStatusChanged::ID => {
                Self::NetConnectionStatusChanged(NetConnectionStatusChanged::from_raw(data))
            }
            AuthSessionTicketResponse::ID => {
                Self::AuthSessionTicketResponse(AuthSessionTicketResponse::from_raw(data))
            }
            DownloadItemResult::ID => Self::DownloadItemResult(DownloadItemResult::from_raw(data)),
            FloatingGamepadTextInputDismissed::ID => Self::FloatingGamepadTextInputDismissed(
                FloatingGamepadTextInputDismissed::from_raw(data),
            ),
            GameLobbyJoinRequested::ID => {
                Self::GameLobbyJoinRequested(GameLobbyJoinRequested::from_raw(data))
            }
            GameOverlayActivated::ID => {
                Self::GameOverlayActivated(GameOverlayActivated::from_raw(data))
            }
            GamepadTextInputDismissed::ID => {
                Self::GamepadTextInputDismissed(GamepadTextInputDismissed::from_raw(data))
            }
            GameRichPresenceJoinRequested::ID => {
                Self::GameRichPresenceJoinRequested(GameRichPresenceJoinRequested::from_raw(data))
            }
            LobbyChatMsg::ID => Self::LobbyChatMsg(LobbyChatMsg::from_raw(data)),
            LobbyDataUpdate::ID => Self::LobbyDataUpdate(LobbyDataUpdate::from_raw(data)),
            MicroTxnAuthorizationResponse::ID => {
                Self::MicroTxnAuthorizationResponse(MicroTxnAuthorizationResponse::from_raw(data))
            }
            P2PSessionConnectFail::ID => {
                Self::P2PSessionConnectFail(P2PSessionConnectFail::from_raw(data))
            }
            P2PSessionRequest::ID => Self::P2PSessionRequest(P2PSessionRequest::from_raw(data)),
            PersonaStateChange::ID => Self::PersonaStateChange(PersonaStateChange::from_raw(data)),
            RemotePlayConnected::ID => {
                Self::RemotePlayConnected(RemotePlayConnected::from_raw(data))
            }
            RemotePlayDisconnected::ID => {
                Self::RemotePlayDisconnected(RemotePlayDisconnected::from_raw(data))
            }
            SteamServerConnectFailure::ID => {
                Self::SteamServerConnectFailure(SteamServerConnectFailure::from_raw(data))
            }
            SteamServersConnected::ID => {
                Self::SteamServersConnected(SteamServersConnected::from_raw(data))
            }
            SteamServersDisconnected::ID => {
                Self::SteamServersDisconnected(SteamServersDisconnected::from_raw(data))
            }
            TicketForWebApiResponse::ID => {
                Self::TicketForWebApiResponse(TicketForWebApiResponse::from_raw(data))
            }
            UserAchievementStored::ID => {
                Self::UserAchievementStored(UserAchievementStored::from_raw(data))
            }
            UserStatsReceived::ID => Self::UserStatsReceived(UserStatsReceived::from_raw(data)),
            UserStatsStored::ID => Self::UserStatsStored(UserStatsStored::from_raw(data)),
            ValidateAuthTicketResponse::ID => {
                Self::ValidateAuthTicketResponse(ValidateAuthTicketResponse::from_raw(data))
            },
            GSClientApprove::ID => Self::GSClientApprove(GSClientApprove::from_raw(data)),
            GSClientDeny::ID => Self::GSClientDeny(GSClientDeny::from_raw(data)),
            GSClientKick::ID => Self::GSClientKick(GSClientKick::from_raw(data)),
            GSClientGroupStatus::ID => {
                Self::GSClientGroupStatus(GSClientGroupStatus::from_raw(data))
            }
            _ => return None,
        })
    }
}

pub unsafe trait Callback {
    const ID: i32;
    unsafe fn from_raw(raw: *mut c_void) -> Self;
}

/// A handle that can be used to remove a callback
/// at a later point.
///
/// Removes the callback from the Steam API context when dropped.
pub struct CallbackHandle {
    id: i32,
    inner: Weak<Inner>,
}

impl Drop for CallbackHandle {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.upgrade() {
            match inner.callbacks.callbacks.lock() {
                Ok(mut cb) => {
                    cb.remove(&self.id);
                }
                Err(err) => {
                    eprintln!("error while dropping callback: {:?}", err);
                }
            }
        }
    }
}

pub(crate) unsafe fn register_callback<C, F>(inner: &Arc<Inner>, mut f: F) -> CallbackHandle
where
    C: Callback,
    F: FnMut(C) + Send + 'static,
{
    {
        inner.callbacks.callbacks.lock().unwrap().insert(
            C::ID,
            Box::new(move |param| {
                let param = C::from_raw(param);
                f(param)
            }),
        );
    }
    CallbackHandle {
        id: C::ID,
        inner: Arc::downgrade(inner),
    }
}

pub(crate) unsafe fn register_call_result<C, F>(
    inner: &Arc<Inner>,
    api_call: sys::SteamAPICall_t,
    f: F,
) where
    F: for<'a> FnOnce(&'a C, bool) + 'static + Send,
{
    inner.callbacks.call_results.lock().unwrap().insert(
        api_call,
        Box::new(move |param, failed| {
            let value = param.cast::<C>().read_unaligned();
            f(&value, failed)
        }),
    );
}
