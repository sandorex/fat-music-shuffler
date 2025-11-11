use crate::cli::CmdImport;
use crate::util::BlockDevice;
use crate::{MUSIC_DIR, prelude::*};
use fatfs::{FileSystem, FsOptions};
use fscommon::BufStream;
use std::io::Write;
use std::path::PathBuf;

fn list_files(vec: &mut Vec<PathBuf>, path: PathBuf) -> std::io::Result<()> {
    if path.is_dir() {
        let paths = std::fs::read_dir(&path)?;
        for path_result in paths {
            let full_path = path_result?.path();
            list_files(vec, full_path)?;
        }
    } else {
        // only collect MP3 files
        if let Some(ext) = path.extension() {
            if ext == "mp3" {
                vec.push(path);
            }
        }
    }

    Ok(())
}

pub fn import(target: BlockDevice, interactive: bool, args: CmdImport) -> Result<()> {
    println!("Scanning for files..");
    let mut files = vec![];

    for path in args.paths {
        list_files(&mut files, path.clone()).with_context(|| anyhow!("Error scanning {path:?}"))?;
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
    let root_dir = fs.root_dir();

    let music_dir = root_dir.create_dir(MUSIC_DIR)?;

    for (i, path) in files.iter().enumerate() {
        // update progress
        print!("\rCopying files [{}/{}]", i + 1, files.len());
        let _ = std::io::stdout().flush();

        // add the require extension
        let name = format!(
            "{}{}",
            path.file_name().unwrap().to_string_lossy(),
            crate::MUSIC_EXT
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

        std::io::copy(&mut file, &mut fat_file)
            .with_context(|| anyhow!("Failed to import {path:?}"))?;
    }
    println!();

    println!("Done!");

    Ok(())
}
