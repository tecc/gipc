use gipc::connection::Listener;
use crate::NAME;

pub fn main() {
    println!("[listener] Listening to socket {}", NAME);
    // You begin by setting up a listener like so
    let mut listener = Listener::listen_as_socket(NAME, false).expect("Couldn't listen! That's sad.");

    // And then you just accept incoming connections!
    // This is a bad implementation, however - this can only handle one connection at a time!
    // See the async example for a better example on how a listener should work
    while let Ok(mut connection) = listener.accept() {
        // You can send messages like so
        connection.send(&"Hello, client!").expect("Couldn't send greeting!");
        // And receive messages like so
        let greeting: String = connection.receive().expect("Couldn't accept greeting!");
        if greeting == "Hello, server!" {
            println!("[listener] Yay, the client greeted me!");
            break; // Just breaks so the process exits
        }
    }
    println!("[listener] Welp, seems there are no more connections for me!")
}