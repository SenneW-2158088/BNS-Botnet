use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    time::Duration,
};

use nostr_sdk::prelude::*;

const RELAYS: [&str; 1] = ["wss://relay.primal.net"];

const RELAY: &'static str = "wss://jingle.nostrver.se";

pub async fn hello() -> Result<()> {
    println!("In hello");
    let keys: Keys =
        Keys::parse("nsec1ne7jjumjazcm8rw22sk63w8djdwp2dy0aa26agemxkzhx4gtcy7q4v9tls").unwrap();

    // let keys: Keys = Keys::generate();

    eprintln!("private key: {:?}", keys.secret_key().display_secret());
    eprintln!(
        "private key (nsec): {}",
        keys.secret_key().to_bech32().unwrap()
    );

    let connection: Connection = Connection::new();
    let opts = Options::new().connection(connection);

    let client = Client::builder().signer(keys.clone()).opts(opts).build();

    for relay in RELAYS {
        client.add_relay(relay).await?;
        client.connect_relay(relay).await?;
        println!("Connected to relay: {}", relay);
    }

    let metadata = Metadata::new()
        .display_name("updated")
        .about("updated ballz")
        .nip05("username@example.com");

    // Update metadata
    client.set_metadata(&metadata).await?;

    // Create event with lower POW difficulty
    let message = "Hey 8=>";
    let wolf = PublicKey::parse("npub1de7mdl5rwlxxwmasaq4z27eyydhrhdp8xasxhv6lza3d75kh7k3sv4p0sa")
        .unwrap();

    let signer = client.signer().await.unwrap();
    let a = signer.clone();

    // let builder = EventBuilder::private_msg(&a, wolf, message, [])
    //     .await
    //     .unwrap();

    let nip = nip04::encrypt(
        keys.secret_key(),
        &wolf,
        "without account creation".as_bytes(),
    )
    .unwrap();
    let tag = Tag::public_key(wolf);
    let event = EventBuilder::new(Kind::EncryptedDirectMessage, nip).tag(tag);

    // Send with timeout
    println!("sending event...");
    let output = client.send_event_builder(event).await.unwrap();

    // TODO: We can filter for events published only by the C2C server

    let subscription = Filter::new().author(keys.public_key()).generic_tags;

    let output = client.subscribe(subscription.clone(), None).await?;
    println!("Subscription ID: {}", output.val);

    let mut stream = client.stream_events(subscription, Duration::MAX).await?;

    println!("Listening for all events");

    // Used to keep the stream alive
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
    // client.disconnect_relay(RELAY).await.unwrap();

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
