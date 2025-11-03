use crate::prelude::*;
use crate::{LABEL, LINK_DIR, MUSIC_DIR, cli::CmdFormat};
use fatfs::{FileSystem, FsOptions};
use fscommon::BufStream;
use std::io::prelude::*;

pub fn format(args: CmdFormat) -> Result<()> {
    use fatfs::{FormatVolumeOptions, StdIoWrapper, format_volume};

    let data = crate::lsblk::query_block_device(&args.target)?;

    if !data.is_partition() {
        bail!("{:?} is not a partition", args.target);
    }

    crate::confirm_partition(&data, "Formatting".to_string())?;

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
        readme.write(crate::text::README.as_bytes())?;
    }

    println!("Done!");

    Ok(())
}
