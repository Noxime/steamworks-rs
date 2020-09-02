use super::*;

use libc::{ c_void };
use crate::sys;

use std::sync::{ Arc, Weak };

pub unsafe trait Callback {
    const ID: i32;
    const SIZE: i32;
    unsafe fn from_raw(raw: *mut c_void) -> Self;
}

/// A handle that can be used to remove a callback
/// at a later point.
///
/// Removes the callback when dropped
pub struct CallbackHandle<Manager = ClientManager> {
    id: i32,
    inner: Weak<Inner<Manager>>,
}
unsafe impl <Manager> Send for CallbackHandle<Manager> {}

impl <Manager> Drop for CallbackHandle<Manager> {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.upgrade() {
            let mut cb = inner.callbacks.lock().unwrap();
            cb.callbacks.remove(&self.id);
        }
    }
}

pub(crate) unsafe fn register_callback<C, F, Manager>(inner: &Arc<Inner<Manager>>, mut f: F, _game_server: bool) -> CallbackHandle<Manager>
    where C: Callback,
          F: FnMut(C) + Send + 'static
{
    {
        let mut callbacks = inner.callbacks.lock().unwrap();
        callbacks.callbacks.insert(C::ID, Box::new(move |param| {
            let param = C::from_raw(param);
            f(param)
        }));
    }
    CallbackHandle {
        id: C::ID,
        inner: Arc::downgrade(&inner),
    }
}

pub(crate) unsafe fn register_call_result<C, F, Manager>(inner: &Arc<Inner<Manager>>, api_call: sys::SteamAPICall_t, _callback_id: i32, f: F)
    where F: for <'a> FnOnce(&'a C, bool) + 'static + Send
{
    let mut callbacks = inner.callbacks.lock().unwrap();
    callbacks.call_results.insert(api_call, Box::new(move |param, failed| {
        f(&*(param as *const C), failed)
    }));
}