use std::io;
use std::thread::sleep;
use std::time::Duration;

mod client;
mod listener;

const NAME: &str = "gipc-example-sync";

// Just some boilerplate to demonstrate the examples
fn main() -> Result<(), io::Error> {
    println!("An example of how gipc works synchronously:");

    // Run the listener
    let listener = std::thread::Builder::new()
        .name("listener thread".to_string())
        .spawn(listener::main)?;
    // Run the client after sleeping for a bit so that we know that the listener is connected
    sleep(Duration::from_secs(1));
    let client = std::thread::Builder::new()
        .name("client thread".to_string())
        .spawn(client::main)?;
    listener.join().expect("Couldn't join listener thread");
    client.join().expect("Couldn't join client thread");

    Ok(())
}
