use crate::{app_settings::AppSettings, whitenoise::Whitenoise};
use anyhow::Result;
use log::debug;
use sled::{Db, IVec};
use std::path::Path;
use tauri::State;
const DB_NAME: &str = "wdb";

#[derive(Debug)]
pub struct Database {
    pub db: Db,
}
impl Database {
    /// Creates a new Database instance.
    ///
    /// This function opens or creates a sled database at the specified path,
    /// using the constant DB_NAME as the database name.
    ///
    /// # Arguments
    ///
    /// * `path` - A PathBuf representing the directory where the database should be stored.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the Database instance if successful,
    /// or an error if the database couldn't be opened.
    ///
    /// # Errors
    ///
    /// This function will return an error if sled fails to open the database.
    pub fn new(path: &Path) -> Result<Self> {
        debug!(target: "database::new", "Opening database at: {:?}", path.to_string_lossy());
        let db = sled::open(format!("{}/{}", path.to_string_lossy(), DB_NAME))?;
        Ok(Self { db })
    }

    /// Inserts a key-value pair into the database.
    ///
    /// This function takes a string key and a string value, converts the value to bytes,
    /// and inserts the pair into the database.
    ///
    /// # Arguments
    ///
    /// * `key` - A string slice that holds the key.
    /// * `value` - A string slice that holds the value to be inserted.
    ///
    /// # Returns
    ///
    /// Returns a Result containing an Option<IVec>. The Option will be:
    /// - Some(IVec) containing the previous value if the key already existed.
    /// - None if the key did not previously exist.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying sled database
    /// encounters an error during the insert operation.
    pub fn insert(&self, key: &str, value: &str) -> Result<Option<IVec>> {
        let result = self.db.insert(key, value.as_bytes())?;
        Ok(result)
    }

    /// Use this when you want a namespaced tree for the data
    pub fn insert_in_tree(&self, tree: &str, key: &str, value: &str) -> Result<Option<IVec>> {
        let tree = self.db.open_tree(tree)?;
        let result = tree.insert(key, value.as_bytes())?;
        Ok(result)
    }

    /// Retrieves a value from the database for a given key.
    ///
    /// This function takes a string key and attempts to retrieve the corresponding value
    /// from the database.
    ///
    /// # Arguments
    ///
    /// * `key` - A string slice that holds the key to look up.
    ///
    /// # Returns
    ///
    /// Returns a Result containing an Option<IVec>. The Option will be:
    /// - Some(IVec) containing the value if the key exists in the database.
    /// - None if the key does not exist in the database.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying sled database
    /// encounters an error during the get operation.
    pub fn get(&self, key: &str) -> Result<Option<IVec>> {
        let result = self.db.get(key)?;
        Ok(result)
    }

    pub fn get_from_tree(&self, tree: &str, key: &str) -> Result<Option<IVec>> {
        let tree = self.db.open_tree(tree)?;
        let result = tree.get(key)?;
        Ok(result)
    }

    /// Deletes a key-value pair from the database.
    ///
    /// This function removes the entry for the specified key from the database.
    ///
    /// # Arguments
    ///
    /// * `key` - A string slice that holds the key to be deleted.
    ///
    /// # Returns
    ///
    /// Returns a Result containing an Option<IVec>. The Option will be:
    /// - Some(IVec) containing the value of the removed key if it existed.
    /// - None if the key did not exist in the database.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying sled database
    /// encounters an error during the remove operation.
    #[allow(dead_code)]
    pub fn delete(&self, key: &str) -> Result<Option<IVec>> {
        let result = self.db.remove(key)?;
        Ok(result)
    }

    /// Clears all data from the database.
    ///
    /// This function removes all key-value pairs from the database, effectively resetting it
    /// to an empty state.
    ///
    /// # Returns
    ///
    /// Returns a Result<()>. If the operation is successful, it returns Ok(()), otherwise
    /// it returns an error.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying sled database encounters an error
    /// during the clear operation.
    pub fn clear(&self) -> Result<()> {
        self.db.clear()?;
        Ok(())
    }
}

/// --- Commands ---

/// Deletes all application data and resets settings to default.
///
/// This function clears the entire database and then saves default settings.
/// It should be used with caution as it will result in loss of all stored data.
///
/// # Arguments
///
/// * `state` - A State containing the AppState, which includes the database.
///
/// # Panics
///
/// This function will panic if:
/// - It fails to clear the database
/// - It fails to save the default settings
///
/// # Safety
///
/// This is a destructive operation that cannot be undone. Use with extreme caution.
#[tauri::command]
pub fn delete_app_data(state: State<'_, Whitenoise>) {
    let db = &state.wdb;
    db.clear().expect("Couldn't clear database");
    let settings = AppSettings::default();
    settings.save(db).expect("Couldn't save settings");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn setup_test_db() -> Result<Database> {
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test.db");
        Database::new(&db_path)
    }

    #[test]
    fn test_insert_and_get() {
        let db = setup_test_db().unwrap();
        let key = "test_key";
        let value = "test_value";

        db.insert(key, value).expect("Failed to insert");
        let result = db
            .get(key)
            .expect("Failed to get")
            .expect("Value not found");
        assert_eq!(result.as_ref(), value.as_bytes());
    }

    #[test]
    fn test_delete() {
        let db = setup_test_db().unwrap();
        let key = "test_key";
        let value = "test_value";

        db.insert(key, value).expect("Failed to insert");
        db.delete(key).expect("Failed to delete");
        let result = db.get(key).expect("Failed to get");
        assert!(result.is_none());
    }

    #[test]
    fn test_clear() {
        let db = setup_test_db().unwrap();
        let key1 = "test_key1";
        let key2 = "test_key2";
        let value = "test_value";

        db.insert(key1, value).expect("Failed to insert key1");
        db.insert(key2, value).expect("Failed to insert key2");
        db.clear().expect("Failed to clear database");

        let result1 = db.get(key1).expect("Failed to get key1");
        let result2 = db.get(key2).expect("Failed to get key2");
        assert!(result1.is_none());
        assert!(result2.is_none());
    }
}
