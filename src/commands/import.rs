use crate::cli::CmdImport;
use crate::util::{BlockDevice, find_mp3_files};
use crate::{MUSIC_DIR, MUSIC_EXT, prelude::*};
use fatfs::{FileSystem, FsOptions};
use fscommon::BufStream;
use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;

pub fn import(target: BlockDevice, interactive: bool, args: CmdImport) -> Result<()> {
    println!("Scanning for files..");
    let mut files: Vec<PathBuf> = vec![];

    for path in args.paths {
        find_mp3_files(&mut files, path.clone())
            .with_context(|| anyhow!("Error scanning {path:?}"))?;
    }

    if interactive {
        crate::confirm_prompt(format!(
            "Importing {} MP3 files, do you wish to proceed?",
            files.len()
        ))?;
    }

    let file = target.open(false)?;
    let stream = BufStream::new(file);
    let fs = FileSystem::new(stream, FsOptions::new())?;

    {
        let root_dir = fs.root_dir();
        let music_dir = root_dir.create_dir(MUSIC_DIR)?;

        for (i, path) in files.iter().enumerate() {
            // update progress
            print!("\rCopying files [{}/{}]", i + 1, files.len());
            let _ = std::io::stdout().flush();

            let name = format!(
                "{}{MUSIC_EXT}",
                path.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .chars()
                    // only allow ascii characters
                    .filter(|c| c.is_ascii())
                    .collect::<String>(),
            );

            let mut file = std::fs::OpenOptions::new()
                .read(true)
                .write(false)
                .open(path)
                .with_context(|| anyhow!("Failed to open file {path:?}"))?;

            // do not overwrite as that could break hardlinks and corrupt the filesystem in the process
            match music_dir.open_file(&name) {
                Ok(_) => {
                    println!("\rSkipping {name:?} already exists");
                    continue;
                }
                Err(fatfs::Error::NotFound) => {}
                Err(err) => bail!(err),
            }

            let mut fat_file = music_dir
                .create_file(&name)
                .with_context(|| anyhow!("Failed to create file {name:?}"))?;

            let mut buf_file = BufReader::new(&mut file);
            let mut buf_fat_file = BufWriter::new(&mut fat_file);

            std::io::copy(&mut buf_file, &mut buf_fat_file)
                .with_context(|| anyhow!("Failed to copy {path:?}"))?;
        }
    }

    // NOTE without this the hardlinks wont play on the mp3 player!
    fs.unmount()?;

    println!("\nDone!");

    Ok(())
}
