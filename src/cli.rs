use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use humantime::Duration;

/// Utility for shuffling MP3 music files for dumb MP3 players
#[derive(Parser, Debug, Clone)]
#[command(author, version, about)]
pub struct Cli {
    /// Target block device, leave empty for prompt
    // hide the flag on windows cause its useless
    #[cfg_attr(target_os = "windows", clap(skip))]
    pub target: Option<String>,

    /// Shows all disk devices instead of only removable ones (SD, Flash drive..)
    #[clap(long)]
    pub show_all_disks: bool,

    #[command(subcommand)]
    pub cmd: CliCommands,
}

#[derive(Args, Debug, Clone)]
pub struct CmdShuffle {
    /// Repeats all songs until they fill up at minimum this amount of time
    ///
    /// This is a hack to implement quasi-shuffle by repeating everything but
    /// in different predefined order
    ///
    /// This feature can create A LOT of links so beware it can take a while
    #[clap(long)]
    pub repeat_fill: Option<Duration>,
}

#[derive(Args, Debug, Clone)]
pub struct CmdClean {
    /// Partition to clean
    pub target: String,
}

#[derive(Args, Debug, Clone)]
pub struct CmdImport {
    /// Files to import
    #[clap(num_args = 1..)]
    pub files: Vec<PathBuf>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum CliCommands {
    /// Formats device/partition (ERASES ALL DATA!)
    ///
    /// In case target is a device block file then it formats it to contain a
    /// single FAT32 partition with MBR/BIOS partition table
    #[cfg_attr(target_os = "windows", clap(skip))]
    Format,

    /// Shuffle imported music
    Shuffle(CmdShuffle),

    /// Cleans up the links making it editable directly
    Clean(CmdClean),

    /// Copies all files so no mounting is required, does not reshuffle music
    Import(CmdImport),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert()
    }
}
