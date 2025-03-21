use std::path::PathBuf;

use clap::Parser;
use speciesnet::SpeciesNet;

/// CLI arguments struct after parsing.
#[derive(Debug, clap::Parser)]
#[command(version, about, long_about = None)]
pub struct CliArguments {
    /// Path of the detector model.
    detector_model: PathBuf,
    /// Path of the instances.json file.
    #[arg(long, value_name = "PATH")]
    instances_json: Option<PathBuf>,
    /// Running only the detector.
    #[arg(long)]
    detector_only: bool,
    /// Path of the output detections.json file.
    #[arg(long)]
    detections_json: Option<PathBuf>,
}

fn main() -> Result<(), speciesnet::error::Error> {
    let args = CliArguments::parse();

    let model = SpeciesNet::new(&args.detector_model)?;

    println!("args {:?}", args);

    Ok(())
}
