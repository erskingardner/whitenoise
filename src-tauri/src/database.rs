use crate::app_settings::AppSettings;
use crate::AppState;
use crate::Result;
use sled::{Db, IVec};
use std::{path::PathBuf, sync::Arc};
use tauri::State;

const DB_NAME: &str = "whitenoise_db";

#[derive(Debug)]
pub struct Database {
    db: Arc<Db>,
}
impl Database {
    pub fn new(path: PathBuf) -> Result<Self> {
        let db = sled::open(format!("{}/{}", path.to_string_lossy(), DB_NAME))?;
        Ok(Self { db: Arc::new(db) })
    }

    pub fn insert(&self, key: &str, value: &str) -> Result<Option<IVec>> {
        let result = self.db.insert(key, value.as_bytes())?;
        Ok(result)
    }

    pub fn get(&self, key: &str) -> Result<Option<IVec>> {
        let result = self.db.get(key)?;
        Ok(result)
    }

    // pub fn delete(&self, key: &str) -> Result<Option<IVec>> {
    //     let result = self.db.remove(key)?;
    //     Ok(result)
    // }

    pub fn clear(&self) -> Result<()> {
        self.db.clear()?;
        Ok(())
    }
}

// --- Commands ---

/// 🚨🚨 WARNING!!! This clears the entire database. 🚨🚨
#[tauri::command]
pub fn delete_app_data(state: State<'_, AppState>) {
    let db = state.db.clone();
    db.clear().expect("Couldn't clear database");
    let settings = AppSettings::default();
    settings.save(&db).expect("Couldn't save settings");
}
