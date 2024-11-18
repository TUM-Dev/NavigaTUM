use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Context;
use clap::Parser;
use tracing::debug;
use tracing::trace;
use tracing_log::log::info;
mod telemetry;

/// Convert a cad file from the ifc format to osm
#[derive(Parser, Debug)]
struct Cli {
    ///  name of the floor [`Self::floor_level`]
    #[arg(long, value_name = "NAME")]
    floor_name: Option<String>,
    ///  floor level to assign to most of the structure
    floor_level: i8,
    /// Increase the verbosity. Can be Repeated multiple times
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
    /// path to the file to read
    path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    human_panic::setup_panic!();

    let args = Cli::try_parse()?;
    telemetry::setup_logging(args.verbose);
    debug!("got args: {args:?}");

    let _building_part = parse_ifc(args.path).unwrap();
    info!("parsed ifc");
    Ok(())
}

fn parse_ifc(path: PathBuf)->anyhow::Result<ifc_rs::IFC>{
  let content = std::fs::read_to_string(&path)
          .with_context(|| format!("could not read file `{}`", path.display()))?;
      trace!("read file of length {}", content.len());
      
      ifc_rs::IFC::from_str(&content)
}
