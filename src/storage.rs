use crate::config::{Credentials, StorageType};
use anyhow::{self, Context};
use s3;

pub struct Storage {
    bucket: Box<s3::Bucket>,
}

impl Storage {
    pub fn new(creds: &Credentials) -> Result<Self, s3::error::S3Error> {
        let region = match creds.storage_type {
            StorageType::R2 => s3::Region::R2 {
                account_id: creds.storage_account_id.clone(),
            },
        };

        let bucket = s3::Bucket::new(
            &creds.storage_name,
            region,
            s3::creds::Credentials {
                access_key: Some(creds.storage_access_key.clone()),
                secret_key: Some(creds.storage_secret_key.clone()),
                security_token: None,
                session_token: None,
                expiration: None,
            },
        )?;
        Ok(Storage { bucket })
    }

    fn list_common_prefixes(&self, prefix: String) -> anyhow::Result<Vec<String>> {
        Ok(self
            .bucket
            .list(prefix, Some("/".to_string()))?
            .first()
            .cloned()
            .context("No response data received")?
            .common_prefixes
            .context("No common prefixes received")?
            .iter()
            .map(|p| p.prefix.strip_suffix("/").unwrap_or(&p.prefix).to_owned())
            .collect())
    }

    pub fn list_firmwares(&self) -> anyhow::Result<Vec<String>> {
        Ok(self.list_common_prefixes("".to_string())?)
    }

    pub fn list_firmware_versions(&self, firmware_name: &str) -> anyhow::Result<Vec<String>> {
        let mut firmware_name = firmware_name.to_owned();
        firmware_name.push_str("/");
        Ok(self
            .list_common_prefixes(firmware_name.clone())?
            .iter()
            .map(|p| p.strip_prefix(&firmware_name).unwrap_or(&p).to_owned())
            .collect())
    }
}
