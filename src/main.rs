use std::str::FromStr;

use clap::Parser;
use colored::Colorize;

#[derive(Parser, Clone, Debug)]
#[command(author, version, about, long_about = None)]
// #[clap(args_conflicts_with_subcommands = true)]
pub struct Cli {
    #[command(subcommand)]
    pub subcommand: Subcommand,
    #[arg(short, long)]
    source: Option<String>,
}

#[derive(Debug, clap::Subcommand, Clone)]
pub enum Subcommand {
    Export(ExportArgs),
    Up,
    Down,
}

#[derive(Debug, clap::Args, Clone)]
pub struct ExportArgs {
    #[arg(short, long)]
    platform: Option<String>,
}

// TODO: deal with source
fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .format_target(false)
        .format_timestamp(None)
        .init();
    let cli = Cli::parse();
    let source = &cli.source;
    match cli.subcommand {
        Subcommand::Export(ExportArgs { platform }) => {
            export(platform.as_deref(), source.as_deref())
        }
        Subcommand::Up => chnroutes::up(&Default::default())?,
        Subcommand::Down => {
            chnroutes::down(&Default::default())?;
        }
    }
    Ok(())
}

pub fn export(platform: Option<&str>, source: Option<&str>) {
    let target = chnroutes::Target::from_str(platform.unwrap_or_default());
    if let Ok(target) = target {
        target.export_file(&Default::default()).unwrap();
    } else {
        eprint!("Unknown platform. platform must in ");
        ["windows", "mac", "linux", "android", "openvpn"]
            .iter()
            .for_each(|x| eprint!("{}, ", x.green()));
        eprintln!();
        std::process::exit(1);
    }
}
