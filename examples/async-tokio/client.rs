use gipc::connection::AsyncConnection;
use crate::NAME;

pub async fn main() {
    // The client code is virtually the same as the synchronous equivalent.
    // This is to make everything far simpler for the
    let mut connection = AsyncConnection::connect_to_socket(NAME, false).await.expect("Connection worked");

    let greeting: String = connection.receive().await.expect("Couldn't receive greeting from server!");
    if greeting == "Hello, client!" {
        println!("[client] Yay, the server greeted me! I'll greet them back.");
        connection.send(&"Hello, server!").await.expect("Couldn't send greeting to server!");
    }
}