//! ## SpeciesNet CLI
//!
//! `speciesnet-cli` is a CLI for running a cameratrapai ensemble written in Rust.
//!
//! ### Installation
//!
//! #### Cargo
//!
//! ```bash
//! cargo install --git https://github.com/zubalis/speciesnet-rust.git --path speciesnet-cli
//! ```
//!
//! ### Cargo features
//!
//! - `download-model`, enabled by default, downloads the [ONNX](https://onnx.ai)-converted detector and classifier model automatically to be used, you will have to manually pass the path to the extracted model folder in order to run the program if this feature is turned off.
//!
//! ### Usage
//!
//! The CLI is designed to be pretty similar to how [google/cameratrapai](https://github.com/google/cameratrapai) is, except there are some differences as we're working towards the python equivalent.
//!
//! - The input instance file only support `filepath`, `country`, and `admin1_region` keys.
//! - The classifier output only support the `classes`, and `scores` key.
//! - Other keys of `predictions` are still not supported on both reading from and writing to files.
//! - The Rust version does not override or edit an existing `predictions.json` file, if one is found when supplied using `--predictions-json`, CLI will error saying the file already existed.
//! - The CLI flag `--country` and `--admin1-region` currently does nothing to the input.
//!
//! below is the examples of running the ensemble using speciesnet compared to cameratrapai.
//!
//! #### Running only the detector
//!
//! ```bash
//! # cameratrapai.
//! python3 -m speciesnet.scripts.run_model --instance_json ./instance.json --predictions_json ./predictions.json --detector_only
//!
//! # speciesnet-rust.
//! speciesnet-cli --instance-json ./instance.json --predictions-json ./predictions.json --detector-only
//! ```
//!
//! #### Running only the classifier
//!
//! ```bash
//! # cameratrapai.
//! python3 -m speciesnet.scripts.run_model --instance_json ./instance.json --predictions_json ./predictions.json --classifier_only
//!
//! # speciesnet-rust.
//! speciesnet-cli --instance-json ./instance.json --predictions-json ./predictions.json --classifier-only
//! ```
//!
//! #### Running only the ensemble
//!
//! ```bash
//! # cameratrapai.
//! python3 -m speciesnet.scripts.run_model --instance_json ./instance.json --classifications_json ./output_classifier.json --detections_json ./output_detector.json --predictions_json ./predictions.json --ensemble_only
//!
//! # speciesnet-rust.
//! speciesnet-cli --instance-json ./instance.json --classifications-json ./output_classifier.json --detections-json ./output_detector.json --predictions-json ./predictions.json --ensemble-only
//! ```
//!
//! #### Running the whole inference pipeline
//!
//! ```bash
//! # cameratrapai.
//! python3 -m speciesnet.scripts.run_model --instance_json ./instance.json --predictions_json ./predictions.json
//!
//! # speciesnet-rust.
//! speciesnet-cli --instance-json ./instance.json --predictions-json ./predictions.json
//! ```

use std::{fs::File, io::BufWriter, path::PathBuf};

use clap::{Args, CommandFactory, Parser, error::ErrorKind};
use inputs::prepare_image_inputs;
use speciesnet::SpeciesNet;
use speciesnet_core::io::Predictions;
use tracing::info;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

mod file_extension;
mod inputs;

/// The name of the environment variable that can be set to specify the log level of speciesnet.
const SPECIESNET_LOG_ENV_NAME: &str = "SPECIESNET_LOG";

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
#[group(required = false, multiple = true)]
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
    /// Output predictions.json file path of the predictions result.
    #[arg(long)]
    predictions_json: PathBuf,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_env(SPECIESNET_LOG_ENV_NAME)
                .unwrap_or_else(|_| "debug,ort=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_ansi(true))
        .init();

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
    let speciesnet = SpeciesNet::new()?;

    if args.run_type.detector_only {
        let detector_results = speciesnet.detect(&images)?;
        let predictions = Predictions::from(detector_results);

        info!(
            "Saving the detected results to {}.",
            args.predictions_json.display()
        );

        let writer = BufWriter::new(File::create(&args.predictions_json)?);
        serde_json::to_writer(writer, &predictions)?;

        info!(
            "Predictions file has been successfully saved to {}.",
            args.predictions_json.display()
        );
    }

    if args.run_type.classifier_only {
        let output_detection_path = args.additional_config.detections_json.clone();
        let classifier_results = speciesnet.classify(&output_detection_path.unwrap())?; // assumed labels is in the same folder as model
        let predictions = Predictions::from(classifier_results);

        info!(
            "Saving the classified results to {}.",
            args.predictions_json.display()
        );

        let writer = BufWriter::new(File::create(&args.predictions_json)?);
        serde_json::to_writer(writer, &predictions)?;

        info!(
            "Predictions file has been successfully saved to {}.",
            args.predictions_json.display()
        );
    }

    if args.run_type.ensemble_only {
        let instances_json_path = args.input_type.instances_json.clone();
        let output_detection_path = args.additional_config.detections_json.clone();
        let output_classification_path = args.additional_config.classifications_json.clone();
        let ensemble_results = speciesnet.ensemble(
            &instances_json_path.unwrap(),
            &output_detection_path.unwrap(),
            &output_classification_path.unwrap(),
        )?;
        let predictions = Predictions::from(ensemble_results);

        info!(
            "Saving the classified results to {}.",
            args.predictions_json.display()
        );

        let writer = BufWriter::new(File::create(&args.predictions_json)?);
        serde_json::to_writer(writer, &predictions)?;

        info!(
            "Predictions file has been successfully saved to {}.",
            args.predictions_json.display()
        );
    }

    // Performs full inference when none of the options are set.
    if !args.run_type.detector_only
        && !args.run_type.classifier_only
        && !args.run_type.ensemble_only
    {
        let full_results = speciesnet.predict(&images)?;
        let predictions = Predictions::from(full_results);

        info!(
            "Saving the detected results to {}.",
            args.predictions_json.display()
        );

        let writer = BufWriter::new(File::create(&args.predictions_json)?);
        serde_json::to_writer_pretty(writer, &predictions)?;

        info!(
            "Predictions file has been successfully saved to {}.",
            args.predictions_json.display()
        );
    }

    info!("Program finished.");
    Ok(())
}
