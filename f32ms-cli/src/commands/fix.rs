use crate::cli::CmdFix;
use crate::prelude::*;
use crate::util::find_mp3_files;
use std::{
    io::Write,
    path::{Path, PathBuf},
};

pub fn process_file(input: &Path, output: &Path, overwrite: bool) -> Result<()> {
    // skip existing files if not overwriting
    if !overwrite && output.exists() {
        return Ok(());
    }

    // TODO Add volume modification filter thingy
    let cmd = std::process::Command::new("ffmpeg")
        .args([
            "-nostdin",
            "-y",
            "-i",
            input.to_str().unwrap(),
            "-map",
            "a",
            // apply the replaygain
            "-filter:a",
            "volume=replaygain=track",
            // remove metadata
            "-map_metadata",
            "-1",
            // remove silence at beginning and end
            "-af",
            "silenceremove=start_periods=1:start_duration=1:start_threshold=-60dB:detection=peak,aformat=dblp,areverse,silenceremove=start_periods=1:start_duration=1:start_threshold=-60dB:detection=peak,aformat=dblp,areverse",
        ])
        .arg(output.to_str().unwrap())
        .output()
        .with_context(|| anyhow!("Could not run ffmpeg"))?;

    if !cmd.status.success() {
        bail!(
            "Failed to process {input:?}, exit code {:?}",
            cmd.status.code()
        )
    }

    // TODO check if duration has not reduced by more than 5%
    let orig = mp3_duration::from_path(input)
        .with_context(|| anyhow!("Could not read duration from input {input:?}"))?;

    let out = mp3_duration::from_path(output)
        .with_context(|| anyhow!("Could not read duration from output {output:?}"))?;

    // the output may be entirely broken
    if out.as_secs_f64() == 0.0 {
        println!("\rWarning: output file {output:?} has length of 0 seconds!");
        return Ok(());
    }

    let diff = orig.as_secs_f64() - out.as_secs_f64();
    let procentage = (diff / out.as_secs_f64()) * 100.0;

    // check if file duration has changed a lot
    if procentage >= 80.00 {
        println!("\rWarning: output file {output:?} duration was reduced by over 80%!");
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
