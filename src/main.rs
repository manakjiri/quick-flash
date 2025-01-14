use anyhow::{self, Context};
use chrono::DateTime;
use clap::{self, Parser};
use quick_flash::credentials::get_credentials_from_command_line;
use quick_flash::credentials_manager::CredentialsManager;
use quick_flash::storage::Storage;
use quick_flash::{flash_firmware, get_probes, BaseDirs};
use std::process::exit;

/// Flash centrally hosted firmware binaries with one command
#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    firmware_name: Option<String>,
    firmware_version: Option<String>,

    /// Lists available firmware names (if given no arguments) or versions of a specific firmware (if given FIRMWARE_NAME)
    #[arg(long, short)]
    list: bool,

    // TODO add '--probe VID:PID' or '--probe VID:PID:Serial'.
    /// Select a specific probe in the list, accepts '--probe Serial'
    #[arg(long)]
    probe: Option<String>,

    /// Lists all available probes
    #[arg(long)]
    list_probes: bool,

    /// Deletes the cache directory prior to running the rest of the program
    #[arg(long)]
    clear_cache: bool,

    /* /// Deletes the credentials file prior to running the rest of the program
    #[arg(long)]
    clear_credentials: bool, */
    /// Use this flag to assert the nreset & ntrst pins during attaching the probe to the chip
    #[arg(long, short('r'))]
    connect_under_reset: bool,

    /// Show dates of last modification for entries in the list
    #[arg(long)]
    dates: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if args.list_probes {
        let probes = get_probes()?;
        println!(
            "VID:PID:Serial (name) listing of {} available debug probe{}:",
            probes.len(),
            if probes.len().eq(&1) { "" } else { "s" }
        );
        for probe in probes {
            println!(
                "  - {:04X}:{:04X}:{} ({})",
                probe.vendor_id,
                probe.product_id,
                probe.serial_number.unwrap_or_default(),
                probe.identifier
            );
        }
        exit(0);
    }

    let base_dirs = BaseDirs::new()?;

    if args.clear_cache {
        eprintln!("Clearing cache directory...");
        base_dirs
            .clear_firmware_cache()
            .context("Failed to clear firmware cache directory")?;
    }

    /* if args.clear_credentials {
        eprintln!("Clearing credentials...");
        if creds_path.exists() {
            fs::remove_file(&creds_path).context("Failed to remove credentials file")?;
        }
    } */

    let creds_manager = CredentialsManager::new(base_dirs.creds_dir);
    let mut all_creds = creds_manager
        .get_all()
        .context("Failed to load saved credentials")?;

    if all_creds.is_empty() {
        let creds = get_credentials_from_command_line()
            .context("Failed to read credentials from the command line")?;
        creds_manager
            .add(creds)
            .context("Failed to save new credentials")?;
        eprintln!("Credentials saved successfully");
        all_creds = creds_manager.get_all()?;
    }

    if all_creds.len() > 1 {
        anyhow::bail!("Multiple credentials management is not supported in this version");
    }

    let creds = all_creds.first().unwrap();

    eprintln!("Connecting to \"{}\" storage...", creds.user_storage_name);
    let storage = Storage::new(creds).context("Failed to init storage client")?;

    let (firmwares, firmwares_modified) = storage
        .list_firmwares()
        .context("Failed to fetch firmware names from the Bucket")?
        .iter()
        .map(|f| (f.name.clone(), f.last_modified))
        .collect::<(Vec<String>, Vec<i64>)>();

    if firmwares.is_empty() {
        anyhow::bail!("No firmware found in the Bucket");
    }

    /* firmware names list command */
    if args.list && args.firmware_name.is_none() {
        println!(
            "Listing {} available firmware name{}:",
            firmwares.len(),
            if firmwares.len().eq(&1) { "" } else { "s" }
        );
        for (name, timestamp) in firmwares.iter().zip(firmwares_modified.iter()) {
            if args.dates {
                println!(
                    "  - {} ({})",
                    name,
                    DateTime::from_timestamp(*timestamp, 0)
                        .ok_or(anyhow::anyhow!("not a timestamp"))?
                );
            } else {
                println!("  - {}", name);
            }
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

    let (versions, versions_modified) = storage
        .list_firmware_versions(&firmware_name)
        .context("Failed to fetch firmware versions from the Bucket")?
        .iter()
        .map(|f| (f.version.clone(), f.last_modified))
        .collect::<(Vec<String>, Vec<i64>)>();

    /* firmware version list command */
    if args.list && args.firmware_version.is_none() {
        println!(
            "Listing {} version{} of firmware \"{}\"",
            versions.len(),
            if versions.len().eq(&1) { "" } else { "s" },
            firmware_name
        );
        for (version, timestamp) in versions.iter().zip(versions_modified.iter()) {
            if args.dates {
                println!(
                    "  - {} ({})",
                    version,
                    DateTime::from_timestamp(*timestamp, 0)
                        .ok_or(anyhow::anyhow!("not a timestamp"))?
                );
            } else {
                println!("  - {}", version);
            }
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
    let probes = get_probes()?;

    // Use the first probe found.
    let probe = match args.probe {
        //TODO add VID:PID:Serial parsing
        Some(ref p) => probes
            .iter()
            .find(|probe| probe.serial_number.as_ref().expect("Probe without serial") == p)
            .context("Probe not found")?,
        None => &probes[0],
    };
    let probe = probe.open().context("Failed to open probe")?;

    let firmware = storage
        .download_firmware(
            &firmware_name,
            &firmware_version,
            &base_dirs.firmware_cache_dir,
        )
        .context("Failed to download firmware")?;

    flash_firmware(probe, firmware, args.connect_under_reset, &|s| {
        eprintln!("{}", s);
    })?;

    Ok(())
}
