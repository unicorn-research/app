use crate::wallet::{WalletError, WalletResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

/// Storage manager for wallet data
#[derive(Debug)]
pub struct StorageManager {
    data_dir: PathBuf,
}

impl StorageManager {
    pub fn new(data_dir: PathBuf) -> WalletResult<Self> {
        std::fs::create_dir_all(&data_dir)
            .map_err(|e| WalletError::Storage(format!("Failed to create data directory: {}", e)))?;

        Ok(Self { data_dir })
    }

    /// Save data to a file
    pub async fn save<T: Serialize>(&self, filename: &str, data: &T) -> WalletResult<()> {
        let file_path = self.data_dir.join(filename);
        let json_data = serde_json::to_string_pretty(data)
            .map_err(|e| WalletError::Storage(format!("Serialization failed: {}", e)))?;

        fs::write(file_path, json_data)
            .await
            .map_err(|e| WalletError::Storage(format!("Failed to write file: {}", e)))?;

        Ok(())
    }

    /// Load data from a file
    pub async fn load<T: for<'de> Deserialize<'de>>(&self, filename: &str) -> WalletResult<T> {
        let file_path = self.data_dir.join(filename);

        if !file_path.exists() {
            return Err(WalletError::Storage(format!(
                "File {} does not exist",
                filename
            )));
        }

        let json_data = fs::read_to_string(file_path)
            .await
            .map_err(|e| WalletError::Storage(format!("Failed to read file: {}", e)))?;

        serde_json::from_str(&json_data)
            .map_err(|e| WalletError::Storage(format!("Deserialization failed: {}", e)))
    }

    /// Check if a file exists
    pub fn exists(&self, filename: &str) -> bool {
        self.data_dir.join(filename).exists()
    }

    /// Delete a file
    pub async fn delete(&self, filename: &str) -> WalletResult<()> {
        let file_path = self.data_dir.join(filename);

        if file_path.exists() {
            fs::remove_file(file_path)
                .await
                .map_err(|e| WalletError::Storage(format!("Failed to delete file: {}", e)))?;
        }

        Ok(())
    }

    /// Get the data directory path
    pub fn data_dir(&self) -> &PathBuf {
        &self.data_dir
    }
}
