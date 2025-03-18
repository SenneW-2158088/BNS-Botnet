use nostr_sdk::prelude::*;

use crate::{CNC_PUB_KEY, session::Session};

pub trait Command {
    fn new(args: &[&str]) -> Self;
    fn execute(&self, session: &Session) -> impl Future<Output = Result<()>>;
}

pub struct HelloWorldCommand {}

impl Command for HelloWorldCommand {
    fn new(args: &[&str]) -> Self {
        HelloWorldCommand {}
    }
    async fn execute(&self, session: &Session) -> Result<()> {
        session
            .send_msg("Hello World!", PublicKey::parse(CNC_PUB_KEY).unwrap())
            .await?;
        Ok(())
    }
}

pub struct DisableCommand {}

impl Command for DisableCommand {
    fn new(args: &[&str]) -> Self {
        DisableCommand {}
    }
    async fn execute(&self, session: &Session) -> Result<()> {
        // session.update_about("I'm disabled").await?;
        session
            .send_msg("I'm disabled", PublicKey::parse(CNC_PUB_KEY).unwrap())
            .await?;
        Ok(())
    }
}

pub struct EnabledCommand {}

impl Command for EnabledCommand {
    fn new(args: &[&str]) -> Self {
        EnabledCommand {}
    }
    async fn execute(&self, session: &Session) -> Result<()> {
        session.update_about("I'm enabled").await?;
        session
            .send_msg("I'm enabled", PublicKey::parse(CNC_PUB_KEY).unwrap())
            .await?;
        Ok(())
    }
}

pub enum Commands {
    HelloWorld(HelloWorldCommand),
    Disable(DisableCommand),
    Enable(EnabledCommand),
}
impl Commands {
    pub fn parse(command: &str) -> Option<Commands> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        // all commands start with '/' and have at least 1 character
        if parts.is_empty() || parts[0].len() <= 1 || !parts[0].starts_with('/') {
            return None;
        }

        let command_str = &parts[0][1..];
        let args = &parts[1..];

        println!("Parsing command: {}", command_str);
        match command_str {
            "hello_world" => Some(Commands::HelloWorld(HelloWorldCommand::new(args))),
            "disable" => Some(Commands::Disable(DisableCommand::new(args))),
            "enable" => Some(Commands::Enable(EnabledCommand::new(args))),
            _ => None,
        }
    }

    pub async fn execute(&self, session: &Session) -> Result<()> {
        match self {
            Commands::HelloWorld(command) => command.execute(session).await,
            Commands::Disable(command) => command.execute(session).await,
            Commands::Enable(command) => command.execute(session).await,
        }
    }
}
