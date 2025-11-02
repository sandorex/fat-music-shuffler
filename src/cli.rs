// use crate::{Context, FULL_VERSION, LONG_VERSION, config::Config};
use clap::{Args, Parser, Subcommand};

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
    /// Partition where to shuffle music
    pub target: String,
}

#[derive(Subcommand, Debug, Clone)]
pub enum CliCommands {
    Format(CmdFormat),

    /// Reshuffle music hardlinks
    Shuffle(CmdShuffle),
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
