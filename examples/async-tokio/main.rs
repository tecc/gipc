use std::io;
use std::time::Duration;
use tokio::time::sleep;

mod client;
mod listener;

const NAME: &str = "gipc-example-async-tokio";

#[tokio::main]
async fn main() -> io::Result<()> {
    println!("An example of how gipc works asynchronously (using Tokio):");

    let listener = tokio::spawn(listener::main());
    sleep(Duration::from_secs(1)).await; // once again, sleep to let the listener listen
    let client = tokio::spawn(client::main());
    listener.await?;
    client.await?;
    Ok(())
}