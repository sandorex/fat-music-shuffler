use crate::cli::CmdClean;
use crate::{DIRTY_FLAG_FILE, LINK_DIR, prelude::*};
use fatfs::{FileSystem, FsOptions};
use fscommon::BufStream;
use std::io::Write;

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

    let link_dir = root_dir.open_dir(LINK_DIR)?;

    // count links ignoring any directories
    let links = link_dir
        .iter()
        .flatten()
        .filter(|x| x.is_file())
        .map(|x| x.file_name())
        .collect::<Vec<_>>();

    for (i, file_name) in links.iter().enumerate() {
        link_dir.remove_entry(file_name)?;
        print!("\rRemoving old links [{}/{}]", i + 1, links.len());
        let _ = std::io::stdout().flush();
    }

    // delete the flag file if present
    let _ = root_dir.remove(DIRTY_FLAG_FILE);

    println!("\nDone!");

    Ok(())
}
