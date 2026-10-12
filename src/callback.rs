use super::*;
use crate::networking_messages::*;
use crate::networking_types::*;
use crate::networking_utils::*;
use crate::screenshots::*;

use crate::sys;

use std::sync::atomic::Ordering;
use std::sync::{Arc, Weak};

/// A sum type over all possible callback results
#[derive(Debug)]
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
    NewUrlLaunchParameters(NewUrlLaunchParameters),
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
            LobbyChatUpdate::ID => Self::LobbyChatUpdate(LobbyChatUpdate::from_raw(data)),
            LobbyCreated::ID => Self::LobbyCreated(LobbyCreated::from_raw(data)),
            LobbyDataUpdate::ID => Self::LobbyDataUpdate(LobbyDataUpdate::from_raw(data)),
            LobbyEnter::ID => Self::LobbyEnter(LobbyEnter::from_raw(data)),
            MicroTxnAuthorizationResponse::ID => {
                Self::MicroTxnAuthorizationResponse(MicroTxnAuthorizationResponse::from_raw(data))
            }
            NetworkingMessagesSessionFailed::ID => Self::NetworkingMessagesSessionFailed(
                NetworkingMessagesSessionFailed::from_raw(data),
            ),
            NetworkingMessagesSessionRequest::ID => Self::NetworkingMessagesSessionRequest(
                NetworkingMessagesSessionRequest::from_raw(data),
            ),
            P2PSessionConnectFail::ID => {
                Self::P2PSessionConnectFail(P2PSessionConnectFail::from_raw(data))
            }
            P2PSessionRequest::ID => Self::P2PSessionRequest(P2PSessionRequest::from_raw(data)),
            PersonaStateChange::ID => Self::PersonaStateChange(PersonaStateChange::from_raw(data)),
            RelayNetworkStatusCallback::ID => {
                Self::RelayNetworkStatusCallback(RelayNetworkStatusCallback::from_raw(data))
            }
            RemotePlayConnected::ID => {
                Self::RemotePlayConnected(RemotePlayConnected::from_raw(data))
            }
            RemotePlayDisconnected::ID => {
                Self::RemotePlayDisconnected(RemotePlayDisconnected::from_raw(data))
            }
            ScreenshotRequested::ID => {
                Self::ScreenshotRequested(ScreenshotRequested::from_raw(data))
            }
            ScreenshotReady::ID => Self::ScreenshotReady(ScreenshotReady::from_raw(data)),
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
            UserAchievementIconFetched::ID => {
                Self::UserAchievementIconFetched(UserAchievementIconFetched::from_raw(data))
            }
            UserStatsReceived::ID => Self::UserStatsReceived(UserStatsReceived::from_raw(data)),
            UserStatsStored::ID => Self::UserStatsStored(UserStatsStored::from_raw(data)),
            ValidateAuthTicketResponse::ID => {
                Self::ValidateAuthTicketResponse(ValidateAuthTicketResponse::from_raw(data))
            }
            GSClientApprove::ID => Self::GSClientApprove(GSClientApprove::from_raw(data)),
            GSClientDeny::ID => Self::GSClientDeny(GSClientDeny::from_raw(data)),
            GSClientKick::ID => Self::GSClientKick(GSClientKick::from_raw(data)),
            GSClientGroupStatus::ID => {
                Self::GSClientGroupStatus(GSClientGroupStatus::from_raw(data))
            }
            NewUrlLaunchParameters::ID => {
                Self::NewUrlLaunchParameters(NewUrlLaunchParameters::from_raw(data))
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
/// Removes the registration this handle was created for from the Steam API
/// context when dropped. Other registrations of the same callback type are
/// unaffected.
pub struct CallbackHandle {
    id: i32,
    seq: u64,
    inner: Weak<Inner>,
}

impl Drop for CallbackHandle {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.upgrade() {
            match inner.callbacks.callbacks.lock() {
                Ok(mut cbs) => {
                    let mut empty = false;
                    let mut found = false;
                    if let Some(entries) = cbs.get_mut(&self.id) {
                        let before = entries.len();
                        entries.retain(|(seq, _)| *seq != self.seq);
                        found = entries.len() != before;
                        empty = entries.is_empty();
                    }
                    if !found {
                        // The seq is not in the table because its entry is
                        // mid-dispatch (the bucket was taken out whole, or
                        // re-created around it by a re-entrant registration):
                        // record a tombstone so the dispatcher does not put
                        // this registration back.
                        inner
                            .callbacks
                            .pending_removal
                            .lock()
                            .unwrap()
                            .insert(self.seq);
                    }
                    if empty {
                        cbs.remove(&self.id);
                    }
                }
                Err(err) => {
                    eprintln!("error while dropping callback: {:?}", err);
                }
            }
        }
    }
}

macro_rules! impl_callback {
    ($fn_arg_name:ident: $sys_ty:ident => $callback_ty:ident $from_raw_impl:tt) => {
        pastey::item! {
            unsafe impl Callback for $callback_ty {
                const ID: i32 = steamworks_sys::[<$sys_ty _ k_iCallback>] as i32;

                unsafe fn from_raw(raw: *mut c_void) -> Self {
                    let $fn_arg_name = raw.cast::<steamworks_sys::$sys_ty>().read_unaligned();
                    $from_raw_impl
                }
            }
        }
    };
}

pub(crate) unsafe fn register_callback<C, F>(inner: &Arc<Inner>, mut f: F) -> CallbackHandle
where
    C: Callback,
    F: FnMut(C) + Send + 'static,
{
    let seq = inner.callbacks.next_seq.fetch_add(1, Ordering::Relaxed);
    {
        inner
            .callbacks
            .callbacks
            .lock()
            .unwrap()
            .entry(C::ID)
            .or_default()
            .push((
                seq,
                Box::new(move |param| {
                    let param = C::from_raw(param);
                    f(param)
                }),
            ));
    }
    CallbackHandle {
        id: C::ID,
        seq,
        inner: Arc::downgrade(inner),
    }
}

/// Registers a callback whose re-registration replaces the previous
/// registration made through this helper for the same callback type, while
/// leaving registrations made by other callers (e.g. via
/// `Client::register_callback`) untouched. This backs the wrapper APIs whose
/// docs say "calling this function more than once will replace the previous
/// callback".
pub(crate) unsafe fn register_replacing_callback<C, F>(inner: &Arc<Inner>, f: F)
where
    C: Callback,
    F: FnMut(C) + Send + 'static,
{
    let handle = register_callback::<C, F>(inner, f);
    let old = inner
        .callbacks
        .replacing
        .lock()
        .unwrap()
        .insert(C::ID, handle);
    // Drop the replaced handle only after the registry lock is released:
    // dropping it takes the callback table lock.
    drop(old);
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
