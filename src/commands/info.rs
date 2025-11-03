use std::{os::unix::fs::FileTypeExt, time::Duration};

use crate::{DIRTY_FLAG_FILE, LINK_DIR, MUSIC_DIR, cli::CmdInfo, prelude::*};
use fatfs::{Dir, FileSystem, FsOptions, ReadWriteSeek};
use fscommon::BufStream;

pub fn info(args: CmdInfo) -> Result<()> {
    let meta = std::fs::metadata(&args.target)?;

    if meta.file_type().is_block_device() {
        info_partition(args)?;
    } else if meta.file_type().is_dir() {
        info_mounted(args)?;
    } else {
        bail!("Invalid target path {:?}", args.target);
    }

    Ok(())
}

// when target is the partition block device
fn info_partition(args: CmdInfo) -> Result<()> {
    let data = crate::lsblk::query_block_device(&args.target)?;

    if !data.is_partition() {
        bail!("{:?} is not a partition", args.target);
    }

    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(false)
        .open(&data.path)?;

    let stream = BufStream::new(file);

    let fs = FileSystem::new(stream, FsOptions::new())?;
    let root_dir = fs.root_dir();

    let is_dirty = match root_dir.open_file(DIRTY_FLAG_FILE) {
        Ok(_) => true,
        Err(fatfs::Error::NotFound) => false,
        Err(e) => bail!(e),
    };

    println!("Inspecting {:?}", data.path);
    println!("Dirty: {is_dirty}");

    let mut duration: Duration = Duration::from_secs(0);
    for entry in root_dir.open_dir(MUSIC_DIR)?.iter().flatten() {
        // TODO should be recursive
        if !entry.is_file() {
            continue;
        }

        let mut file = entry.to_file();
        duration += mp3_duration::from_read(&mut file)?;
    }
    println!("Original runtime: {}", humantime::format_duration(duration));

    let mut link_count = 0;
    duration = Duration::from_secs(0);
    for entry in root_dir.open_dir(LINK_DIR)?.iter().flatten() {
        // TODO should be recursive
        if !entry.is_file() {
            continue;
        }

        let mut file = entry.to_file();
        duration += mp3_duration::from_read(&mut file)?;
        link_count += 1;
    }

    println!("Playlist runtime: {}", humantime::format_duration(duration));
    println!("Hardlink count: {link_count}");

    Ok(())
}

// when target is mounted path
fn info_mounted(args: CmdInfo) -> Result<()> {
    todo!();
    // let root_dir = std::path::Path::new(&args.target);
    // let is_dirty = std::fs::exists(root_dir.join(DIRTY_FLAG_FILE)).unwrap_or(false);

    // println!("Dirty: {is_dirty}");

    // let mut link_count = 0;
    // let mut duration: Duration = Duration::from_secs(0);
    // for entry in root_dir.read_dir()?.flatten() {
    //     entry.file_type()
    //     // let mut file = entry.to_file();
    //     // duration += mp3_duration::from_read(&mut file)?;
    //     // link_count += 1;
    // }

    // println!("Total runtime: {}", humantime::format_duration(duration));
    // println!("Total hardlinks: {link_count}");

    // Ok(())
}
