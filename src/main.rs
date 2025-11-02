mod cli;
mod lsblk;

use anyhow::{Context, Result, anyhow, bail};
use clap::Parser;
use fatfs::{FileSystem, FsOptions};
use fscommon::BufStream;
use std::fmt::Display;
use std::fs::OpenOptions;
use std::io::{self, prelude::*};
use std::os::unix::fs::FileTypeExt;

#[derive(Debug, Clone)]
pub enum Target {
    /// The device is a block device
    BlockDevice(String),

    /// Using a file
    File(String),
}

// just print the target itself
impl Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::BlockDevice(x) => x,
            Self::File(x) => x,
        };
        f.write_str(str)
    }
}

impl TryFrom<String> for Target {
    type Error = anyhow::Error;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        let meta = std::fs::metadata(&value)
            .with_context(|| anyhow!("Error getting metadata for file {value:?}"))?;

        let ftype = meta.file_type();
        if ftype.is_block_device() {
            Ok(Self::BlockDevice(value))
        } else if ftype.is_file() {
            Ok(Self::File(value))
        } else {
            // invalid path
            bail!("Invalid path {value:?}")
        }
    }
}

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

fn main() -> Result<()> {
    let args = cli::Cli::parse();
    dbg!(&args);

    match args.cmd {
        cli::CliCommands::Format(x) => {
            let target = TryInto::<Target>::try_into(x.target.clone())?;

            match &target {
                Target::BlockDevice(x) => {
                    let data = lsblk::query_block_device(x)?;
                    println!("{data:#?}");

                    if !data.is_partition() {
                        bail!("The target has to be a partition!");
                    }

                    // confirm_prompt(format!(
                    //     "Erasing {target} ({size} MiB)\nAre you ABSOLUTELY sure you wish to proceed"
                    // ))?;

                    // println!("would erase");
                }
                _ => {}
            }
        }
        _ => todo!(),
    }

    Ok(())
}
