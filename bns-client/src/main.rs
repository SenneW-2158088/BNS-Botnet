use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    time::Duration,
};

use bns_lib::{
    bot::{Bot, Config},
    session::{Session, SessionProps},
};
use nostr_sdk::prelude::*;

const RELAYS: [&str; 1] = ["wss://relay.primal.net"];

pub async fn run() -> Result<()> {
    let mut bot = Bot::create(Config {
        name: None,
        display_name: None,
        private_key: None,
        relays: RELAYS.to_vec().iter().map(|&s| s.to_string()).collect(),
    });

    bot.run().await
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
