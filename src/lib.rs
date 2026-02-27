use std::{
    collections::VecDeque,
    sync::{Condvar, Mutex},
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
