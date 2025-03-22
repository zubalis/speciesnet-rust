use std::path::PathBuf;

use clap::Parser;
use speciesnet::SpeciesNet;
use speciesnet::io::{InstanceList, PredictionList};

/// CLI arguments struct after parsing.
#[derive(Debug, clap::Parser)]
#[command(version, about, long_about = None)]
pub struct CliArguments {
    /// Path of the detector model.
    detector_model: Option<PathBuf>,
    /// Path of the input file.
    #[arg(long, value_name = "PATH")]
    instances_json: PathBuf,
    /// Path of the output file.
    #[arg(long)]
    predictions_json: PathBuf,
    /// Running only the detector (no additional requirements).
    #[arg(long)]
    detector_only: bool,
    /// Running only the classifier (requires detections_json).
    #[arg(long)]
    classifier_only: bool,
    #[arg(long)]
    detections_json: Option<PathBuf>,
    /// Running only the ensemble (requires detections_json and classifications_json).
    #[arg(long)]
    ensemble_only: bool,
    #[arg(long)]
    classifications_json: Option<PathBuf>,
}

fn main() -> Result<(), speciesnet::error::Error> {
    let args = CliArguments::parse();
    println!("args {:?}", args);

    let instances = InstanceList::from_json_file(args.instances_json.to_str().unwrap()).unwrap();
    println!("Got instances...");
    println!("{:#?}", instances);

    if (args.predictions_json.exists()) {
        // TODO: implement resume from existing predictions
        panic!("Output file already exists");
    }

    if args.classifier_only && args.detections_json.is_none() {
        panic!("`--classifier-only` requires `--detections-json PATH`");
    }

    if args.ensemble_only && (args.detections_json.is_none() || args.classifications_json.is_none())
    {
        panic!(
            "`--ensemble-only` requires `--detections-json PATH` and `--classifications-json PATH`"
        );
    }

    let detections: Option<PredictionList> = if !args.classifier_only && !args.ensemble_only {
        None
    } else {
        let detections_json_path = args.detections_json.unwrap();
        if (!detections_json_path.exists()) {
            panic!("`--detections-json PATH` does not exist");
        }
        Some(PredictionList::from_json_file(detections_json_path.to_str().unwrap()).unwrap())
    };
    let classifications: Option<PredictionList> = if !args.ensemble_only {
        None
    } else {
        let classifications_json_path = args.classifications_json.unwrap();
        if (!classifications_json_path.exists()) {
            panic!("`--classifications-json PATH` does not exist");
        }
        Some(PredictionList::from_json_file(classifications_json_path.to_str().unwrap()).unwrap())
    };

    println!("Detections...");
    println!("{:#?}", detections);

    println!("Classifications...");
    println!("{:#?}", classifications);

    let detector_model_path = args
        .detector_model
        .unwrap_or(PathBuf::from("assets/model/md_v5a.0.0_traced.pt"));

    let _detector_model = SpeciesNet::new(detector_model_path)?;

    Ok(())
}
