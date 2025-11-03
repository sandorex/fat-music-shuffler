mod cli;
mod lsblk;
mod text;

use crate::cli::{CmdFormat, CmdShuffle};
use anyhow::{Context, Result, anyhow, bail};
use clap::Parser;
use fatfs::{FileSystem, FsOptions};
use fscommon::BufStream;
use rand::seq::SliceRandom;
use std::{io::prelude::*, ops::Div, time::Duration};

const LABEL: [u8; 11] = [b'f', b'a', b't', b'3', b'2', b'm', b's', 0, 0, 0, 0];

/// Directory containing the original music files
const MUSIC_DIR: &str = "ORIG";

/// Suffix used to prevent player from playing the original music files
const MUSIC_EXT: &str = ".mp3.x";

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

fn main() -> Result<()> {
    let args = cli::Cli::parse();

    match args.cmd {
        cli::CliCommands::Format(x) => format(x)?,
        cli::CliCommands::Shuffle(x) => shuffle(x)?,
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

    let mut stream = StdIoWrapper::from(BufStream::new(file));

    // quick format
    format_volume(
        &mut stream,
        FormatVolumeOptions::new()
            .fat_type(fatfs::FatType::Fat32)
            .volume_label(LABEL), // TODO allow custom volume names
    )?;

    println!("Creating required files..");

    // create the structure
    let fs = FileSystem::new(stream, FsOptions::new())?;
    let root_dir = fs.root_dir();
    root_dir.create_dir(MUSIC_DIR)?;
    root_dir.create_dir(LINK_DIR)?;

    {
        let mut readme = root_dir.create_file("README.txt")?;
        readme.write(text::README.as_bytes())?;
    }

    println!("Done!");

    Ok(())
}

fn shuffle(args: CmdShuffle) -> Result<()> {
    let data = lsblk::query_block_device(&args.target)?;

    if !data.is_partition() {
        bail!("{:?} is not a partition", args.target);
    }

    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(data.path)?;

    let stream = BufStream::new(file);

    let fs = FileSystem::new(stream, FsOptions::new())?;
    let root_dir = fs.root_dir();

    let music_dir = root_dir.open_dir(MUSIC_DIR)?;

    let mut music: Vec<String> = vec![];
    let mut duration: Duration = Duration::from_secs(0);

    for entry in music_dir.iter().flatten() {
        if !entry.is_file() {
            continue;
        }

        let name = entry.file_name();
        if name.ends_with(MUSIC_EXT) {
            let mut file = entry.to_file();
            let dur = mp3_duration::from_read(&mut file)?;
            music.push(name);
            duration += dur;
        }
    }

    drop(music_dir);

    if music.len() < 3 {
        bail!("Shuffling requires at least 3 songs to be present!");
    }

    if duration.as_secs_f64() > args.fill_duration.as_secs_f64() {
        bail!(
            "Duration requested is lower than total duration of songs ({} < {})",
            args.fill_duration,
            humantime::format_duration(duration)
        );
    }

    println!(
        "Found {} songs, total duration is {}",
        music.len(),
        humantime::format_duration(duration)
    );

    let repeat_count = (args.fill_duration.as_secs_f64() / duration.as_secs_f64())
        .ceil()
        .round() as usize;

    println!(
        "The songs would repeat {repeat_count} times to achieve duration of {}",
        args.fill_duration
    );

    confirm_prompt("No modifications were made yet, do you wish to continue?".to_string())?;

    let mut rng = rand::rng();
    let music_len = music.len();

    let links_dir = root_dir.create_dir(LINK_DIR)?;

    for repeat_index in 0..repeat_count {
        music.shuffle(&mut rng);

        for (i, name) in music.iter().enumerate() {
            links_dir.create_hardlink(&format!("{}.mp3", (repeat_index * music_len) + i), name)?;
            // println!("create {name:?} index {}", (repeat_index * music_len) + i);
        }
    }

    Ok(())
}
