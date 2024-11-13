use crate::credentials::Credentials;
use anyhow::{self, Context};
use chrono::Utc;
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    path::PathBuf,
};

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
                Credentials::read_from_path(&path)
            })
            .collect()
    }

    pub fn remove(&self, user_storage_name: &str) -> anyhow::Result<()> {
        self.base_path
            .read_dir()
            .context("Failed to read from credentials directory")?
            .find(|entry| {
                let path = entry.as_ref().map_or_else(|_| PathBuf::new(), |e| e.path());
                Credentials::read_from_path(&path)
                    .ok()
                    .map_or(false, |c| c.user_storage_name == user_storage_name)
            })
            .context("Credentials not found")?
            .and_then(|path| std::fs::remove_file(&path.path()))
            .context("Failed to remove credentials file")?;
        Ok(())
    }

    pub fn add(&self, creds: Credentials) -> anyhow::Result<()> {
        if !self.base_path.exists() {
            std::fs::create_dir_all(&self.base_path)
                .context("Failed to create credentials directory")?;
        }

        if creds.user_storage_name.is_empty() {
            anyhow::bail!("User storage name cannot be empty");
        }

        /* check if credentials with the same name do not exist already */
        self.get_all().and_then(|existing_creds| {
            if existing_creds
                .iter()
                .any(|c| c.user_storage_name == creds.user_storage_name)
            {
                anyhow::bail!("Credentials with the same name already exist");
            }

            let mut hasher = DefaultHasher::new();
            creds.hash(&mut hasher);

            let name = format!(
                "{}_{:0x}.toml",
                Utc::now().format("%Y-%m-%d_%H-%M-%S").to_string(),
                hasher.finish()
            );
            let path = self.base_path.join(name);
            creds.write_to_path(&path)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_credentials_manager() {
        let temp_dir = tempdir().unwrap();
        let creds_dir = temp_dir.path().join("creds");
        let creds_manager = CredentialsManager::new(creds_dir.clone());

        let creds = Credentials::new_r2(
            "test".to_string(),
            "storage_name".to_string(),
            "account_id".to_string(),
            "access_key".to_string(),
            "secret_key".to_string(),
        );

        creds_manager.add(creds.clone()).unwrap();
        let all_creds = creds_manager.get_all().unwrap();
        assert_eq!(all_creds.len(), 1);
        assert_eq!(all_creds[0], creds);

        let creds2 = Credentials::new_r2(
            "test2".to_string(),
            "storage_name".to_string(),
            "account_id".to_string(),
            "access_key".to_string(),
            "secret_key".to_string(),
        );

        creds_manager.add(creds2.clone()).unwrap();
        let all_creds = creds_manager.get_all().unwrap();
        assert_eq!(all_creds.len(), 2);
        assert_eq!(all_creds.contains(&creds), true);
        assert_eq!(all_creds.contains(&creds2), true);

        creds_manager.remove("test").unwrap();
        let all_creds = creds_manager.get_all().unwrap();
        assert_eq!(all_creds.len(), 1);
        assert_eq!(all_creds[0].user_storage_name, "test2");

        creds_manager.remove("test2").unwrap();
        let all_creds = creds_manager.get_all().unwrap();
        assert_eq!(all_creds.len(), 0);
    }
}
