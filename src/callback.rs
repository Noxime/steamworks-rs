use super::*;

use crate::sys;

use std::sync::{Arc, Weak};

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
unsafe impl<Manager> Send for CallbackHandle<Manager> {}

impl<Manager> Drop for CallbackHandle<Manager> {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.upgrade() {
            match inner.callbacks.lock() {
                Ok(mut cb) => {
                    cb.callbacks.remove(&self.id);
                }
                Err(err) => {
                    eprintln!("error while dropping callback: {:?}", err);
                }
            }
        }
    }
}

pub(crate) unsafe fn register_callback<C, F, Manager>(
    inner: &Arc<Inner<Manager>>,
    mut f: F,
) -> CallbackHandle<Manager>
where
    C: Callback,
    F: FnMut(C) + Send + 'static,
{
    {
        let mut callbacks = inner.callbacks.lock().unwrap();
        callbacks.callbacks.insert(
            C::ID,
            Box::new(move |param| {
                let param = C::from_raw(param);
                f(param)
            }),
        );
    }
    CallbackHandle {
        id: C::ID,
        inner: Arc::downgrade(&inner),
    }
}

pub(crate) unsafe fn register_call_result<C, F, Manager>(
    inner: &Arc<Inner<Manager>>,
    api_call: sys::SteamAPICall_t,
    _callback_id: i32,
    f: F,
) where
    F: for<'a> FnOnce(&'a C, bool) + 'static + Send,
{
    let mut callbacks = inner.callbacks.lock().unwrap();
    callbacks.call_results.insert(
        api_call,
        Box::new(move |param, failed| f(&*(param as *const C), failed)),
    );
}

macro_rules! callback_in_struct {
    (
        $name:ident;
        $(
            $fn_name:ident($clear_before_call:expr): ( $( $fn_arg_name:ident: $cpp_fn_arg:ty => $rust_fn_arg:ty where $normalize_fn:tt),* )
        ),*
    ) => {
        paste::item! {
            $(
                extern fn [<$name _ $fn_name _virtual>](_self: *mut [<$name _callbacks_real>] $(, $fn_arg_name: $cpp_fn_arg)*) {
                    unsafe {
                        ((*(*_self).rust_callbacks).$fn_name)($($normalize_fn ($fn_arg_name)),*);
                        if ($clear_before_call == true) {
                            [<free_ $name>](_self);
                        }
                    }
                }
            )*
            
            pub struct [<$name _rust_callbacks>] {
                $(
                    pub $fn_name: Box<dyn Fn($($rust_fn_arg),*)>
                ),*
            }
            
            #[repr(C)]
            struct [<$name _callbacks_real>] {
                pub vtable: *mut [<$name _callbacks_virtual>],
                pub rust_callbacks: *mut [<$name _rust_callbacks>],
            }
            
            #[repr(C)]
            struct [<$name _callbacks_virtual>] {
                $(
                    pub $fn_name: extern fn(*mut [<$name _callbacks_real>] $(, $cpp_fn_arg)*)
                ),*
            }
            
            unsafe fn [<create_ $name>](rust_callbacks: [<$name _rust_callbacks>]) -> *mut [<$name _callbacks_real>] {
                let vtable_layout = std::alloc::Layout::new::<[<$name _callbacks_virtual>]>();
                let callbacks_layout = std::alloc::Layout::new::<[<$name _callbacks_real>]>();
                let rustcallbacks_layout = std::alloc::Layout::new::<[<$name _rust_callbacks>]>();
                let mut __callbacks: *mut [<$name _rust_callbacks>] = std::alloc::alloc(rustcallbacks_layout).cast();
                if __callbacks.is_null() {
                    std::alloc::handle_alloc_error(rustcallbacks_layout);
                }
                __callbacks.write(rust_callbacks);
                let mut vtable: *mut [<$name _callbacks_virtual>] = std::alloc::alloc(vtable_layout).cast();
                if vtable.is_null() {
                    std::alloc::handle_alloc_error(vtable_layout);
                }
                {
                    let strct = [<$name _callbacks_virtual>] {
                        $(
                            $fn_name: [<$name _ $fn_name _virtual>]
                        ),*
                    };
                    vtable.write(strct);
                }
                let mut callbacks: *mut [<$name _callbacks_real>] = std::alloc::alloc(callbacks_layout).cast();
                if callbacks.is_null() {
                    std::alloc::handle_alloc_error(callbacks_layout);
                }
                {
                    let strct = [<$name _callbacks_real>] {
                        vtable: vtable,
                        rust_callbacks: __callbacks,
                    };
                    callbacks.write(strct);
                }
                
                callbacks
            }
            
            unsafe fn [<free_ $name>](real: *mut [<$name _callbacks_real>]) {
                let vtable_layout = std::alloc::Layout::new::<[<$name _callbacks_virtual>]>();
                let callbacks_layout = std::alloc::Layout::new::<[<$name _callbacks_real>]>();
                let rustcallbacks_layout = std::alloc::Layout::new::<[<$name _rust_callbacks>]>();
                
                std::alloc::dealloc((*real).rust_callbacks.cast(), rustcallbacks_layout);
                std::alloc::dealloc((*real).vtable.cast(), vtable_layout);
                std::alloc::dealloc(real.cast(), callbacks_layout);
            }
        }
    };
}
pub(crate) use callback_in_struct;
