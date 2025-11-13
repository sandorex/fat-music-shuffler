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
        println!("Formatting the disk..");
        format_disk(&target.path)?;

        // wait for the partition to be reloaded
        std::thread::sleep(std::time::Duration::from_secs(1));

        // re-query the device
        target = crate::lsblk::query_block_device(&target.path)?;

        // use first partition
        target = target
            .partitions
            .and_then(|x| x.first().cloned())
            .with_context(|| anyhow!("Could not find a partition after formatting"))?;
    }

    println!("Formatting the partition..");
    format_partition(&target)?;

    println!("Setting up the directory structure..");
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
        // always wipe partitions
        .args(["--wipe", "always", "--wipe-partitions", "always", path])
        .stdin(std::process::Stdio::piped())
        // TODO print on error and always in debug builds
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?;

    let mut stdin = child
        .stdin
        .take()
        .with_context(|| anyhow!("Unable to take child stdin"))?;

    // NOTE: basically create MBR partition table and single "W95 FAT32 (LBA)" partition
    stdin.write_all(b"label: dos\ntype=c")?;

    drop(stdin);

    child.wait()?;

    Ok(())
}

fn setup(target: &BlockDevice) -> Result<()> {
    let file = target.open(false)?;
    let stream = BufStream::new(file);

    // create the structure
    let fs = FileSystem::new(stream, FsOptions::new())?;
    {
        let root_dir = fs.root_dir();
        root_dir.create_dir(MUSIC_DIR)?;
        root_dir.create_dir(LINK_DIR)?;

        {
            let mut readme = root_dir.create_file("README.txt")?;
            readme.write(crate::text::README.as_bytes())?;
        }
    }

    fs.unmount()?;

    Ok(())
}
