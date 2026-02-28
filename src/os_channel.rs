use std::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::atomic::{AtomicU8, Ordering},
};

// One-Shot Channel Impl
const EMPTY: u8 = 0;
const WRITING: u8 = 1;
const READY: u8 = 2;
const READING: u8 = 3;

pub struct Channel<T> {
    message: UnsafeCell<MaybeUninit<T>>,
    // in_use: AtomicBool,
    // ready: AtomicBool,
    state: AtomicU8,
}

unsafe impl<T> Sync for Channel<T> where T: Send {}

impl<T> Channel<T> {
    pub const fn new() -> Self {
        Self {
            message: UnsafeCell::new(MaybeUninit::uninit()),
            // in_use: AtomicBool::new(false),
            // ready: AtomicBool::new(false),
            state: AtomicU8::new(EMPTY),
        }
    }

    pub fn send(&self, message: T) {
        // Relaxed memory ordering is possible here because the total modification order of in_use guarantees
        // there will only be a single swap operation on in_use that will return false, which is the
        // only case in which send will attempt to access the cell

        /* if self.in_use.swap(true, Ordering::Relaxed) {
            panic!("can't send more than one message");
        }
        unsafe { (*self.message.get()).write(message) };
        self.ready.store(true, Ordering::Release); */

        if self
            .state
            .compare_exchange(EMPTY, WRITING, Ordering::Relaxed, Ordering::Relaxed)
            .is_err()
        {
            panic!("can't send more than one message!");
        }
        unsafe { (*self.message.get()).write(message) };
        self.state.store(READY, Ordering::Release);
    }

    // Ordering can now be relaxed because we have an acquire load flag in the receive method
    pub fn is_ready(&self) -> bool {
        self.state.load(Ordering::Relaxed) == READY
    }

    pub fn receive(&self) -> T {
        /* if !self.ready.swap(false, Ordering::Acquire) {
            panic!("no message available!")
        } */

        if self
            .state
            .compare_exchange(READY, READING, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            panic!("no message available!");
        }
        unsafe { (*self.message.get()).assume_init_read() }
    }
}

// An atomic operataion is not needed to check the atomic ready flag, because an object can only be dropped if it
// is fully owned by whichever thread is dropping it, with no outstanding borrows.  This means we can use the AtomicBool::get_mut method,
// which takes an exclusive reference (&mut self), proving atomic access is unnecessary.  The same holds for UnsafeCell, through UnsafeCell::get_mut
impl<T> Drop for Channel<T> {
    fn drop(&mut self) {
        if *self.state.get_mut() == READY {
            unsafe {
                self.message.get_mut().assume_init_drop();
            }
        }
    }
}

// Previous Method Impls
// *************************************************************
// V1 Impl
// *************************************************************
// Safety: Only call this once!
/* pub unsafe fn _send(&self, message: T) {
    (*self.message.get()).write(message);
    self.ready.store(true, Ordering::Release);
}

pub fn _is_ready(self) -> bool {
    self.ready.load(Ordering::Acquire)
}

/// Safety: Only call this once,
/// and only after _is_ready() returns true
pub unsafe fn _receive(&self) -> T {
    (*self.message.get()).assume_init_read()
} */
