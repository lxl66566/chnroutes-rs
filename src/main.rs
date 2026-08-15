use std::path::PathBuf;

use chnroutes::{Source, Target};
use clap::Parser;
use log::LevelFilter;

#[inline]
pub fn log_init() {
    #[cfg(not(debug_assertions))]
    log_init_with_default_level(LevelFilter::Info);
    #[cfg(debug_assertions)]
    log_init_with_default_level(LevelFilter::Debug);
}

#[inline]
pub fn log_init_with_default_level(level: LevelFilter) {
    _ = pretty_env_logger::formatted_builder()
        .filter_level(level)
        .format_timestamp_secs()
        .filter_module("reqwest", LevelFilter::Info)
        .parse_default_env()
        .try_init();
}

#[derive(Parser, Clone, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub subcommand: Subcommand,
    /// Source to generate ip rules
    #[arg(short, long, value_enum, default_value_t = Source::Apnic)]
    source: Source,
}

#[derive(Debug, clap::Subcommand, Clone)]
pub enum Subcommand {
    /// Export up and down scripts for windows, mac, linux, android, openvpn
    Export(ExportArgs),
    /// Write IP rules to system route table
    Up,
    /// Remove IP rules from system route table
    Down,
}

#[derive(Debug, clap::Args, Clone)]
pub struct ExportArgs {
    /// The platform of script you want to export
    #[arg(short, long, value_enum)]
    platform: Target,
    /// Directory to write the script files into
    #[arg(short, long, default_value = ".")]
    output: PathBuf,
}

#[tokio::main]
async fn main() -> chnroutes::Result<()> {
    log_init();
    let cli = Cli::parse();
    match cli.subcommand {
        Subcommand::Export(args) => {
            args.platform.export_file(&cli.source, &args.output).await?;
        },
        Subcommand::Up => chnroutes::up(&cli.source).await?,
        Subcommand::Down => chnroutes::down(&cli.source).await?,
    }
    Ok(())
}
