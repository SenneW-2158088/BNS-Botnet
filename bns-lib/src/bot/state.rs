use uuid::Uuid;

pub struct State {
    pub name: String,
    pub enabled: bool,
}

impl Default for State {
    fn default() -> Self {
        State {
            name: format!("bot-{}", Uuid::new_v4()),
            enabled: true,
        }
    }
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"# Bot State Overview
-----------------
Name: {}
Status: {}
-----------------"#,
            self.name,
            if self.enabled {
                "Enabled ✅"
            } else {
                "Disabled ❌"
            },
        )
    }
}
