use channels::ref_channel::Channel;
use std::thread;

fn main() {
    let mut channel = Channel::new();
    thread::scope(|s| {
        let (sender, receiver) = channel.split();

        // Calling split() a second time within the thread scope results in an error, while calling it after the scope is acceptable.
        // Even calling split() before the scope is fine, as long as you stop using the returned Sender and Receiver before the Scope starts
        // let (sender1, receiver2) = channel.split();

        s.spawn(move || {
            sender.send("Hello World!");
        });
        assert_eq!(receiver.receive(), "Hello World!");
    });
}
