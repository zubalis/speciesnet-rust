//! ## SpeciesNet Rust
//!
//! The home of the ensemble for running [SpeciesNet] models in Rust.
//!
//! ## Setup
//!
//! Run
//!
//! ```bash
//! cargo add --git https://github.com/zubalis/speciesnet-rust.git --path speciesnet
//! cargo add --git https://github.com/zubalis/speciesnet-rust.git --path core
//! cargo add ort@=2.0.0-rc.9 -F download-binaries
//! ```
//!
//! to your program to get the lastest version of speciesnet.
//!
//! ## Cargo features
//!
//! - `download-model`, enabled by default, this allows you to run [`SpeciesNet::new`] to
//!   initialize the ensemble, which will download the default model from the internet.
//!
//! ## Model setup
//!
//! SpeciesNet Rust ensemble is using [ort] to run the model, which means the detector model and
//! the classifier model needs to be converted to [ONNX] before this program can be run. We have a
//! separate repository for generating SpeciesNet Rust compatible models in [zubalis/speciesnet-onnx].
//! You can grab the model files from there, extract it to a directory and use
//! [`SpeciesNet::from_model_folder`] to initialize the model from that folder.
//!
//! ## Examples
//!
//! Initializing speciesnet with a custom extracted model folder.
//!
//! ```rust
//! use speciesnet::SpeciesNet;
//!
//! let speciesnet = SpeciesNet::from_model_folder("./speciesnet-v4a/")?;
//! ```
//!
//! Running the entire pipeline (detector + classifier + ensemble).
//!
//! ```rust
//! use std::path::PathBuf;
//!
//! use speciesnet_core::io::Instance;
//! use speciesnet::SpeciesNet;
//!
//! let instances = vec![
//!     Instance::from_path_buf(PathBuf::from("./img1.jpeg")),
//!     Instance::from_path_buf(PathBuf::from("./img2.jpeg"))
//! ];
//!
//! let speciesnet = SpeciesNet::new()?;
//! let detections = speciesnet.predict(&instances)?;
//! ```
//!
//! Running the detector pipeline.
//!
//! ```rust
//! use std::path::PathBuf;
//!
//! use speciesnet_core::io::Instance;
//! use speciesnet::SpeciesNet;
//!
//! let instances = vec![
//!     Instance::from_path_buf(PathBuf::from("./img1.jpeg")),
//!     Instance::from_path_buf(PathBuf::from("./img2.jpeg"))
//! ];
//!
//! let speciesnet = SpeciesNet::new()?;
//! let detections = speciesnet.detect(&instances)?;
//! ```
//!
//! The returned detections is in the format of [Prediction] vector, which is the same for all
//! apis.
//!
//! Running the classifier pipeline.
//!  
//! ```rust
//! use std::path::PathBuf;
//!
//! use speciesnet_core::io::Instance;
//! use speciesnet::SpeciesNet;
//!
//! let instances = vec![
//!     Instance::from_path_buf(PathBuf::from("./img1.jpeg")),
//!     Instance::from_path_buf(PathBuf::from("./img2.jpeg"))
//! ];
//!
//! let speciesnet = SpeciesNet::new()?;
//! let classifications = speciesnet.classify(&instances)?;
//! ```
//!
//! Running the ensemble and geofence of the pipeline.
//!
//! NOTE: This function differs from other functions where it operates on each instance of
//! prediction, instead of taking the vector of predictions or instance like other API.
//!
//! ```rust
//! use std::path::PathBuf;
//!
//! use speciesnet_core::{
//!     classifier::ClassificationBundle,
//!     detector::{BoundingBox, Category, Detection},
//! };
//! use speciesnet::SpeciesNet;
//!
//! let instances_json_path = "./instances.json";
//! let detector_file_path = "./output_detector.json";
//! let classifier_file_path = "./output_classifier.json";
//!
//! let speciesnet = SpeciesNet::new()?;
//! let ensembles = speciesnet.ensemble(
//!     instances_json_path,
//!     detector_file_path,
//!     classifier_file_path,
//! )?;
//! ```
//!
//! [SpeciesNet]: https://www.kaggle.com/models/google/speciesnet
//! [Prediction]: speciesnet_core::io::Prediction
//! [ONNX]: https://onnx.ai
//! [zubalis/speciesnet-onnx]: https://github.com/zubalis/speciesnet-onnx
//! [ort]: https://docs.rs/ort

pub mod error;
pub mod model_info;
pub mod speciesnet;

pub use speciesnet::SpeciesNet;
