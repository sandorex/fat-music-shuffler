use crate::cli::CmdShuffle;
use crate::util::BlockDevice;
use crate::{DIRTY_FLAG_FILE, prelude::*};
use crate::{LINK_DIR, MUSIC_DIR, MUSIC_EXT};
use fatfs::{FileSystem, FsOptions};
use fscommon::BufStream;
use rand::seq::SliceRandom;
use std::io::Write;
use std::time::Duration;

pub fn shuffle(target: BlockDevice, interactive: bool, cmd_args: CmdShuffle) -> Result<()> {
    if interactive {
        crate::confirm_prompt(format!(
            "Shuffling music on partition {target}, do you wish to proceed?",
        ))?;
    }

    let file = target.open(false)?;
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

    let repeat_count = if let Some(repeat_duration) = cmd_args.repeat_fill {
        if duration.as_secs_f64() > repeat_duration.as_secs_f64() {
            bail!(
                "Duration requested is lower than total duration of songs ({} < {})",
                repeat_duration,
                humantime::format_duration(duration)
            );
        }

        println!(
            "Found {} songs, total duration is {}",
            music.len(),
            humantime::format_duration(duration)
        );

        // NOTE: this will usually overshoot but it does not matter
        let repeat_count = (repeat_duration.as_secs_f64() / duration.as_secs_f64())
            .ceil()
            .round() as usize;

        println!();

        if interactive {
            crate::confirm_prompt(format!(
                "The songs would repeat {repeat_count} times to achieve duration of at least {}, do you wish to proceed?",
                repeat_duration
            ))?;
        }

        repeat_count
    } else {
        1
    };

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

        if !old_links.is_empty() {
            for (i, file_name) in old_links.iter().enumerate() {
                link_dir.remove_entry(file_name)?;
                print!("\rRemoving old links [{}/{}]", i + 1, old_links.len());
                let _ = std::io::stdout().flush();
            }
            println!();
        }
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
