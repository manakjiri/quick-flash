use crate::utils;
use anyhow::{self, Context};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub enum StorageType {
    R2,
}

#[derive(Serialize, Deserialize)]
pub struct Credentials {
    pub user_storage_name: String,
    pub storage_type: StorageType,
    pub storage_name: String,
    pub storage_account_id: String,
    pub storage_access_key: String,
    pub storage_secret_key: String,
    pub timestamp: i64,
}

impl Credentials {
    pub fn new_r2(
        user_storage_name: String,
        storage_name: String,
        storage_account_id: String,
        storage_access_key: String,
        storage_secret_key: String,
    ) -> Self {
        Self {
            user_storage_name,
            storage_type: StorageType::R2,
            storage_name,
            storage_account_id,
            storage_access_key,
            storage_secret_key,
            timestamp: Utc::now().timestamp(),
        }
    }

    pub fn read_from_path(path: &Path) -> anyhow::Result<Self> {
        let contents = fs::read_to_string(path).context(format!(
            "Failed to read credentials file {}",
            path.display()
        ))?;
        let credentials: Credentials = toml::from_str(&contents).context(format!(
            "Failed to parse credentials file {}",
            path.display()
        ))?;
        Ok(credentials)
    }

    pub fn write_to_path(&self, path: &Path) -> anyhow::Result<()> {
        let contents = toml::to_string(self)?;
        fs::write(path, contents)?;
        Ok(())
    }
}

pub fn get_credentials_from_command_line() -> anyhow::Result<Credentials> {
    eprintln!("Input credentials for the R2 bucket below:");

    eprint!("Bucket Name: ");
    let storage_name = utils::read_line()?;
    eprint!("Bucket Account ID: ");
    let storage_account_id = utils::read_line()?;
    eprint!("Bucket Access Key: ");
    let storage_access_key = utils::read_line()?;
    eprint!("Bucket Secret Key: ");
    let storage_secret_key = utils::read_line()?;
    eprint!(
        "Optionally, name the storage for future reference [{}]: ",
        &storage_name
    );
    let user_storage_name = utils::read_line().unwrap_or(storage_name.clone());

    let creds = Credentials::new_r2(
        user_storage_name,
        storage_name,
        storage_account_id,
        storage_access_key,
        storage_secret_key,
    );
    /* eprintln!("Saving credentials to {}...", path.display());
    write_credentials(&path, &creds)?; */
    Ok(creds)
}
