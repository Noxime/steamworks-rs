use super::*;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

type Handler = Box<dyn FnOnce(*mut c_void, Option<CallResultFailure>) + Send + 'static>;

const PENDING: u8 = 0;
const CANCELLED: u8 = 1;
const DISPATCHING: u8 = 2;
const COMPLETED: u8 = 3;

/// Outcome of cancelling delivery of an asynchronous Steam result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CancelOutcome {
    Cancelled,
    AlreadyCancelled,
    AlreadyDispatching,
    Completed,
}

struct Delivery {
    phase: AtomicU8,
    native_pending: AtomicBool,
    handler: Mutex<Option<Handler>>,
}

impl Delivery {
    fn cancel(&self) -> CancelOutcome {
        match self
            .phase
            .compare_exchange(PENDING, CANCELLED, Ordering::AcqRel, Ordering::Acquire)
        {
            Ok(_) => {
                let handler = self.handler.lock().unwrap().take();
                // Captured values can themselves cancel another request.
                drop(handler);
                CancelOutcome::Cancelled
            }
            Err(CANCELLED) => CancelOutcome::AlreadyCancelled,
            Err(DISPATCHING) => CancelOutcome::AlreadyDispatching,
            Err(_) => CancelOutcome::Completed,
        }
    }

    fn take_handler(&self) -> Option<Handler> {
        if self
            .phase
            .compare_exchange(PENDING, DISPATCHING, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return None;
        }
        self.handler.lock().unwrap().take()
    }
}

/// Owns delivery of one asynchronous result, not the native Steam operation.
///
/// Keep this handle alive to receive the callback. Drop/cancel prevents delivery
/// that has not started. An already executing callback is not interrupted.
/// Native resources can remain pending until completion or SDK shutdown.
#[must_use = "Keep the handle alive to receive the result"]
pub struct CallResultHandle {
    delivery: Weak<Delivery>,
    cancel_on_drop: bool,
}

impl CallResultHandle {
    /// Stops pending user delivery. Does not cancel or roll back a Steam operation.
    pub fn cancel(&self) -> CancelOutcome {
        self.delivery
            .upgrade()
            .map_or(CancelOutcome::Completed, |d| d.cancel())
    }

    /// Whether user delivery has neither started nor been cancelled.
    pub fn is_delivery_pending(&self) -> bool {
        self.delivery
            .upgrade()
            .is_some_and(|d| d.phase.load(Ordering::Acquire) == PENDING)
    }

    /// Whether the native result still needs consuming or shutdown cleanup.
    ///
    /// Can remain true after `cancel`; include this work in in-flight budgets.
    pub fn is_native_pending(&self) -> bool {
        self.delivery
            .upgrade()
            .is_some_and(|d| d.native_pending.load(Ordering::Acquire))
    }

    pub(crate) fn detach(mut self) {
        self.cancel_on_drop = false;
    }
}

impl Drop for CallResultHandle {
    fn drop(&mut self) {
        if self.cancel_on_drop {
            self.cancel();
        }
    }
}

impl fmt::Debug for CallResultHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CallResultHandle")
            .field("delivery_pending", &self.is_delivery_pending())
            .field("native_pending", &self.is_native_pending())
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CallResultFailure {
    Transport,
    Retrieval,
    InvalidPayload,
}

/// The registry owns native cleanup independently from cancellable user code.
pub(crate) struct CallResultEntry {
    pub(crate) size: usize,
    pub(crate) callback_id: Option<i32>,
    delivery: Arc<Delivery>,
    cleanup: Option<Box<dyn FnOnce() + Send>>,
}

impl CallResultEntry {
    pub(crate) fn new(
        size: usize,
        callback_id: Option<i32>,
        handler: impl FnOnce(*mut c_void, Option<CallResultFailure>) + Send + 'static,
        cleanup: Option<Box<dyn FnOnce() + Send>>,
    ) -> (Self, CallResultHandle) {
        let delivery = Arc::new(Delivery {
            phase: AtomicU8::new(PENDING),
            native_pending: AtomicBool::new(true),
            handler: Mutex::new(Some(Box::new(handler))),
        });
        let handle = CallResultHandle {
            delivery: Arc::downgrade(&delivery),
            cancel_on_drop: true,
        };
        (
            Self {
                size,
                callback_id,
                delivery,
                cleanup,
            },
            handle,
        )
    }

