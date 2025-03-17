use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    time::Duration,
};

use nostr_sdk::prelude::*;

const RELAYS: [&str; 3] = [
    "wss://jingle.nostrver.se",
    "wss://relay.damus.io",
    "wss://nostr.bitcoiner.social",
];

const RELAY: &'static str = "wss://jingle.nostrver.se";

pub async fn hello() -> Result<()> {
    println!("In hello");
    let keys: Keys = Keys::generate();

    let connection: Connection = Connection::new();
    let opts = Options::new().connection(connection);

    let client = Client::builder().signer(keys.clone()).opts(opts).build();

    for relay in RELAYS {
        client.add_relay(relay).await?;
        client.connect_relay(relay).await?;
    }

    let metadata = Metadata::new()
        .name("zenneh")
        .display_name("zenneh")
        .about("a test message")
        .nip05("username@example.com")
        .lud16("pay@yukikishimoto.com")
        .custom_field("test", "value");

    // Update metadata
    client.set_metadata(&metadata).await?;

    // Create event with lower POW difficulty
    let message = "From rust sdk";
    let builder = EventBuilder::text_note(message);

    // Send with timeout
    println!("sending event...");
    let output = client.send_event_builder(builder).await?;

    println!("Event ID: {}", output.id());
    println!("Sent to: {:?}", output.success);
    println!("Not sent to: {:?}", output.failed);

    let subscription = Filter::new();

    // Subscribe
    let output = client.subscribe(subscription.clone(), None).await?;
    println!("Subscription ID: {}", output.val);

    let mut stream = client
        .stream_events(subscription, Duration::from_secs(10))
        .await?;

    println!("Listening for all events");
    // Use tokio::time to keep the stream alive
    let timeout = tokio::time::sleep(Duration::from_secs(60)); // Listen for 60 seconds
    tokio::pin!(timeout);

    loop {
        tokio::select! {
            Some(event) = stream.next() => {
                println!("Got event {:?}", event);
            }
            _ = &mut timeout => {
                println!("Timeout reached, stopping stream");
                break;
            }
        }
    }

    // Disconnect gracefully
    client.disconnect_relay(RELAY);

    Ok(())
}

#[tokio::main]
async fn main() {
    match hello().await {
        Err(e) => {
            println!("Got an error {}", e.to_string())
        }
        _ => println!("Succeeded"),
    }
}
