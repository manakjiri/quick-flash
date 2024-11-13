use crate::credentials::{Credentials, StorageType};
use anyhow::{self, Context};
use s3;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug)]
pub struct Firmware {
    pub name: String,
    pub version: String,
    pub chip: String,
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize)]
struct Manifest {
    chip: String,
}

pub struct Storage {
    bucket: Box<s3::Bucket>,
}

impl Storage {
    pub fn new(creds: &Credentials) -> anyhow::Result<Self> {
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

    pub fn is_available(&self) -> anyhow::Result<()> {
        match self.bucket.exists() {
            Ok(true) => Ok(()),
            Ok(false) => anyhow::bail!("Bucket does not exist"),
            Err(e) => Err(e.into()),
        }
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
        self.list_common_prefixes("".to_string())
    }

    pub fn list_firmware_versions(&self, firmware_name: &str) -> anyhow::Result<Vec<String>> {
        let mut firmware_name = firmware_name.to_owned();
        firmware_name.push('/');
        Ok(self
            .list_common_prefixes(firmware_name.clone())?
            .iter()
            .map(|p| p.strip_prefix(&firmware_name).unwrap_or(p).to_owned())
            .collect())
    }

    pub fn download_firmware(
        &self,
        name: &str,
        version: &str,
        cache_base: &Path,
    ) -> anyhow::Result<Firmware> {
        let cache_base = cache_base.to_path_buf().join(name).join(version);
        let cache_firmware = cache_base.join("firmware.elf");
        let cache_manifest = cache_base.join("manifest.json");

        if !cache_base.exists() {
            std::fs::create_dir_all(&cache_base)?;
            let bucket_base = format!("{}/{}", name, version);
            let bucket_firmware = format!("{}/firmware.elf", bucket_base);
            let bucket_manifest = format!("{}/manifest.json", bucket_base);

            eprintln!("Downloading firmware to {}...", cache_base.display());
            let firmware = self.bucket.get_object(&bucket_firmware)?;
            std::fs::write(&cache_firmware, firmware.bytes())?;
            let manifest = self.bucket.get_object(&bucket_manifest)?;
            std::fs::write(&cache_manifest, manifest.bytes())?;
        }

        let manifest: Manifest = serde_json::from_str(&std::fs::read_to_string(cache_manifest)?)?;

        Ok(Firmware {
            name: name.to_owned(),
            version: version.to_owned(),
            chip: manifest.chip,
            path: cache_firmware,
        })
    }
}
