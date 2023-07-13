use crate::NAME;
use gipc::connection::Listener;

pub fn main() {
    println!("[listener] Listening to socket {}", NAME);
    // You begin by setting up a listener, much like you do a connection (see the client example).
    // The only difference is that you cannot begin sending and receiving messages yet.
    let mut listener =
        Listener::listen_as_socket(NAME, false).expect("Couldn't listen! That's sad.");

    // And then you just accept incoming connections!
    // This is a bad implementation, however - this can only handle one connection at a time!
    // See the async example for a better example on how a listener should work.
    while let Ok(mut connection) = listener.accept() {
        // Once you've accepted the connection, everything works precisely like it does on
        // the client-side - this is because gipc uses the same type to represent a connection
        // from a client to a listener as it does for the listener to a client.

        // You use `send` to send a message to the client.
        connection
            .send(&"Hello, client!")
            .expect("Couldn't send greeting!");
        // And you can receive messages by using the `receive` method.
        let message: String = connection.receive().expect("Couldn't receive greeting!");
        if message == "Hello, server!" {
            println!("[listener] Yay, the client greeted me!");
        }

        let message: String = connection
            .receive()
            .expect("Couldn't receive weather questions!");
        if message == "What's the weather like today?" {
            // Whilst I doubt it'd be used very often (since listeners tend to receive first and then send),
            // you can use `send_and_receive` to merge the sending and receiving of messages.
            let thoughts_on_the_weather: String = connection
                .send_and_receive(&"sunny")
                .expect("Couldn't receive thoughts about the weather");
            if thoughts_on_the_weather != "That's nice!" {
                panic!("Oh no! The client didn't think the weather was nice!");
            }
            println!("[listener] The client thought the weather was nice! Well, let's close this connection now.");

            // This break is here to let the example exit - in reality, you should only break
            // when the listener should be closed (e.g. when the application is shutting down).
            // Otherwise you would just continue the loop.
            break;
        }
        // Connections close automatically when they are dropped,
        // but you can close them manually through the `close` method.
        connection.close();
    }
    println!("[listener] Welp, seems there are no more connections for me!");
    // Similarly to connections, listeners are also automatically closed when they are dropped.
    // You can close them manually too, using the `close` method.
    listener.close().expect("Could not close listener!");
}
