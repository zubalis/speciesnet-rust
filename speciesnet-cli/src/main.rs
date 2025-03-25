use std::{fs::File, io::BufWriter, path::PathBuf};

use clap::{Args, CommandFactory, Parser, error::ErrorKind};
use inputs::prepare_image_inputs;
use log::debug;
use speciesnet::SpeciesNet;
use speciesnet_core::prediction::Predictions;

mod file_extension;
mod inputs;

#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
struct InputType {
    /// Path of instances.json file input.
    #[arg(long)]
    instances_json: Option<PathBuf>,
    /// Path of filepaths to be added into the model.
    #[arg(long)]
    filepaths: Vec<PathBuf>,
    /// Path of the filepaths.txt file.
    #[arg(long)]
    filepaths_txt: Option<PathBuf>,
    /// Path of folders to run inference on.
    #[arg(long)]
    folders: Vec<PathBuf>,
    /// Path of folders.txt file.
    #[arg(long)]
    folders_txt: Option<PathBuf>,
}

#[derive(Debug, Args)]
#[group(required = false, multiple = false)]
pub struct AdditionalConfiguration {
    /// Path of detections.json file to put in the model.
    #[arg(long)]
    detections_json: Option<PathBuf>,
    /// Path of classifications.json file to put in the model.
    #[arg(long)]
    classifications_json: Option<PathBuf>,
}

#[derive(Debug, Args)]
#[group(required = false, multiple = true)]
pub struct RunType {
    /// Running only the detector.
    #[arg(long)]
    detector_only: bool,
    /// Running only the classifier (requires detections-json).
    #[arg(long)]
    classifier_only: bool,
    /// Running only the ensembler (requires detections-json and classifications-json).
    #[arg(long)]
    ensemble_only: bool,
    /// Enables geofence while running the ensembler.
    #[arg(long)]
    geofence: bool,
}

#[derive(Debug, Parser)]
pub struct CliArguments {
    #[command(flatten)]
    input_type: InputType,
    #[command(flatten)]
    run_type: RunType,
    #[command(flatten)]
    additional_config: AdditionalConfiguration,
    /// The path of the detector model.
    #[arg(long)]
    detector_model: PathBuf,
    /// Output predictions.json file path of the predictions result.
    #[arg(long)]
    predictions_json: PathBuf,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args = CliArguments::parse();
    let mut cmd = CliArguments::command();

    // Stops the run if predictions-json exists.
    if args.predictions_json.exists() {
        cmd.error(
            ErrorKind::ValueValidation,
            format!(
                "Predictions file at {:?} already exists.",
                args.predictions_json.display()
            ),
        )
        .exit();
    }

    // Classifier can only be run when detections-json is provided.
    if args.run_type.classifier_only && args.additional_config.detections_json.is_none() {
        cmd.error(
            ErrorKind::MissingRequiredArgument,
            "When --classifier_only is set, --detections-json must be provided.",
        )
        .exit();
    }

    // Ensemble can only be run when detections-json and classifications-json is provided.
    if args.run_type.ensemble_only
        && (args.additional_config.detections_json.is_none()
            || args.additional_config.classifications_json.is_none())
    {
        cmd.error(
            ErrorKind::MissingRequiredArgument,
            "When --ensemble-only is set, --classifications-json and --detections-json must be provided."
        ).exit();
    }

    // Parse the input files into list of files.
    let images = prepare_image_inputs(&args.input_type)?;
    let speciesnet = SpeciesNet::new(&args.detector_model)?;

    if args.run_type.detector_only {
        let detector_results = speciesnet.detect(&images)?;
        let predictions = Predictions::from(detector_results);

        debug!(
            "Saving the detected results to {}.",
            args.predictions_json.display()
        );

        let writer = BufWriter::new(File::create(&args.predictions_json)?);
        serde_json::to_writer(writer, &predictions)?;

        debug!(
            "Predictions file has been successfully saved to {}.",
            args.predictions_json.display()
        );
    }

    debug!("Program finished.");
    Ok(())
}
