use std::time::Duration;
use crate::cli::CmdShuffle;
use crate::{DIRTY_FLAG_FILE, prelude::*};
use crate::{LINK_DIR, MUSIC_DIR, MUSIC_EXT};
use fatfs::{FileSystem, FsOptions};
use fscommon::BufStream;
use rand::seq::SliceRandom;

pub fn shuffle(args: CmdShuffle) -> Result<()> {
    let data = crate::lsblk::query_block_device(&args.target)?;

    if !data.is_partition() {
        bail!("{:?} is not a partition", args.target);
    }

    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&data.path)?;

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
        if name.ends_with(&format!(".mp3{MUSIC_EXT}")) {
            let mut file = entry.to_file();
            let dur = mp3_duration::from_read(&mut file)?;
            music.push(name);
            duration += dur;
        }
    }

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

    // NOTE: this will usually overshoot but it does not matter
    let repeat_count = (args.fill_duration.as_secs_f64() / duration.as_secs_f64())
        .ceil()
        .round() as usize;

    println!(
        "The songs would repeat {repeat_count} times to achieve duration of {}",
        args.fill_duration
    );

    crate::confirm_partition(&data, "No changes were made to".to_string())?;

    // basically a flag that the filesystem contains links
    root_dir.create_file(DIRTY_FLAG_FILE)?;

    let mut rng = rand::rng();
    let music_len = music.len();

    let link_dir = root_dir.create_dir(LINK_DIR)?;

    println!("Hardlinking music files..");

    for repeat_index in 0..repeat_count {
        music.shuffle(&mut rng);

        for (i, name) in music.iter().enumerate() {
            let link_name = format!("{}.mp3", (repeat_index * music_len) + i);
            link_dir.create_hardlink(&link_name, &music_dir, name)?;
        }
    }

    println!("Created {} hardlinks", repeat_count * music_len);

    Ok(())
}
