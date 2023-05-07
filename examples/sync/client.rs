use gipc::connection::Connection;
use crate::NAME;

pub fn main() {
    println!("[client] Connecting to socket {}", NAME);
    // First, we connect to the socket we want to connect to.
    // The first parameter specifies the name of the socket we want to connect to.
    // Internally, gipc resolves the name to some location deterministically.
    // The second parameter here specifies whether or not to resolve the socket globally (i.e. when the listening process exists for the entire system).
    // In our case, it doesn't, so we set that parameter to false.
    let mut connection = Connection::connect_to_socket(NAME, false).expect("Connection should connect properly");

    // Once we've successfully connected, we can use its two main methods: `send` and `receive`.
    // `receive` attempts to receive a message from the other process (in this case, the listener).
    // It can receive any type as long as it implements Serde's `Deserialize` trait.
    let greeting: String = connection.receive().expect("Couldn't receive greeting from server!");
    if greeting == "Hello, client!" {
        println!("[client] Yay, the server greeted me! I'll greet them back.");
        // Conversely, `send` sends a message to the other process.
        // The type that you send must implement Serde's `Serialize` trait.
        connection.send(&"Hello, server!").expect("Couldn't send greeting to server!");
    }

    // However, writing `send` and `receive` like so might get repetitive - therefore, gipc provides
    // a method that does both - `send_and_receive`.
    // It first sends a value (whose type must implement `Serialize`),
    // and then receives a value (whose type must implement `Deserialize`).
    let weather: String = connection.send_and_receive(&"What's the weather like today?").expect("Could not ask what the weather was!");
    println!("[client] The weather is apparently {}. That's nice!", weather);
    connection.send(&"That's nice!").expect("Could not respond with thoughts on the weather!");

    // Connections close automatically when they are dropped,
    // but you can close them manually through the `close` method.
    connection.close();
}