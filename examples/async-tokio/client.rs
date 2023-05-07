use gipc::connection::AsyncConnection;
use crate::NAME;

// The asynchronous client code is virtually the same as the synchronous client code.
// Every method signature is as close to the synchronous API as possible.

pub async fn main() {
    // Much like the synchronous connection, you connect using the `connect_to_socket` function.
    // It has the same parameters - the first one is the name of the socket, and the second is whether
    // the socket is global or not.
    let mut connection = AsyncConnection::connect_to_socket(NAME, false).await.expect("Connection worked");

    // The async and sync variants of the receiving and sending of messages is also similar.
    // With `receive` you can receive messages, like so:
    let greeting: String = connection.receive().await.expect("Couldn't receive greeting from server!");
    if greeting == "Hello, client!" {
        println!("[client] Yay, the server greeted me! I'll greet them back.");
        // And with `send`, you can send messages, like so:
        connection.send(&"Hello, server!").await.expect("Couldn't send greeting to server!");
    }

    // If you wish to contract these methods into one call,
    // you can do it much like the synchronous equivalent: using `send_and_receive`.
    let weather: String = connection.send_and_receive(&"What's the weather like today?").await.expect("Could not ask what the weather was!");
    println!("[client] The weather is apparently {}. That's nice!", weather);
    connection.send(&"That's nice!").await.expect("Could not respond with thoughts on the weather!");

    // Unlike synchronous connections, asynchronous connections must be closed manually.
    // This is because gipc cannot close the connection (which is async)
    // in a synchronous context (the dropping of the connection).
    connection.close().await;
}