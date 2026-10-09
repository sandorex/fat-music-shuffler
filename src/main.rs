mod cli;
mod commands;
mod lsblk;
mod text;
mod util;

#[cfg(feature = "gui")]
mod gui;

pub mod prelude {
    pub use anyhow::{Context, Result, anyhow, bail};
}

use crate::util::DiskOrPartition;
use clap::Parser;
use prelude::*;
use std::io::prelude::*;

/// Partition label, something recognizable so you don't mess with it
const LABEL: [u8; 11] = [b'F', b'A', b'T', b'3', b'2', b'M', b'S', 0, 0, 0, 0];

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

// TODO make two functions that return either only Disks or Partitions so i dont have to unpack
// DiskOrPartition
fn ask_for_target(no_disk: bool, only_removable: bool) -> Result<DiskOrPartition> {
    let devices = lsblk::query_all_block_devices()?;

    let find_device = |path: &str| -> Option<DiskOrPartition> {
        for device in &devices {
            if device.path() == path {
                return Some(device.clone());
            }

            if let DiskOrPartition::Disk(disk) = &device {
                for partition in &disk.partitions {
                    return Some(DiskOrPartition::Partition(partition.clone()))
                }
            }
        }

        None
    };

    for device in &devices {
        // hide non-removable if requested
        if only_removable && !device.removeable() {
            continue;
        }

        println!("{device}");
        if let DiskOrPartition::Disk(disk) = &device {
            for part in &disk.partitions {
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
            if no_disk && !matches!(device, DiskOrPartition::Partition(_)) {
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

pub fn handle_cli(args: cli::Cli) -> Result<()> {
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

            let DiskOrPartition::Partition(target) = target else { panic!("Target is not a partition") };

            commands::shuffle(target, true, x)?;
        }
        cli::CliCommands::Clean(x) => {
            let target = if let Some(target) = args.target.as_ref() {
                crate::lsblk::query_block_device(target)?
            } else {
                crate::ask_for_target(true, !args.show_all_disks)?
            };

            let DiskOrPartition::Partition(target) = target else { panic!("Target is not a partition") };

            commands::clean(target, true, x)?;
        }
        cli::CliCommands::Import(x) => {
            let target = if let Some(target) = args.target.as_ref() {
                crate::lsblk::query_block_device(target)?
            } else {
                crate::ask_for_target(true, !args.show_all_disks)?
            };

            let DiskOrPartition::Partition(target) = target else { panic!("Target is not a partition") };

            commands::import(target, true, x)?;
        }
        cli::CliCommands::Process(x) => commands::process(true, x)?,
    }

    Ok(())
}

#[cfg(not(feature = "gui"))]
fn main() -> Result<()> {
    let args = cli::Cli::parse();

    handle_cli(args)
}

#[cfg(feature = "gui")]
pub use gui::main;

