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
use std::{char::REPLACEMENT_CHARACTER, collections::HashMap, io::prelude::*, ops::Deref};

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

#[deprecated]
fn confirm_partition(bd: &BlockDevice, prompt: String) -> Result<()> {
    confirm_prompt(format!(
        "{prompt} {} {bd}\nDo you wish to proceed",
        if bd.is_partition { "partition" } else { "disk" }
    ))
}

fn choose_option(options: &Vec<String>) -> Result<usize> {
    println!();
    for (i, opt) in options.iter().enumerate() {
        println!("{i}: {opt}");
    }

    let mut buffer = String::with_capacity(16);
    let mut ans: String;

    let index = loop {
        print!("Please choose an option (0-{}): ", options.len() - 1);
        std::io::stdout().flush()?;

        buffer.clear();
        std::io::stdin().read_line(&mut buffer)?;

        ans = buffer.trim().to_string();

        // allow user to abort by entering nothing
        if ans.is_empty() {
            bail!("User aborted");
        }

        match ans.parse::<usize>() {
            Ok(x) if x < options.len() => break x,

            _ => {
                println!("Invalid answer {ans:?}");
                continue;
            }
        }
    };

    Ok(index)
}

fn main() -> Result<()> {
    let args = cli::Cli::parse();

    dbg!(&args);

    let device = if let Some(target) = args.target.as_ref() {
        lsblk::query_block_device(target)?
    } else {
        let devices = lsblk::query_all_block_devices()?;
        let mut map: HashMap<usize, &BlockDevice> = HashMap::new();
        let mut options: Vec<String> = vec![];
        let mut count: usize = 0;

        for disk in devices.iter() {
            map.insert(count, disk);
            options.push(format!("{disk}"));
            count += 1;

            if let Some(partitions) = disk.partitions.as_ref() {
                for part in partitions {
                    map.insert(count, part);
                    options.push(format!(" {part}"));
                    count += 1;
                }
            }
        }

        let ans = choose_option(&options)?;

        (*map.get(&ans).unwrap()).clone()
    };

    println!("chosen one: {device}");

    match args.cmd {
        // cli::CliCommands::Format(x) => commands::format(x)?,
        cli::CliCommands::Shuffle(x) => commands::shuffle(device, x)?,
        // cli::CliCommands::Clean(x) => commands::clean(x)?,
        _ => todo!(),
    }

    Ok(())
}
