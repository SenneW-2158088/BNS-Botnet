use std::path::PathBuf;

use tempfile::TempPath;
use uuid::Uuid;

pub struct State {
    pub name: String,
    pub enabled: bool,
    pub payload: Option<TempPath>,
    pub child: Option<std::process::Child>,
    pub owner: Option<nostr_sdk::PublicKey>,
}

impl Default for State {
    fn default() -> Self {
        State {
            name: format!("bot-{}", Uuid::new_v4()),
            enabled: false,
            payload: None,
            child: None,
            owner: None,
        }
    }
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"Name: {} | Status: {}"#,
            self.name,
            if self.enabled {
                "Enabled ✅"
            } else {
                "Disabled ❌"
            },
        )
    }
}
