mod cli;
mod commands;
mod lsblk;
mod text;
mod util;

pub mod prelude {
    pub use anyhow::{Context, Result, anyhow, bail};
}

use crate::util::BlockDevice;
use clap::Parser;
use prelude::*;
use std::io::prelude::*;

const LABEL: [u8; 11] = [b'f', b'a', b't', b'3', b'2', b'm', b's', 0, 0, 0, 0];

/// Directory containing the original music files
const MUSIC_DIR: &str = "ORIG";

/// Suffix used to prevent player from playing the original music files
const MUSIC_EXT: &str = ".x";

/// Directory that contains all hardlinks
const LINK_DIR: &str = "LINK";

/// File that signifies if the partition is dirty and contains hardlinks
const DIRTY_FLAG_FILE: &str = "DO_NOT_MODIFY";

fn confirm_prompt(prompt: String) -> Result<()> {
    print!("{prompt} (y/N): ");
    std::io::stdout().flush()?;

    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer)?;

    match buffer.trim() {
        "y" | "Y" | "yes" => {}
        _ => bail!("User aborted"),
    }

    Ok(())
}

fn ask_for_target(no_disk: bool, only_removable: bool) -> Result<BlockDevice> {
    let devices = lsblk::query_all_block_devices()?;

    let find_device = |path: &str| -> Option<&BlockDevice> {
        for device in &devices {
            if device.path == path {
                return Some(device);
            }

            if let Some(found) = device
                .partitions
                .as_ref()
                .and_then(|x| x.iter().find(|y| y.path == path))
            {
                return Some(found);
            }
        }

        None
    };

    for device in &devices {
        // hide non-removable if requested
        if only_removable && !device.removable {
            continue;
        }

        println!("{device}");
        if let Some(partitions) = &device.partitions {
            for part in partitions {
                println!("  {part}");
            }
        }
    }

    let mut buffer = String::with_capacity(32);
    let mut ans: String;

    let device = loop {
        print!("Enter device path: ");
        std::io::stdout().flush()?;

        buffer.clear();
        std::io::stdin().read_line(&mut buffer)?;

        ans = buffer.trim().to_string();

        // allow user to abort by entering nothing
        if ans.is_empty() {
            bail!("User aborted");
        }

        if let Some(device) = find_device(&ans) {
            if no_disk && !device.is_partition {
                println!("Invalid path, {ans:?} is not a partition");
                continue;
            }

            break device;
        } else {
            println!("Invalid path, {ans:?} is not a device");
            continue;
        }
    };

    Ok(device.clone())
}

fn main() -> Result<()> {
    let args = cli::Cli::parse();

    match args.cmd {
        cli::CliCommands::Format => {
            let target = if let Some(target) = args.target.as_ref() {
                crate::lsblk::query_block_device(target)?
            } else {
                crate::ask_for_target(false, !args.show_all_disks)?
            };

            commands::format(target, true)?;
        }
        cli::CliCommands::Shuffle(x) => {
            let target = if let Some(target) = args.target.as_ref() {
                crate::lsblk::query_block_device(target)?
            } else {
                crate::ask_for_target(true, !args.show_all_disks)?
            };

            commands::shuffle(target, true, x)?;
        }
        cli::CliCommands::Clean => {
            let target = if let Some(target) = args.target.as_ref() {
                crate::lsblk::query_block_device(target)?
            } else {
                crate::ask_for_target(true, !args.show_all_disks)?
            };

            commands::clean(target, true)?;
        }
        cli::CliCommands::Import(x) => {
            let target = if let Some(target) = args.target.as_ref() {
                crate::lsblk::query_block_device(target)?
            } else {
                crate::ask_for_target(true, !args.show_all_disks)?
            };

            commands::import(target, true, x)?;
        }
    }

    Ok(())
}