    pub(crate) fn complete(self, data: *mut c_void, failure: Option<CallResultFailure>) {
        if let Some(handler) = self.delivery.take_handler() {
            handler(data, failure);
        }
        // Self::drop runs even if the user's callback unwinds.
    }

    pub(crate) fn matches(&self, id: i32, size: usize) -> bool {
        self.size == size && self.callback_id.map_or(true, |expected| expected == id)
    }
}

impl Drop for CallResultEntry {
    fn drop(&mut self) {
        self.delivery.cancel();
        let handler = self.delivery.handler.lock().unwrap().take();
        drop(handler);
        if let Some(cleanup) = self.cleanup.take() {
            cleanup();
        }
        self.delivery
            .phase
            .compare_exchange(DISPATCHING, COMPLETED, Ordering::AcqRel, Ordering::Acquire)
            .ok();
        self.delivery.native_pending.store(false, Ordering::Release);
    }
}

pub(crate) fn register_raw_call_result(
    inner: &Inner,
    api_call: sys::SteamAPICall_t,
    size: usize,
    callback_id: Option<i32>,
    handler: impl FnOnce(*mut c_void, Option<CallResultFailure>) + Send + 'static,
    cleanup: Option<Box<dyn FnOnce() + Send>>,
) -> CallResultHandle {
    let (entry, handle) = CallResultEntry::new(size, callback_id, handler, cleanup);
    let replaced = inner
        .callbacks
        .call_results
        .lock()
        .unwrap()
        .insert(api_call, entry);
    drop(replaced);
    handle
}

#[cfg(test)]
pub(crate) mod checked_ugc_delivery_tests {
    use super::*;
    use std::sync::{atomic::AtomicUsize, Barrier};

    pub(crate) fn test_inner(stopped: Arc<AtomicBool>) -> Arc<Inner> {
        Arc::new(Inner {
            manager: Manager::Test(stopped),
            callbacks: Callbacks {
                callbacks: Mutex::new(HashMap::new()),
                call_results: Mutex::new(HashMap::new()),
            },
            networking_sockets_data: Mutex::new(NetworkingSocketsData {
                sockets: HashMap::new(),
                independent_connections: HashMap::new(),
                connection_callback: Weak::new(),
            }),
        })
    }

    #[test]
    fn completing_handler_can_register_next_result_with_registry_unlocked() {
        let inner = test_inner(Arc::new(AtomicBool::new(false)));
        let weak = Arc::downgrade(&inner);
        let called = Arc::new(AtomicBool::new(false));
        let next_called = called.clone();
        let first = register_raw_call_result(
            &inner,
            1,
            0,
            Some(17),
            move |_, _| {
                let inner = weak.upgrade().unwrap();
                // try_lock makes a regression fail instead of hanging this test.
                assert!(inner.callbacks.call_results.try_lock().is_ok());
                register_raw_call_result(
                    &inner,
                    2,
                    0,
                    Some(18),
                    move |_, _| next_called.store(true, Ordering::SeqCst),
                    None,
                )
                .detach();
            },
            None,
        );
        let entry = inner.callbacks.take_call_result(1).unwrap();
        assert!(entry.matches(17, 0));
        assert!(!entry.matches(18, 0));
        assert!(!entry.matches(17, 1));
        entry.complete(std::ptr::null_mut(), None);
        assert!(!first.is_native_pending());
        inner
            .callbacks
            .take_call_result(2)
            .unwrap()
            .complete(std::ptr::null_mut(), None);
        assert!(called.load(Ordering::SeqCst));
        assert!(inner.callbacks.take_call_result(1).is_none());
    }

    #[test]
    fn shutdown_drains_real_inner_before_manager_and_has_no_reference_cycle() {
        let stopped = Arc::new(AtomicBool::new(false));
        let cleaned = Arc::new(AtomicBool::new(false));
        let inner = test_inner(stopped.clone());
        let weak = Arc::downgrade(&inner);
        let cleanup_stopped = stopped.clone();
        let cleanup_done = cleaned.clone();
        let handle = register_raw_call_result(
            &inner,
            1,
            0,
            None,
            |_, _| panic!("shutdown must not deliver"),
            Some(Box::new(move || {
                assert!(!cleanup_stopped.load(Ordering::SeqCst));
                cleanup_done.store(true, Ordering::SeqCst);
            })),
        );
        drop(inner);
        assert!(weak.upgrade().is_none());
        assert!(cleaned.load(Ordering::SeqCst));
        assert!(stopped.load(Ordering::SeqCst));
        assert!(!handle.is_native_pending());
    }

