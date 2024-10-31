use anyhow::{self, Context};
use etcetera::{self, AppStrategy, AppStrategyArgs};
use probe_rs::probe::{list::Lister, DebugProbeInfo};
use std::{fs, path::PathBuf};

mod credentials;
mod credentials_manager;
mod storage;
mod utils;

pub struct BaseDirs {
    pub config_dir: PathBuf,
    pub firmware_cache_dir: PathBuf,
}

impl BaseDirs {
    pub fn new() -> anyhow::Result<Self> {
        let strategy = etcetera::choose_app_strategy(AppStrategyArgs {
            top_level_domain: "cz".to_string(),
            author: "manakjiri".to_string(),
            app_name: "quick-flash".to_string(),
        })
        .context("Failed to resolve application directories")?;

        let config_dir = strategy.config_dir().join("credentials");
        let firmware_cache_dir = strategy.cache_dir().join("firmware");

        fs::create_dir_all(&config_dir).context("Failed to create config directory")?;
        fs::create_dir_all(&firmware_cache_dir)
            .context("Failed to create firmware cache directory")?;

        Ok(BaseDirs {
            config_dir,
            firmware_cache_dir,
        })
    }

    pub fn clear_firmware_cache(&self) -> anyhow::Result<()> {
        fs::remove_dir_all(&self.firmware_cache_dir).context("Failed to clear cache directory")?;
        fs::create_dir_all(&self.firmware_cache_dir)
            .context("Failed to create firmware cache directory")?;
        Ok(())
    }
}

pub fn get_probes() -> anyhow::Result<Vec<DebugProbeInfo>> {
    let lister = Lister::new();
    let probes = lister.list_all();
    if probes.is_empty() {
        anyhow::bail!("No debug probes found")
    }
    Ok(probes)
}
