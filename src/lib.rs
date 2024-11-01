use anyhow::{self, Context, Ok};
use etcetera::{self, AppStrategy, AppStrategyArgs};
use probe_rs::{
    flashing::{download_file_with_options, DownloadOptions, FlashProgress, Format, ProgressEvent},
    probe::{list::Lister, DebugProbeInfo, Probe},
    Permissions,
};
use std::{fs, path::PathBuf};
use storage::Firmware;

pub mod credentials;
pub mod credentials_manager;
pub mod storage;
mod utils;

pub struct BaseDirs {
    pub creds_dir: PathBuf,
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

        let creds_dir = strategy.config_dir().join("credentials");
        let firmware_cache_dir = strategy.cache_dir().join("firmware");

        fs::create_dir_all(&creds_dir).context("Failed to create config directory")?;
        fs::create_dir_all(&firmware_cache_dir)
            .context("Failed to create firmware cache directory")?;

        Ok(BaseDirs {
            creds_dir,
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

pub fn flash_firmware(
    probe: Probe,
    firmware: Firmware,
    connect_under_reset: bool,
) -> anyhow::Result<()> {
    // Attach to a chip.
    eprintln!("Attaching to target...");
    let mut session = match connect_under_reset {
        true => probe.attach_under_reset(&firmware.chip, Permissions::default()),
        false => probe.attach(&firmware.chip, Permissions::default()),
    }
    .context("Failed to attach probe")?;

    // Download the firmware binary.
    eprintln!(
        "Downloading {}/{} to target chip {}...",
        firmware.name, firmware.version, firmware.chip
    );
    let mut options = DownloadOptions::default();
    options.progress = Some(FlashProgress::new(|e| match e {
        ProgressEvent::StartedErasing => eprintln!("Flash erasing..."),
        ProgressEvent::FinishedErasing => eprintln!("Flash programming..."),
        _ => {}
    }));
    options.verify = true;
    options.do_chip_erase = true;
    download_file_with_options(&mut session, firmware.path, Format::Elf, options)
        .context("Failed to flash firmware")?;

    eprintln!("Resetting target...");
    session.core(0)?.reset()?;

    Ok(())
}
