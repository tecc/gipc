use gipc::connection::{AsyncListener, AsyncConnection};
use crate::NAME;

// This is the code that handles the connections to the listener
async fn handle_connection(mut connection: AsyncConnection) {
    // This code is effectively the same as the synchronous example
    connection.send(&"Hello, client!").await.expect("Couldn't send greeting!");
    // And receive messages like so
    let greeting: String = connection.receive().await.expect("Couldn't accept greeting!");
    if greeting == "Hello, server!" {
        println!("[handler] Yay, the client greeted me!");
    }
    // Asynchronous connections are also closed automatically when they are dropped
}

pub async fn main() {
    // Now, here's where gipc really shines - asynchronous listening is very
    println!("[listener] Listening to socket {}", NAME);

    // You begin by setting up a listener like so, much like the synchronous example
    let mut listener = AsyncListener::listen_as_socket(NAME, false).expect("Couldn't listen! That's sad.");

    // However, here things change!
    let mut handles = vec![]; // to allow for joining the handles later on
    while let Ok(connection) = listener.accept().await {
        // This is the good thing about the async support listener-side:
        // Using this, you can spawn the handler and then continue to accept connections,
        // allowing you to handle multiple connections concurrently
        handles.push(tokio::spawn(handle_connection(connection)));
        // This break is here to let the example exit - in reality, you should only break
        // when the listener should be closed
        break
    }
    println!("[listener] Welp, seems there are no more connections for me!")
}