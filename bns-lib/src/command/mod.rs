use crate::bot::state::State;
use nostr_sdk::prelude::*;

pub enum Commands {
    HelloWorld(HelloWorldCommand),
    Disable(DisableCommand),
    Enable(EnabledCommand),
}

pub type CommandResult = Result<String>;

impl Commands {
    pub fn parse(command: &str) -> Option<Self> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() || parts[0].len() <= 1 || !parts[0].starts_with('/') {
            return None;
        }

        let command_str = &parts[0][1..];
        match command_str {
            "hello" => Some(Commands::HelloWorld(HelloWorldCommand {})),
            "disable" => Some(Commands::Disable(DisableCommand {})),
            "enable" => Some(Commands::Enable(EnabledCommand {})),
            _ => None,
        }
    }

    pub async fn execute(&self, state: &mut State, session: &Session) -> Result<()> {
        match self {
            Commands::HelloWorld(cmd) => cmd.execute(state, session).await,
            Commands::Disable(cmd) => cmd.execute(state, session).await,
            Commands::Enable(cmd) => cmd.execute(state, session).await,
        }
    }
}

use crate::{CNC_PUB_KEY, session::Session};

pub trait Command {
    fn execute(&self, state: &mut State, session: &Session) -> impl Future<Output = Result<()>>;
}

pub struct HelloWorldCommand {}
pub struct DisableCommand {}
pub struct EnabledCommand {}

impl Command for HelloWorldCommand {
    async fn execute(&self, state: &mut State, session: &Session) -> Result<()> {
        // session.update_about("I'm disabled").await?;
        session
            .send_msg("I'm disabled", PublicKey::parse(CNC_PUB_KEY).unwrap())
            .await?;
        Ok(())
    }
}

impl Command for DisableCommand {
    async fn execute(&self, state: &mut State, session: &Session) -> Result<()> {
        state.enabled = true;
        session.update_metadata(state.to_string().as_str()).await?;
        session
            .send_msg("I'm disabled", PublicKey::parse(CNC_PUB_KEY).unwrap())
            .await?;
        Ok(())
    }
}

impl Command for EnabledCommand {
    async fn execute(&self, state: &mut State, session: &Session) -> Result<()> {
        state.enabled = true;
        session.update_metadata(state.to_string().as_str()).await?;
        session
            .send_msg("I'm enabled", PublicKey::parse(CNC_PUB_KEY).unwrap())
            .await?;
        Ok(())
    }
}
