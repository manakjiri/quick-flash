use anyhow::{self, Context};
use clap::{self, Parser};
use etcetera::{self, AppStrategy, AppStrategyArgs};
use probe_rs::flashing::{
    download_file_with_options, DownloadOptions, FlashProgress, Format, ProgressEvent,
};
use probe_rs::probe::list::Lister;
use probe_rs::Permissions;
use std::fs;
use std::process::exit;

mod config;
mod storage;
mod utils;

/// Flash centrally hosted firmware binaries with one command
#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    firmware_name: Option<String>,
    firmware_version: Option<String>,

    /// Lists available firmware names (if given no arguments) or versions of a specific firmware (if given FIRMWARE_NAME)
    #[arg(long, short)]
    list: bool,

    #[arg(long)]
    clear_cache: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let strategy = etcetera::choose_app_strategy(AppStrategyArgs {
        top_level_domain: "cz".to_string(),
        author: "manakjiri".to_string(),
        app_name: "quick-flash".to_string(),
    })?;
    fs::create_dir_all(strategy.config_dir()).context("Failed to create config directory")?;
    let creds_path = strategy.config_dir().join("credentials.toml");
    let cache_base = strategy.cache_dir().join("firmware");
    fs::create_dir_all(&cache_base).context("Failed to create cache directory")?;

    if args.clear_cache {
        eprintln!("Clearing cache directory...");
        fs::remove_dir_all(&cache_base).context("Failed to clear cache directory")?;
        fs::create_dir_all(&cache_base)?;
    }

    let creds = config::get_credentials(&creds_path).context("Failed to read credentials")?;
    let storage = storage::Storage::new(&creds).context("Failed to create storage client")?;

    let firmwares = storage
        .list_firmwares()
        .context("Failed to fetch firmware names from the Bucket")?;

    if firmwares.is_empty() {
        anyhow::bail!("No firmware found in the Bucket");
    }

    /* firmware names list command */
    if args.list && args.firmware_name.is_none() {
        println!(
            "Listing {} available firmware name{}:",
            firmwares.len(),
            firmwares.len().eq(&1).then_some("").unwrap_or("s")
        );
        for name in storage.list_firmwares()? {
            println!("  - {}", name);
        }
        exit(0);
    }

    /* firmware name sanity checks */
    let firmware_name = match args.firmware_name {
        Some(n) => {
            if !firmwares.contains(&n) {
                anyhow::bail!("Firmware name \"{}\" not found in the Bucket", n)
            }
            n
        }
        None => {
            anyhow::bail!(
                "Please specify firmware name to continue (you can use the --list option to list all names)"
            )
        }
    };

    let versions = storage
        .list_firmware_versions(&firmware_name)
        .context("Failed to fetch firmware versions from the Bucket")?;

    /* firmware version list command */
    if args.list && args.firmware_version.is_none() {
        println!(
            "Listing {} version{} of firmware \"{}\"",
            versions.len(),
            versions.len().eq(&1).then_some("").unwrap_or("s"),
            firmware_name
        );
        for version in versions {
            println!("  - {}", version);
        }
        exit(0);
    } else if args.list && args.firmware_version.is_some() {
        anyhow::bail!("Invalid use of the --list command")
    }

    /* firmware version sanity checks */
    let firmware_version = match args.firmware_version {
        Some(v) => {
            if !versions.contains(&v) {
                anyhow::bail!("Firmware version \"{}\" not found in the Bucket", v)
            }
            v
        }
        None => {
            anyhow::bail!(
                "Please specify firmware version to continue (you can use the --list option to list all versions)"
            )
        }
    };

    /* Finally onto the firmware flashing itself */
    // Get a list of all available debug probes.
    let lister = Lister::new();
    let probes = lister.list_all();
    // Use the first probe found.
    let probe = probes[0].open().context("Failed to open probe")?;

    let firmware = storage
        .download_firmware(&firmware_name, &firmware_version, &cache_base)
        .context("Failed to download firmware")?;

    // Attach to a chip.
    let mut session = probe
        .attach_under_reset(&firmware.chip, Permissions::default())
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