    #[test]
    fn cancelling_drops_captures_outside_delivery_mutex() {
        struct CancelOnDrop(CallResultHandle);
        impl Drop for CancelOnDrop {
            fn drop(&mut self) {
                self.0.cancel();
            }
        }
        let (other, handle) = CallResultEntry::new(0, None, |_, _| panic!("cancelled"), None);
        let captured = CancelOnDrop(handle);
        let (entry, handle) = CallResultEntry::new(0, None, move |_, _| drop(captured), None);
        assert_eq!(handle.cancel(), CancelOutcome::Cancelled);
        assert!(other.delivery.phase.load(Ordering::SeqCst) == CANCELLED);
        drop(entry);
        drop(other);
    }

    #[test]
    fn cancellation_drops_user_capture_but_keeps_native_cleanup() {
        struct OnDrop(Arc<AtomicUsize>);
        impl Drop for OnDrop {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::SeqCst);
            }
        }
        let user = Arc::new(AtomicUsize::new(0));
        let native = Arc::new(AtomicUsize::new(0));
        let capture = OnDrop(user.clone());
        let cleanup = native.clone();
        let (entry, handle) = CallResultEntry::new(
            0,
            None,
            move |_, _| {
                drop(capture);
                panic!("cancelled callback ran")
            },
            Some(Box::new(move || {
                cleanup.fetch_add(1, Ordering::SeqCst);
            })),
        );
        assert_eq!(handle.cancel(), CancelOutcome::Cancelled);
        assert_eq!(handle.cancel(), CancelOutcome::AlreadyCancelled);
        assert_eq!(user.load(Ordering::SeqCst), 1);
        assert!(handle.is_native_pending());
        assert_eq!(native.load(Ordering::SeqCst), 0);
        entry.complete(std::ptr::null_mut(), None);
        assert!(!handle.is_native_pending());
        assert_eq!(native.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn cancellation_cannot_retract_an_executing_callback() {
        let barrier = Arc::new(Barrier::new(2));
        let other = barrier.clone();
        let (entry, handle) = CallResultEntry::new(
            0,
            None,
            move |_, _| {
                other.wait();
                other.wait();
            },
            None,
        );
        let thread = std::thread::spawn(move || entry.complete(std::ptr::null_mut(), None));
        barrier.wait();
        assert_eq!(handle.cancel(), CancelOutcome::AlreadyDispatching);
        barrier.wait();
        thread.join().unwrap();
        assert_eq!(handle.cancel(), CancelOutcome::Completed);
    }

    #[test]
    fn completion_keeps_cleanup_until_after_user_code() {
        let cleaned = Arc::new(AtomicBool::new(false));
        let in_callback = cleaned.clone();
        let in_cleanup = cleaned.clone();
        let (entry, handle) = CallResultEntry::new(
            0,
            None,
            move |_, _| assert!(!in_callback.load(Ordering::SeqCst)),
            Some(Box::new(move || in_cleanup.store(true, Ordering::SeqCst))),
        );
        entry.complete(std::ptr::null_mut(), None);
        assert!(cleaned.load(Ordering::SeqCst));
        assert!(!handle.is_delivery_pending());
    }

    #[test]
    fn callback_unwind_still_cleans_native_resource() {
        let cleaned = Arc::new(AtomicBool::new(false));
        let copy = cleaned.clone();
        let (entry, _handle) = CallResultEntry::new(
            0,
            None,
            |_, _| panic!("test panic"),
            Some(Box::new(move || copy.store(true, Ordering::SeqCst))),
        );
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(
            || entry.complete(std::ptr::null_mut(), None)
        ))
        .is_err());
        assert!(cleaned.load(Ordering::SeqCst));
    }

    #[test]
    fn detach_preserves_legacy_ignored_return_behavior() {
        let called = Arc::new(AtomicBool::new(false));
        let copy = called.clone();
        let (entry, handle) = CallResultEntry::new(
            0,
            None,
            move |_, _| copy.store(true, Ordering::SeqCst),
            None,
        );
        handle.detach();
        entry.complete(std::ptr::null_mut(), None);
        assert!(called.load(Ordering::SeqCst));
    }
}
