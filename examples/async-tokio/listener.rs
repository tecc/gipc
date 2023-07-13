use crate::NAME;
use gipc::connection::{AsyncConnection, AsyncListener};

pub async fn main() {
    // Now, here's where gipc really shines - asynchronous listening allows
    // you to spawn tasks and then immediately return to listening for more connections.
    println!("[listener] Listening to socket {}", NAME);

    // You begin by setting up a listener like so, much like the synchronous example
    //
    let mut listener =
        AsyncListener::listen_as_socket(NAME, false).expect("Couldn't listen! That's sad.");

    // However, here things change!
    // Unlike the synchronous listener, which would need a lot more code to allow for handling
    // multiple connections at once, the asynchronous listener can simply spawn a handler task.
    // This is in general good practice, especially if you need to be able to handle multiple connections.
    // Even if you don't, you should still do this.
    let mut handles = vec![]; // to allow for joining the handles later on
    while let Ok(connection) = listener.accept().await {
        // This is the good thing about the async support listener-side:
        // Using this, you can spawn the handler and then continue to accept connections,
        // allowing you to handle multiple connections concurrently
        handles.push(tokio::spawn(handle_connection(connection)));
        // This break is here to let the example exit - in reality, you should only break
        // when the listener should be closed (e.g. when the application is shutting down).
        break;
    }
    println!("[listener] Welp, seems there are no more connections for me!");

    // This is just for cleanup.
    for handle in handles {
        handle.await.expect("Could not join handle for handler!")
    }

    // Unlike synchronous listeners, asynchronous listeners must be closed manually.
    // This is because gipc cannot close the listener (which is async)
    // in a synchronous context (the dropping of the connection).
    listener.close().await.expect("Could not close listener!");
}

// This is the code that handles the connections to the listener
async fn handle_connection(mut connection: AsyncConnection) {
    // This code is effectively the same as the synchronous example.
    // You send messages using the `send` method,
    connection
        .send(&"Hello, client!")
        .await
        .expect("Couldn't send greeting!");
    // and receive messages using the `receive` method.
    let greeting: String = connection
        .receive()
        .await
        .expect("Couldn't accept greeting!");
    if greeting == "Hello, server!" {
        println!("[handler] Yay, the client greeted me!");
    }

    let weather_question: String = connection
        .receive()
        .await
        .expect("Couldn't receive weather questions!");

    if weather_question == "What's the weather like today?" {
        println!("[handler] Ah, the weather? Last I checked it was sunny.");
        // Once again, you can inline both `send` and `receive` by using `send_and_receive`.
        let thoughts_on_the_weather: String = connection
            .send_and_receive(&"sunny")
            .await
            .expect("Couldn't receive thoughts about the weather");
        if thoughts_on_the_weather != "That's nice!" {
            panic!("Oh no! The client didn't think the weather was nice!");
        }
        println!("[handler] The client thought the weather was nice! Well, let's close this connection now.");
    }

    // Asynchronous connections must be closed manually, for the same reasons as the listener.
    connection.close().await;
}
