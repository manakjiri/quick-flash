use anyhow;
use clap::{self, Parser};
use etcetera::{self, AppStrategy, AppStrategyArgs};
use probe_rs::probe::{list::Lister, Probe};
use probe_rs::Permissions;
use std::fs;
use std::io::{self, BufRead};
use std::path::{self, Path};
use std::process::exit;

mod config;
mod storage;
mod utils;

/// Flash centrally hosted firmware binaries with one command
#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    firmware_name: Option<String>,

    #[arg(long)]
    list: bool,
}

fn main() -> anyhow::Result<()> {
    /*let s3_path = "test.file";
    let test = b"I'm going to S3!"; */

    //let response_data = bucket.put_object(s3_path, test).await?;
    //assert_eq!(response_data.status_code(), 200);

    //let response_data = bucket.get_object(s3_path).await?;
    //assert_eq!(response_data.status_code(), 200);
    //assert_eq!(test, response_data.as_slice());

    let args = Args::parse();

    let strategy = etcetera::choose_app_strategy(AppStrategyArgs {
        top_level_domain: "cz".to_string(),
        author: "manakjiri".to_string(),
        app_name: "quick-flash".to_string(),
    })?;
    fs::create_dir_all(strategy.config_dir())?;
    let creds_path = strategy.config_dir().join("credentials.toml");

    let creds = config::get_credentials(&creds_path)?;
    let storage = storage::Storage::new(&creds)?;

    if args.list {
        match args.firmware_name {
            Some(name) => {
                for version in storage.list_firmware_versions(&name)? {
                    println!("{}", version);
                }
            }
            None => {
                for name in storage.list_firmwares()? {
                    println!("{}", name);
                }
            }
        }
        exit(0);
    }

    /* let response_data = bucket.list("".to_string(), None)?;
    println!(
        "{:?}",
        response_data[0]
            .contents
            .iter()
            .map(|f| f.key.to_owned())
            .collect::<Vec<String>>()
    ); */

    //let response_data = bucket.delete_object(s3_path).await?;
    //assert_eq!(response_data.status_code(), 204);
    Ok(())
}

/* fn main() -> Result<(), probe_rs::Error> {
    // Get a list of all available debug probes.
    let lister = Lister::new();

    let probes = lister.list_all();

    // Use the first probe found.
    let mut probe = probes[0].open()?;

    // Attach to a chip.
    let mut session = probe.attach("STM32F103RB", Permissions::default())?;

    // Select a core.
    let mut core = session.core(0)?;

    // Halt the attached core.
    core.halt(std::time::Duration::from_millis(10))?;

    Ok(())
} */
