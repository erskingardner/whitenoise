use anyhow::Result;
use log::debug;
use sled::{Db, IVec};
use std::path::Path;
use std::time::Instant;

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
    pub fn insert<K, V>(&self, key: K, value: V) -> Result<Option<IVec>>
    where
        K: AsRef<[u8]>,
        V: AsRef<[u8]>,
    {
        let result = self.db.insert(key, IVec::from(value.as_ref()))?;
        Ok(result)
    }

    /// Use this when you want a namespaced tree for the data
    pub fn insert_in_tree<T, K, V>(&self, tree: T, key: K, value: V) -> Result<Option<IVec>>
    where
        T: AsRef<[u8]>,
        K: AsRef<[u8]>,
        V: AsRef<[u8]>,
    {
        let tree = self.db.open_tree(tree)?;
        let result = tree.insert(key, IVec::from(value.as_ref()))?;
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

    /// Retrieves a value from a specific tree in the database for a given key.
    ///
    /// This function opens a tree with the specified name and attempts to retrieve
    /// the corresponding value for the given key from that tree.
    ///
    /// # Arguments
    ///
    /// * `tree` - A string slice that holds the name of the tree to open.
    /// * `key` - A string slice that holds the key to look up within the tree.
    ///
    /// # Returns
    ///
    /// Returns a Result containing an Option<IVec>. The Option will be:
    /// - Some(IVec) containing the value if the key exists in the specified tree.
    /// - None if the key does not exist in the specified tree.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The underlying sled database encounters an error while opening the tree.
    /// - The get operation on the tree fails.
    #[allow(dead_code)]
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

    /// Deletes a key-value pair from a specific tree in the database.
    ///
    /// This function opens the specified tree and removes the entry for the given key.
    ///
    /// # Arguments
    ///
    /// * `tree` - The name of the tree to open. Can be any type that can be converted to a byte slice.
    /// * `key` - The key to be deleted from the tree. Can be any type that can be converted to a byte slice.
    ///
    /// # Returns
    ///
    /// Returns a Result containing an Option<IVec>. The Option will be:
    /// - Some(IVec) containing the value of the removed key if it existed in the tree.
    /// - None if the key did not exist in the specified tree.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The underlying sled database encounters an error while opening the tree.
    /// - The remove operation on the tree fails.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of the tree name, must implement AsRef<[u8]>.
    /// * `K` - The type of the key, must implement AsRef<[u8]>.
    #[allow(dead_code)]
    pub fn delete_from_tree<T, K>(&self, tree: T, key: K) -> Result<Option<IVec>>
    where
        T: AsRef<[u8]>,
        K: AsRef<[u8]>,
    {
        let tree = self.db.open_tree(tree)?;
        let result = tree.remove(key)?;
        Ok(result)
    }

    /// Deletes all data from the database.
    ///
    /// This function removes all trees and clears all key-value pairs from the main database.
    ///
    /// # Returns
    ///
    /// Returns a Result indicating success (Ok(())) or an error if any operation fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Retrieving tree names fails
    /// - Dropping any tree fails
    /// - Clearing the main database fails
    pub fn delete_data(&self) -> Result<()> {
        let start = Instant::now();
        debug!(target: "database::delete_data", "Deleting all data");
        let tree_names = self.db.tree_names();
        for tree_name in tree_names {
            let tree_name_string = String::from_utf8(tree_name.to_vec())
                .expect("Couldn't convert tree name to string");
            match tree_name_string.as_str() {
                "__sled__default" => (),
                _ => {
                    debug!(target: "database::delete_data", "Deleting tree: {:#?}", tree_name_string);
                    self.db.drop_tree(tree_name)?;
                }
            }
        }
        self.db.clear()?;
        self.db.flush()?;
        debug!(target: "database::delete_data", "Main database cleared in {:#?}", start.elapsed());
        Ok(())
    }
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
        let db = setup_test_db().expect("Couldn't create database for test");
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
        let db = setup_test_db().expect("Couldn't create database for test");
        let key = "test_key";
        let value = "test_value";

        db.insert(key, value).expect("Failed to insert");
        db.delete(key).expect("Failed to delete");
        let result = db.get(key).expect("Failed to get");
        assert!(result.is_none());
    }

    #[test]
    fn test_delete_data() {
        let db = setup_test_db().expect("Couldn't create database for test");
        let key1 = "test_key1";
        let value1 = "test_value1";
        let key2 = "test_key2";
        let value2 = "test_value2";

        // Insert some test data
        db.insert(key1, value1).expect("Failed to insert");
        db.insert(key2, value2).expect("Failed to insert");

        // Verify data was inserted
        assert!(db.get(key1).unwrap().is_some());
        assert!(db.get(key2).unwrap().is_some());

        // Delete all data
        db.delete_data().expect("Failed to delete data");

        // Verify all data was deleted
        assert!(db.get(key1).unwrap().is_none());
        assert!(db.get(key2).unwrap().is_none());

        // Verify all trees were dropped
        // Verify all trees were dropped
        assert_eq!(db.db.tree_names().len(), 1); // Only the default tree should remain
    }
}
