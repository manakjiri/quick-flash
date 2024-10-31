use anyhow::{self, Context};
use chrono::{Timelike, Utc};
use std::path::PathBuf;

use crate::credentials::Credentials;

pub struct CredentialsManager {
    base_path: PathBuf,
}

impl CredentialsManager {
    pub fn new(base_path: PathBuf) -> Self {
        CredentialsManager { base_path }
    }

    pub fn get_all(&self) -> anyhow::Result<Vec<Credentials>> {
        self.base_path
            .read_dir()
            .context("Failed to read from credentials directory")?
            .map(|entry| {
                let path = entry?.path();
                Ok(Credentials::read_from_path(&path)?)
            })
            .collect()
    }

    pub fn add(&self, creds: Credentials) -> anyhow::Result<()> {
        /* check if credentials with the same name do not exist already */
        self.get_all().and_then(|existing_creds| {
            if existing_creds
                .iter()
                .any(|c| c.user_storage_name == creds.user_storage_name)
            {
                anyhow::bail!("Credentials with the same name already exist");
            }

            let name = Utc::now().format("%Y-%m-%d_%H-%M-%S").to_string();
            let path = self.base_path.join(name);
            creds.write_to_path(&path)
        })
    }
}
