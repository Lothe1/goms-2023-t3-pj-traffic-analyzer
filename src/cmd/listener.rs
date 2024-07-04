use clap::{command, Parser};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    // #[clap(subcommand)]
    // pub command: Commands,
    /// Port to listen for packets
    #[clap(short, long, default_value = None, short = 'P')]
    pub port: u128,
}