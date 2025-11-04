use crate::cli::CmdShuffle;
use crate::{DIRTY_FLAG_FILE, prelude::*};
use crate::{LINK_DIR, MUSIC_DIR, MUSIC_EXT};
use fatfs::{FileSystem, FsOptions};
use fscommon::BufStream;
use rand::seq::SliceRandom;
use std::io::Write;
use std::time::Duration;

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
        "The songs would repeat {repeat_count} times to achieve duration of at least {}",
        args.fill_duration
    );

    crate::confirm_partition(&data, "No changes were made to".to_string())?;

    // basically a flag that the filesystem contains links
    root_dir.create_file(DIRTY_FLAG_FILE)?;

    let mut rng = rand::rng();
    let music_len = music.len();

    let link_dir = root_dir.create_dir(LINK_DIR)?;

    // clean the old links before creating new ones
    {
        // count links ignoring any directories
        let old_links = link_dir
            .iter()
            .flatten()
            .filter(|x| x.is_file())
            .map(|x| x.file_name())
            .collect::<Vec<_>>();

        for (i, file_name) in old_links.iter().enumerate() {
            link_dir.remove_entry(file_name)?;
            print!("\rRemoving old links [{}/{}]", i + 1, old_links.len());
            let _ = std::io::stdout().flush();
        }
        println!();
    }

    let total_link_count = repeat_count * music_len;
    for repeat_index in 0..repeat_count {
        music.shuffle(&mut rng);

        for (i, name) in music.iter().enumerate() {
            let index = (repeat_index * music_len) + i;
            print!("\rCreating new links [{}/{total_link_count}]", index + 1);
            let _ = std::io::stdout().flush();
            link_dir.create_hardlink(&format!("{}.mp3", index), &music_dir, name)?;
        }
    }

    println!("\nDone!");

    Ok(())
}
