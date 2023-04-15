use gipc::connection::Connection;
use crate::NAME;

pub fn main() {
    println!("[client] Connecting to socket {}", NAME);
    let mut connection = Connection::connect_to_socket(NAME, false).expect("Connection worked");

    let greeting: String = connection.receive().expect("Couldn't receive greeting from server!");
    if greeting == "Hello, client!" {
        println!("[client] Yay, the server greeted me! I'll greet them back.");
        connection.send(&"Hello, server!").expect("Couldn't send greeting to server!");
    }

    // Connections close automatically when they are dropped
}