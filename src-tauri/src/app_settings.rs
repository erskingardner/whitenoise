use crate::database::Database;
use anyhow::Result;
use log::debug;
use serde::{Deserialize, Serialize};
use std::str::from_utf8;

/// Key used to store and retrieve app settings in the database
const SETTINGS_KEY: &str = "app_settings";

/// Represents the application settings
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    /// Indicates whether dark theme is enabled
    pub dark_theme: bool,
    /// Indicates whether developer mode is enabled
    pub dev_mode: bool,
}

impl AppSettings {
    /// Creates a new `AppSettings` instance with default values
    ///
    /// # Returns
    /// A new `AppSettings` instance with default settings
    pub fn default() -> Self {
        Self {
            dark_theme: true,
            dev_mode: true,
        }
    }

    /// Retrieves app settings from the database
    ///
    /// # Arguments
    /// * `database` - A reference to the `Database` instance
    ///
    /// # Returns
    /// * `Ok(AppSettings)` - The retrieved app settings
    /// * `Err` - If there was an error retrieving or parsing the settings
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

    /// Saves the current app settings to the database
    ///
    /// # Arguments
    /// * `database` - A reference to the `Database` instance
    ///
    /// # Returns
    /// * `Ok(())` - If the settings were successfully saved
    /// * `Err` - If there was an error saving the settings
    #[allow(dead_code)]
    pub fn save(&self, database: &Database) -> Result<()> {
        let json = serde_json::to_string(self)?;
        database.insert(SETTINGS_KEY, json.as_str())?;
        Ok(())
    }

    /// Deletes the current app settings from the database and replaces them with default settings
    ///
    /// # Arguments
    /// * `database` - A reference to the `Database` instance
    ///
    /// # Returns
    /// * `Ok(())` - If the settings were successfully deleted and replaced with defaults
    /// * `Err` - If there was an error during the process
    pub fn delete_data(&self, database: &Database) -> Result<()> {
        debug!(target: "app_settings::delete_data", "Deleting app settings");
        let settings = AppSettings::default();
        settings.save(database)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_settings() {
        let settings = AppSettings::default();
        assert!(settings.dark_theme);
        assert!(settings.dev_mode);
    }

    #[test]
    fn test_save_and_retrieve_settings() -> Result<()> {
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test.db");
        let db = Database::new(&db_path).expect("Couldn't create database for test");

        let mut settings = AppSettings::default();
        settings.dark_theme = false;
        settings.dev_mode = false;

        settings.save(&db)?;

        let retrieved_settings = AppSettings::from_database(&db)?;
        assert!(!retrieved_settings.dark_theme);
        assert!(!retrieved_settings.dev_mode);

        Ok(())
    }

    #[test]
    fn test_from_database_with_no_existing_settings() -> Result<()> {
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test.db");
        let db = Database::new(&db_path).expect("Couldn't create database for test");

        let settings = AppSettings::from_database(&db)?;
        assert!(settings.dark_theme);
        assert!(settings.dev_mode);

        Ok(())
    }
}
