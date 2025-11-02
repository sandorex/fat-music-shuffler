mod cli;
mod lsblk;

use crate::cli::CmdFormat;
use anyhow::{Context, Result, anyhow, bail};
use clap::Parser;
use fscommon::BufStream;
use std::io::prelude::*;

const LABEL: [u8; 11] = [b'f', b'a', b't', b'3', b'2', b'm', b's', 0, 0, 0, 0];

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
        cli::CliCommands::Format(x) => format(x)?,
        _ => todo!(),
    }

    Ok(())
}

fn format(args: CmdFormat) -> Result<()> {
    use fatfs::{FormatVolumeOptions, StdIoWrapper, format_volume};

    let data = lsblk::query_block_device(&args.target)?;

    if !data.is_partition() {
        bail!("{:?} is not a partition", args.target);
    }

    confirm_prompt(format!(
        "Formatting partition {}{} ({})\nAre you ABSOLUTELY sure you wish to proceed",
        data.path,
        // optional label
        data.label.map(|x| format!(" {x:?}")).unwrap_or_default(),
        data.size
    ))?;

    println!("Formatting..");

    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(data.path)?;

    format_volume(
        &mut StdIoWrapper::from(BufStream::new(file)),
        FormatVolumeOptions::new()
            .fat_type(fatfs::FatType::Fat32)
            .volume_label(LABEL), // TODO allow custom volume names
    )?;

    println!("Done!");

    Ok(())
}
