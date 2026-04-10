use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub accounts: HashMap<String, String>,
    pub active_account: String,
    pub model: String,
    #[serde(rename = "telegram_bot_token")]
    pub telegram_bot_token: Option<String>,
    #[serde(rename = "telegram_chat_id", default)]
    pub telegram_chat_id: Option<String>,
}

impl Config {
    pub fn load(path: &PathBuf) -> Result<Self, String> {
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read config: {}", e))?;

        serde_json::from_str(&content).map_err(|e| format!("Failed to parse config: {}", e))
    }

    pub fn get_active_key(&self) -> Option<&String> {
        self.accounts.get(&self.active_account)
    }

    pub fn get_all_keys(&self) -> Vec<&String> {
        self.accounts.values().collect()
    }

    pub fn switch_to_next_key(&mut self) -> bool {
        let current = self.get_active_key();
        if current.is_none() {
            return false;
        }

        let current_key = current.unwrap();
        let keys: Vec<&String> = self.accounts.values().collect();

        let mut found = false;
        for key in &keys {
            if *key == current_key {
                found = true;
            } else if found {
                self.active_account = self
                    .accounts
                    .iter()
                    .find(|(_, v)| *v == *key)
                    .map(|(k, _)| k.clone())
                    .unwrap_or_default();
                return true;
            }
        }

        if let Some(first) = keys.first() {
            self.active_account = self
                .accounts
                .iter()
                .find(|(_, v)| *v == *first)
                .map(|(k, _)| k.clone())
                .unwrap_or_default();
            return true;
        }

        false
    }
}

pub struct AmorshiFiles {
    pub config: Config,
    pub master: String,
    pub reminders: String,
    pub tools: String,
}

impl AmorshiFiles {
    pub fn load() -> Result<Self, String> {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let base_path = std::path::PathBuf::from(format!("{}/amorshi", home));

        // Fallback to current dir if ~/amorshi doesn't exist
        let base_path = if base_path.exists() {
            base_path
        } else {
            std::env::current_dir()
                .map_err(|e| format!("Failed to get current dir: {}", e))?
                .join("amorshi")
        };

        if !base_path.exists() {
            return Err(format!("amorshi folder not found at {:?}", base_path));
        }

        let config = Config::load(&base_path.join("shit.cfg"))?;
        let master = fs::read_to_string(&base_path.join("master.md"))
            .map_err(|e| format!("Failed to read master.md: {}", e))?;
        let reminders = fs::read_to_string(&base_path.join("reminders.md"))
            .map_err(|e| format!("Failed to read reminders.md: {}", e))?;
        let tools = fs::read_to_string(&base_path.join("tools.md"))
            .map_err(|e| format!("Failed to read tools.md: {}", e))?;

        Ok(Self {
            config,
            master,
            reminders,
            tools,
        })
    }
}
