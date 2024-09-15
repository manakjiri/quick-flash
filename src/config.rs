use crate::utils;
use anyhow;
use s3;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize)]
enum StorageType {
    R2,
}

#[derive(Serialize, Deserialize)]
pub struct Credentials {
    storage_type: StorageType,
    storage_name: String,
    storage_account_id: String,
    storage_access_key: String,
    storage_secret_key: String,
}

impl Credentials {
    pub fn new_r2(
        storage_name: String,
        storage_account_id: String,
        storage_access_key: String,
        storage_secret_key: String,
    ) -> Self {
        Self {
            storage_type: StorageType::R2,
            storage_name,
            storage_account_id,
            storage_access_key,
            storage_secret_key,
        }
    }

    pub fn init_bucket(&self) -> Result<Box<s3::Bucket>, s3::error::S3Error> {
        let region = match self.storage_type {
            StorageType::R2 => s3::Region::R2 {
                account_id: self.storage_account_id.clone(),
            },
        };

        let bucket = s3::Bucket::new(
            &self.storage_name,
            region,
            s3::creds::Credentials {
                access_key: Some(self.storage_access_key.clone()),
                secret_key: Some(self.storage_secret_key.clone()),
                security_token: None,
                session_token: None,
                expiration: None,
            },
        )?;
        Ok(bucket)
    }
}

pub fn read_credentials(path: &Path) -> anyhow::Result<Credentials> {
    let contents = fs::read_to_string(path)?;
    let credentials: Credentials = toml::from_str(&contents)?;
    Ok(credentials)
}

pub fn write_credentials(path: &Path, credentials: &Credentials) -> anyhow::Result<()> {
    let contents = toml::to_string(credentials)?;
    fs::write(path, contents)?;
    Ok(())
}

pub fn get_credentials(path: &Path) -> anyhow::Result<Credentials> {
    match read_credentials(&path) {
        Ok(creds) => Ok(creds),
        Err(_) => {
            eprintln!("Input credentials for the R2 bucket below:");

            eprint!("Storage Name: ");
            let storage_name = utils::read_line()?;
            eprint!("Storage Account ID: ");
            let storage_account_id = utils::read_line()?;
            eprint!("Storage Access Key: ");
            let storage_access_key = utils::read_line()?;
            eprint!("Storage Secret Key: ");
            let storage_secret_key = utils::read_line()?;

            let creds = Credentials::new_r2(
                storage_name,
                storage_account_id,
                storage_access_key,
                storage_secret_key,
            );
            eprintln!("Saving credentials to {}...", path.display());
            write_credentials(&path, &creds)?;
            Ok(creds)
        }
    }
}
