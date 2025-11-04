mod cli;
mod commands;
mod lsblk;
mod text;

pub mod prelude {
    pub use anyhow::{Context, Result, anyhow, bail};
}

use crate::lsblk::BlockDeviceInfo;
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

fn confirm_partition(data: &BlockDeviceInfo, prompt: String) -> Result<()> {
    confirm_prompt(format!(
        "{prompt} partition {}{} ({})\nDo you wish to proceed",
        data.path,
        // optional label
        data.label
            .as_ref()
            .map(|x| format!(" {x:?}"))
            .unwrap_or_default(),
        data.size
    ))
}

fn main() -> Result<()> {
    let args = cli::Cli::parse();

    match args.cmd {
        cli::CliCommands::Format(x) => commands::format(x)?,
        cli::CliCommands::Shuffle(x) => commands::shuffle(x)?,
        cli::CliCommands::Clean(x) => commands::clean(x)?,
        _ => todo!(),
    }

    Ok(())
}
