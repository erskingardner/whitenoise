use crate::database::Database;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::str::from_utf8;

const SETTINGS_KEY: &str = "app_settings";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub dark_theme: bool,
    pub current_identity: Option<String>,
}

impl AppSettings {
    pub fn default() -> Self {
        Self {
            dark_theme: true,
            current_identity: None,
        }
    }

    pub fn from_database(database: &Database) -> Result<Self> {
        let results = database.get(SETTINGS_KEY)?;
        match results {
            Some(results) => {
                let settings_str = from_utf8(&results)?;
                Ok(serde_json::from_str(settings_str)?)
            }
            None => Ok(AppSettings::default()),
        }
    }

    pub fn save(&self, database: &Database) -> Result<()> {
        // TODO: Handle errors better and return them to the UI layer
        let json = serde_json::to_string(self)?;
        database.insert(SETTINGS_KEY, json.as_str())?;
        Ok(())
    }
}
