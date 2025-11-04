use crate::prelude::*;
use crate::{LABEL, LINK_DIR, MUSIC_DIR, cli::CmdFormat};
use fatfs::{FileSystem, FsOptions};
use fscommon::BufStream;
use std::io::prelude::*;

// TODO untested
pub fn format(args: CmdFormat) -> Result<()> {
    let data = crate::lsblk::query_block_device(&args.target)?;

    // TODO this will say partition but it may not actually be a partition
    crate::confirm_partition(&data, "Formatting".to_string())?;

    if data.is_partition() {
        format_partition(&data.path)?;
    } else {
        format_disk(&data.path)?;
    }

    println!("Setting up the directory structure..");

    setup(&data.path)?;

    println!("Done!");

    Ok(())
}

fn format_partition(path: &str) -> Result<()> {
    use fatfs::{FormatVolumeOptions, StdIoWrapper, format_volume};

    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)?;

    let mut stream = StdIoWrapper::from(BufStream::new(file));

    // quick format
    format_volume(
        &mut stream,
        FormatVolumeOptions::new()
            .fat_type(fatfs::FatType::Fat32)
            .volume_label(LABEL), // TODO allow custom volume names
    )?;

    Ok(())
}

fn format_disk(path: &str) -> Result<()> {
    let mut child = std::process::Command::new("sfdisk")
        .args([path])
        .stdin(std::process::Stdio::piped())
        .spawn()?;

    let mut stdin = child
        .stdin
        .take()
        .with_context(|| anyhow!("Unable to take child stdin"))?;

    // NOTE: basically create MBR partition table and single FAT partition
    stdin.write_all(b"label: dos\ntype=83")?;

    drop(stdin);

    child.wait()?;

    Ok(())
}

fn setup(path: &str) -> Result<()> {
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)?;

    let stream = BufStream::new(file);

    println!("Creating required files..");

    // create the structure
    let fs = FileSystem::new(stream, FsOptions::new())?;
    let root_dir = fs.root_dir();
    root_dir.create_dir(MUSIC_DIR)?;
    root_dir.create_dir(LINK_DIR)?;

    {
        let mut readme = root_dir.create_file("README.txt")?;
        readme.write(crate::text::README.as_bytes())?;
    }

    Ok(())
}
