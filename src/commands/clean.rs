use crate::cli::CmdClean;
use crate::{DIRTY_FLAG_FILE, LINK_DIR, prelude::*};
use fatfs::{FileSystem, FsOptions};
use fscommon::BufStream;

pub fn clean(args: CmdClean) -> Result<()> {
    let data = crate::lsblk::query_block_device(&args.target)?;

    if !data.is_partition() {
        bail!("{:?} is not a partition", args.target);
    }

    crate::confirm_partition(&data, "Cleaning".to_string())?;

    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&data.path)?;

    let stream = BufStream::new(file);

    let fs = FileSystem::new(stream, FsOptions::new())?;
    let root_dir = fs.root_dir();

    let links_dir = root_dir.open_dir(LINK_DIR)?;

    // delete entries for all the links
    let mut count: usize = 0;
    for i in links_dir.iter().flatten().map(|x| x.file_name()) {
        links_dir.remove_entry(&i)?;
        count += 1;
    }

    // delete the flag file if present
    let _ = root_dir.remove(DIRTY_FLAG_FILE);

    println!("Deleted {count} hardlinks");

    Ok(())
}
