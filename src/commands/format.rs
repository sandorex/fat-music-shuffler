use crate::prelude::*;
use crate::util::BlockDevice;
use crate::{LABEL, LINK_DIR, MUSIC_DIR};
use fatfs::{FileSystem, FsOptions};
use fscommon::BufStream;
use std::io::prelude::*;

pub fn format(mut target: BlockDevice, interactive: bool) -> Result<()> {
    if interactive {
        crate::confirm_prompt(format!(
            "Formatting {} {target}, do you wish to proceed?",
            if target.is_partition {
                "partition"
            } else {
                "disk"
            }
        ))?;
    }

    // if its a disk format the whole disk
    if !target.is_partition {
        format_disk(&target.path)?;

        // NOTE it should always be the first and only partition
        target.path = format!("{}1", target.path);

        if std::fs::exists(&target.path).unwrap_or(false) {
            bail!("Partition does not exist after formatting");
        }
    }

    format_partition(&target)?;
    setup(&target)?;

    println!(
        "Formatting done, for any other commands please use {:?} as the device path",
        target.path
    );

    Ok(())
}

fn format_partition(target: &BlockDevice) -> Result<()> {
    use fatfs::{FormatVolumeOptions, StdIoWrapper, format_volume};

    let file = target.open(false)?;
    let mut stream = StdIoWrapper::from(BufStream::new(file));

    // quick format
    format_volume(
        &mut stream,
        FormatVolumeOptions::new()
            .fat_type(fatfs::FatType::Fat32)
            .volume_label(LABEL),
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

fn setup(target: &BlockDevice) -> Result<()> {
    println!("Setting up the directory structure..");

    let file = target.open(false)?;
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
