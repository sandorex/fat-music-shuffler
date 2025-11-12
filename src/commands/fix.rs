use crate::cli::CmdFix;
use crate::prelude::*;
use crate::util::find_mp3_files;
use std::{
    io::Write,
    path::{Path, PathBuf},
};

pub fn process_file(input: &Path, output: &Path, overwrite: bool) -> Result<()> {
    const THRESHOLD: &str = "0";

    // skip existing files
    if !overwrite && output.exists() {
        return Ok(());
    }

    let cmd = std::process::Command::new("ffmpeg")
        .args([
            "-nostdin",
            "-y",
            "-i",
            input.to_str().unwrap(),
            "-map",
            "a",
            "-map_metadata",
            "-1",
            "-af",
        ])
        .arg(format!("silenceremove=start_periods=1:start_threshold={THRESHOLD}:start_silence=0:stop_periods=1:stop_threshold={THRESHOLD}:stop_silence=0:detection=peak"))
        .arg(output.to_str().unwrap())
        .output()
        .with_context(|| anyhow!("Could not run ffmpeg"))?;

    if !cmd.status.success() {
        bail!(
            "Failed to process {input:?}, exit code {:?}",
            cmd.status.code()
        )
    }

    Ok(())
}

pub fn fix(interactive: bool, args: CmdFix) -> Result<()> {
    println!("Scanning for files..");
    let mut files: Vec<PathBuf> = vec![];

    for path in args.paths {
        find_mp3_files(&mut files, path.clone())
            .with_context(|| anyhow!("Error scanning {path:?}"))?;
    }

    if interactive {
        crate::confirm_prompt(format!(
            "Fixing {} MP3 files, do you wish to proceed?",
            files.len()
        ))?;
    }

    if args.output.try_exists().unwrap_or(false) {
        if !args.output.is_dir() {
            bail!("Output path is not a directory");
        }
    } else {
        std::fs::create_dir_all(&args.output)
            .with_context(|| anyhow!("Failed to create output directory {:?}", args.output))?;
    }

    for (i, path) in files.iter().enumerate() {
        // update progress
        print!("\rProcessing files [{}/{}]", i + 1, files.len());
        let _ = std::io::stdout().flush();

        let output = args.output.join(path.file_name().unwrap());
        process_file(path, output.as_path(), args.overwrite)?;
    }
    println!();

    println!("Done!");

    Ok(())
}
