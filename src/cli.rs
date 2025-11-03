use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use humantime::Duration;

/// Utility for shuffling MP3 music files for dumb MP3 players
#[derive(Parser, Debug, Clone)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: CliCommands,
}

#[derive(Args, Debug, Clone)]
pub struct CmdFormat {
    /// Partition to format
    pub target: String,
}

#[derive(Args, Debug, Clone)]
pub struct CmdShuffle {
    /// Partition with the music
    pub target: String,

    /// Repeat the songs until it is this long
    #[clap(long, default_value = "3 days")]
    pub fill_duration: Duration,
}

#[derive(Args, Debug, Clone)]
pub struct CmdClean {
    /// Partition to clean
    pub target: String,
}

#[derive(Args, Debug, Clone)]
pub struct CmdImport {
    /// Partition to import music into
    pub target: String,

    /// MP3 Files
    #[clap(required = true, num_args = 1..)]
    pub files: Vec<PathBuf>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum CliCommands {
    // TODO like print how long will it take to repeat whole playlist
    // / Prints information about current playlist
    // Info,
    /// Formats partition as FAT32 (ERASES ALL DATA!)
    Format(CmdFormat),

    /// Shuffle music
    Shuffle(CmdShuffle),

    /// Cleans up the links making it editable directly
    Clean(CmdClean),

    /// Import music
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
