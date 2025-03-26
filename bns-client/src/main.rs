use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    time::Duration,
};

use bns_lib::{
    bot::{Bot, Config},
    session::{Session, SessionProps},
};
use nostr_sdk::prelude::*;

const RELAYS: [&str; 1] = [bns_lib::RELAY];

pub async fn run(seed: Option<String>) -> Result<()> {
    let mut bot = Bot::create(Config {
        name: None,
        display_name: None,
        private_key: None,
        relays: RELAYS.to_vec().iter().map(|&s| s.to_string()).collect(),
        seed,
    });

    bot.run().await
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let seed = if args.len() > 1 {
        Some(args[1].clone())
    } else {
        None
    };

    match run(seed).await {
        Err(e) => {
            println!("Got an error {}", e.to_string())
        }
        _ => println!("Succeeded"),
    }
}
