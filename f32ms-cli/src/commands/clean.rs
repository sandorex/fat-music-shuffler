use crate::cli::CmdClean;
use crate::util::BlockDevice;
use crate::{DIRTY_FLAG_FILE, LINK_DIR, MUSIC_DIR, prelude::*};
use fatfs::{FileSystem, FsOptions};
use fscommon::BufStream;
use std::io::Write;

pub fn clean(target: BlockDevice, interactive: bool, args: CmdClean) -> Result<()> {
    if interactive {
        crate::confirm_prompt(format!(
            "Cleaning partition {target}, do you wish to proceed?",
        ))?;
    }

    let file = target.open(false)?;

    let stream = BufStream::new(file);

    let fs = FileSystem::new(stream, FsOptions::new())?;

    {
        let root_dir = fs.root_dir();
        let link_dir = root_dir.open_dir(LINK_DIR)?;

        // count links ignoring any directories
        let links = link_dir
            .iter()
            .flatten()
            .filter(|x| x.is_file())
            .map(|x| x.file_name())
            .collect::<Vec<_>>();

        if !links.is_empty() {
            for (i, file_name) in links.iter().enumerate() {
                link_dir.remove_entry(file_name)?;
                print!("\rRemoving old links [{}/{}]", i + 1, links.len());
                let _ = std::io::stdout().flush();
            }
            println!();
        }

        // optionally remove all songs
        if args.songs {
            let music_dir = root_dir.open_dir(MUSIC_DIR)?;

            // count files ignoring any directories
            let files = music_dir
                .iter()
                .flatten()
                .filter(|x| x.is_file())
                .map(|x| x.file_name())
                .collect::<Vec<_>>();

            if !files.is_empty() {
                for (i, file_name) in files.iter().enumerate() {
                    music_dir.remove(&file_name)?;
                    print!("\rRemoving song [{}/{}]", i + 1, files.len());
                    let _ = std::io::stdout().flush();
                }
                println!();
            }
        }

        // delete the flag file if present
        let _ = root_dir.remove(DIRTY_FLAG_FILE);
    }

    fs.unmount()?;

    println!("Done!");

    Ok(())
}
