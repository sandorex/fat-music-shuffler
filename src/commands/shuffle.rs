use crate::{DIRTY_FLAG_FILE, LINK_DIR, MUSIC_DIR, MUSIC_EXT, cli::CmdShuffle, prelude::*, util::Partition};
use fatfs::{FileSystem, FsOptions, IoBase, OemCpConverter, ReadWriteSeek, TimeProvider};
use fscommon::BufStream;
use rand::seq::SliceRandom;
use std::io::Write;
use std::time::Duration;

pub fn shuffle(target: Partition, interactive: bool, cmd_args: CmdShuffle) -> Result<()> {
    if interactive {
        crate::confirm_prompt(format!(
            "Shuffling music on partition {target}, do you wish to proceed?",
        ))?;
    }

    let file = target.open(false)?;
    let stream = BufStream::new(file);
    let fs = FileSystem::new(stream, FsOptions::new())?;

    shuffle_raw(&fs, interactive, cmd_args)?;

    fs.unmount()?;

    Ok(())
}

/// Remember to call `fs.unmount` afterwards!
fn shuffle_raw<IO, TP, OCC>(fs: &FileSystem<IO, TP, OCC>, interactive: bool, cmd_args: CmdShuffle) -> Result<()>
where
    IO: ReadWriteSeek,
    TP: TimeProvider,
    OCC: OemCpConverter,

    // damn you rust
    <IO as fatfs::IoBase>::Error: std::error::Error + Send + Sync + 'static,
    std::io::Error: From<fatfs::Error<<IO as IoBase>::Error>>
{
    {
        let root_dir = fs.root_dir();
        let music_dir = root_dir.open_dir(MUSIC_DIR)
            .with_context(|| anyhow!("music directory {MUSIC_DIR:?} not found"))?;

        let mut music: Vec<String> = vec![];

        for entry in music_dir.iter().flatten() {
            if !entry.is_file() {
                continue;
            }

            let name = entry.file_name();
            if name.ends_with(&format!(".mp3{MUSIC_EXT}")) || name.ends_with(".mp3") {
                music.push(name);
            }
        }

        // this is expensive so call it only when needed
        let calculate_duration = || -> Result<Duration> {
            let mut duration = Duration::from_secs(0);

            for name in music.iter() {
                let mut file = music_dir.open_file(&name)?;
                let dur = mp3_duration::from_read(&mut file)?;
                duration += dur;
            }

            Ok(duration)
        };

        let repeat_count = if let Some(repeat_duration) = cmd_args.repeat_fill {
            if music.len() < 3 {
                bail!("Shuffling with repeat_fill requires at least 3 songs!");
            }

            let duration = calculate_duration()?;

            println!(
                "Found {} songs, total duration is {}",
                music.len(),
                humantime::format_duration(duration)
            );

            // NOTE: this will usually overshoot but it does not matter
            let repeat_count = (repeat_duration.as_secs_f64() / duration.as_secs_f64())
                .ceil()
                .round() as usize;

            if interactive {
                crate::confirm_prompt(format!(
                    "The songs would repeat {repeat_count} times to achieve duration of at least {}, do you wish to proceed?",
                    repeat_duration
                ))?;
            }

            repeat_count
        } else if let Some(repeat) = cmd_args.repeat {
            repeat.into()
        } else {
            1
        };

        // basically a flag that signifies that the filesystem contains links
        root_dir.create_file(DIRTY_FLAG_FILE)?;

        let mut rng = rand::rng();
        let music_len = music.len();

        let link_dir = root_dir.create_dir(LINK_DIR)?;

        // clean the old links before creating new ones
        {
            // gather links ignoring any directories
            let old_links = link_dir
                .iter()
                .flatten()
                .filter(|x| x.is_file())
                .map(|x| x.file_name())
                .collect::<Vec<_>>();

            if !old_links.is_empty() {
                println!("Removing old links ({})", old_links.len());
                for file_name in &old_links {
                    link_dir.remove_entry(file_name)?;
                }
            }
        }

        // batch link creation for each repeat, simplest solution
        for repeat_index in 0..repeat_count {
            music.shuffle(&mut rng);

            print!("\rLinking music [{}/{repeat_count}]", repeat_index + 1);
            let _ = std::io::stdout().flush();

            let links = music
                .iter()
                .enumerate()
                .map(|(i, x)| (format!("{}.mp3", i + music_len * repeat_index), x.clone()))
                .collect::<Vec<_>>();

            link_dir.create_hardlinks(&links, &music_dir)
                .with_context(|| anyhow!("error creating links"))?;
        }

        println!("\nCreated {} links", repeat_count * music_len);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Write};
    use crate::{LINK_DIR, MUSIC_DIR};

    // TODO i just picked first value that worked for FAT32, as there are a lot of files in single directory
    const BYTES_PER_SECTOR: u16 = 512;
    const SECTOR_COUNT: u32 = 131070;
    const FS_SIZE: usize = SECTOR_COUNT as usize * BYTES_PER_SECTOR as usize;

    const MUSIC_COUNT: usize = 300;
    const REPEAT_COUNT: u16 = 15;

    #[test]
    fn test_shuffle() {
        let mut buf: Vec<u8> = Vec::with_capacity(FS_SIZE);
        let mut image = Cursor::new(&mut buf);

        fatfs::format_volume(
            &mut fatfs::StdIoWrapper::from(&mut image),
            fatfs::FormatVolumeOptions::new()
                .fat_type(fatfs::FatType::Fat32) // FAT32 is required cause there is a lot of files
                .bytes_per_sector(BYTES_PER_SECTOR)
                .total_sectors(SECTOR_COUNT)
        )
        .expect("format volume");

        let fs = fatfs::FileSystem::new(image, fatfs::FsOptions::new())
            .expect("open fs");

        let root_dir = fs.root_dir();

        let music_dir = root_dir
            .create_dir(MUSIC_DIR)
            .expect("create music dir");

        let link_dir = root_dir
            .create_dir(LINK_DIR)
            .expect("create dir link");

        for i in 0..MUSIC_COUNT {
            let name = format!("{i}.mp3");

            let mut file = music_dir
                .create_file(&name)
                .expect(&format!("create file {name:?}"));

            file
                .write_all(&format!("file{}", i + 1).into_bytes())
                .expect(&format!("write to file {name:?}"));
        }

        super::shuffle_raw(
            &fs,
            false,
            crate::cli::CmdShuffle { repeat_fill: None, repeat: Some(REPEAT_COUNT) }
        )
        .expect("shuffle");

        // ignore previous and current directory
        let link_count = link_dir.iter().count() - 2;

        // ensure the same amount of links are created
        assert_eq!(link_count, MUSIC_COUNT * REPEAT_COUNT as usize);
    }
}
