use ndarray::{Array1, Array4, Ix2};
use ort::session::Session;
use ort::session::builder::GraphOptimizationLevel;
use ort::value::Tensor;
use std::path::Path;
use std::sync::Arc;

pub mod classifier;
pub mod error;
pub mod image;
pub mod input;

use crate::error::Error;

#[derive(Debug, Clone)]
pub struct SpeciesNetClassifier {
    model: Arc<Session>,
}

impl SpeciesNetClassifier {
    /// Create classifier from given config
    pub fn new<P>(model_path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let cpus = num_cpus::get();
        let session = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(cpus)?
            .commit_from_file(model_path)?;
        Ok(Self {
            model: Arc::new(session),
        })
    }

    /// run a classification from given input
    pub fn classify(&self, input_tensor: Array4<f32>) -> Result<Array1<f32>, Error> {
        let outputs = self
            .model
            .run(ort::inputs!["input" => Tensor::from_array(input_tensor)?]?)?;
        let output: Array1<f32> = outputs
            .get("dense")
            .unwrap()
            .try_extract_tensor::<f32>()?
            .into_dimensionality::<Ix2>()?
            .row(0) // to get the scores
            .into_owned();
        Ok(output)
    }
}
