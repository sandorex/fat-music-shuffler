// use crate::{Context, FULL_VERSION, LONG_VERSION, config::Config};
use clap::{Args, Parser, Subcommand};

/// Utility for shuffling MP3 music files for dumb MP3 players
#[derive(Parser, Debug, Clone)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: CliCommands,
}

// impl Target {
//     fn parse(input: &str) -> Result<Self, String> {
//         if let Some(x) = input.strip_prefix("/dev/") {}
//     }
// }

// #[derive(Args, Debug, Clone)]
// pub struct TargetArg {}

#[derive(Args, Debug, Clone)]
pub struct CmdFormat {
    /// Block device or file you want to target
    pub target: String,
}

#[derive(Args, Debug, Clone)]
pub struct CmdShuffle {
    /// Block device or file you want to target
    pub target: String,
}

#[derive(Subcommand, Debug, Clone)]
pub enum CliCommands {
    /// Format target as FAT32
    Format(CmdFormat),

    /// Reshuffle music hardlinks in target
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
