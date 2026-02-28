use std::{
    cell::UnsafeCell,
    collections::VecDeque,
    mem::MaybeUninit,
    sync::{
        Condvar, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

// Basic channel implementation: VecDeque protected by a Mutex.  VecDeque acts as a queue of data (messages).
// Senders add the message to the back of the queue and recipients pop from the front.  Receive operation is made blocking using a Condvar to notify
// waiting receivers of a new message
pub struct BasicChannel<T> {
    queue: Mutex<VecDeque<T>>,
    item_ready: Condvar,
}

impl<T> BasicChannel<T> {
    pub fn new() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            item_ready: Condvar::new(),
        }
    }

    pub fn send(&self, message: T) {
        self.queue.lock().unwrap().push_back(message);
        self.item_ready.notify_one();
    }

    pub fn receive(&self) -> T {
        let mut b = self.queue.lock().unwrap();
        loop {
            if let Some(message) = b.pop_front() {
                return message;
            }
            b = self.item_ready.wait(b).unwrap();
        }
    }
}

// Downsides of this implementation: even if there are plelnty of messages ready to be received, any send or receive operation will brifly block any other send or receive operation,
// since they all have to lock the same mutex.  If VecDeque::push has to grow the capacity of the VecDeque, all sending and receiving threads will have to wait for that
// one thread to finish the reallocation

// One-Shot Channel Impl
pub struct Channel<T> {
    message: UnsafeCell<MaybeUninit<T>>,
    in_use: AtomicBool,
    ready: AtomicBool,
}

unsafe impl<T> Sync for Channel<T> where T: Send {}

impl<T> Channel<T> {
    pub const fn new() -> Self {
        Self {
            message: UnsafeCell::new(MaybeUninit::uninit()),
            in_use: AtomicBool::new(false),
            ready: AtomicBool::new(false),
        }
    }

    pub fn send(&self, message: T) {
        // Relaxed memory ordering is possible here because the total modification order of in_use guarantees
        // there will only be a single swap operation on in_use that will return false, which is the
        // only case in which send will attempt to access the cell
        if self.in_use.swap(true, Ordering::Relaxed) {
            panic!("can't send more than one message");
        }
        unsafe { (*self.message.get()).write(message) };
        self.ready.store(true, Ordering::Release);
    }

    // Ordering can now be relaxed because we have an acquire load flag in the receive method
    pub fn is_ready(&self) -> bool {
        self.ready.load(Ordering::Relaxed)
    }

    pub fn receive(&self) -> T {
        if !self.ready.swap(false, Ordering::Acquire) {
            panic!("no message available!")
        }
        unsafe { (*self.message.get()).assume_init_read() }
    }
}

// An atomic operataion is not needed to check the atomic ready flag, because an object can only be dropped if it
// is fully owned by whichever thread is dropping it, with no outstanding borrows.  This means we can use the AtomicBool::get_mut method,
// which takes an exclusive reference (&mut self), proving atomic access is unnecessary.  The same holds for UnsafeCell, through UnsafeCell::get_mut
impl<T> Drop for Channel<T> {
    fn drop(&mut self) {
        if *self.ready.get_mut() {
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
