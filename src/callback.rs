use super::*;

use libc::{ c_void };
use sys;

use std::mem;
use std::sync::{ Arc, Weak };

pub unsafe trait Callback {
    const ID: i32;
    const SIZE: i32;
    unsafe fn from_raw(raw: *mut c_void) -> Self;
}

pub(crate) unsafe fn register_callback<C, F, Manager>(inner: &Arc<Inner<Manager>>, f: F, game_server: bool)
    where C: Callback,
          F: FnMut(C) + Send + Sync + 'static
{
    unsafe extern "C" fn run<C, F>(_: *mut c_void, userdata: *mut c_void, param: *mut c_void)
            where C: Callback,
                  F: FnMut(C) + Send + Sync + 'static
    {
        let func: &mut F = &mut *(userdata as *mut F);
        let param = C::from_raw(param);
        func(param);
    }

    unsafe extern "C" fn run_extra<C, F>(cb: *mut c_void, userdata: *mut c_void, param: *mut c_void, _: bool, _: sys::SteamAPICall)
        where C: Callback,
              F: FnMut(C) + Send + Sync + 'static
    {
        run::<C, F>(cb, userdata, param);
    }

    unsafe extern "C" fn dealloc<C, F>(cb: *mut c_void, userdata: *mut c_void)
        where C: Callback,
              F: FnMut(C) + Send + Sync + 'static
    {
        sys::SteamAPI_UnregisterCallback(cb);

        let func: Box<F> = Box::from_raw(userdata as _);
        drop(func);
    }

    let data = sys::CallbackData {
        param_size: C::SIZE as _,
        userdata: Box::into_raw(Box::new(f)) as _,
        run: run::<C, F> as _,
        run_extra: run_extra::<C, F> as _,
        dealloc: dealloc::<C, F> as _,
    };

    let flags = if game_server { 0x02 } else { 0x00 };
    let ptr = sys::create_rust_callback(flags, C::ID as _, data);

    sys::SteamAPI_RegisterCallback(ptr, C::ID as _);

    let mut cbs = inner.callbacks.lock().unwrap();
    cbs.callbacks.push(ptr);
}

pub(crate) unsafe fn register_call_result<C, F, Manager>(inner: &Arc<Inner<Manager>>, api_call: sys::SteamAPICall, callback_id: i32, f: F)
    where F: for <'a> FnMut(&'a C, bool) + 'static + Send + Sync
{
    struct CallData<F, Manager> {
        func: F,
        api_call: sys::SteamAPICall,
        inner: Weak<Inner<Manager>>,
    }

    unsafe extern "C" fn run<C, F, Manager>(cb: *mut c_void, userdata: *mut c_void, param: *mut c_void)
        where F: for<'a> FnMut(&'a C, bool) + Send + Sync + 'static
    {
        let data: &mut CallData<F, Manager> = &mut *(userdata as *mut CallData<F, Manager>);
        (data.func)(&*(param as *const _), false);
        
        sys::delete_rust_callback(cb);
    }

    unsafe extern "C" fn run_extra<C, F, Manager>(cb: *mut c_void, userdata: *mut c_void, param: *mut c_void, io_error: bool, api_call: sys::SteamAPICall)
        where F: for<'a> FnMut(&'a C, bool) + Send + Sync + 'static
    {
        let data: &mut CallData<F, Manager> = &mut *(userdata as *mut CallData<F, Manager>);

        if api_call == data.api_call {
            (data.func)(&*(param as *const _), io_error);
            sys::delete_rust_callback(cb);
        }
    }

    unsafe extern "C" fn dealloc<C, F, Manager>(cb: *mut c_void, userdata: *mut c_void)
        where F: for <'a> FnMut(&'a C, bool) + Send + Sync + 'static
    {
        let data: Box<CallData<F, Manager>> = Box::from_raw(userdata as _);

        if let Some(inner) = data.inner.upgrade() {
            sys::SteamAPI_UnregisterCallResult(cb, data.api_call);

            let mut cbs = inner.callbacks.lock().unwrap();
            cbs.call_results.remove(&data.api_call);
        }

        drop(data);
    }

    let userdata = CallData {
        func: f,
        api_call: api_call,
        inner: Arc::downgrade(&inner),
    };

    let data = sys::CallbackData {
        param_size: mem::size_of::<C> as _,
        userdata: Box::into_raw(Box::new(userdata)) as _,
        run: run::<C, F, Manager>,
        run_extra: run_extra::<C, F, Manager>,
        dealloc: dealloc::<C, F, Manager>,
    };

    let ptr = sys::create_rust_callback(0x00, callback_id, data);

    sys::SteamAPI_RegisterCallResult(ptr, api_call);

    let mut cbs = inner.callbacks.lock().unwrap();
    cbs.call_results.insert(api_call, ptr);
}