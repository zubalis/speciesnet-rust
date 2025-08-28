use serde::{Deserialize, Serialize};
use speciesnet::Prediction;

/// The output type of `predictions.json` file.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CliPredictions {
    predictions: Vec<CliPrediction>,
}
impl CliPredictions {
    pub fn new(predictions: Vec<CliPrediction>) -> Self {
        Self { predictions }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CliPrediction {
    #[serde(flatten)]
    prediction: Prediction,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    failures: Option<Vec<String>>,
}

impl CliPrediction {
    pub fn new(prediction: Prediction, failures: Option<Vec<String>>) -> Self {
        Self {
            prediction,
            failures,
        }
    }
}

impl
    From<(
        speciesnet::Instance,
        Result<Prediction, speciesnet::error::Error>,
    )> for CliPrediction
{
    fn from(
        (instance, result): (
            speciesnet::Instance,
            Result<Prediction, speciesnet::error::Error>,
        ),
    ) -> Self {
        match result {
            Ok(prediction) => Self::new(prediction, None),
            Err(e) => {
                let prediction = Prediction::new(instance.file_path().to_path_buf());
                let failure = match e {
                    speciesnet::error::Error::DetectorError(_) => "DETECTOR",
                    speciesnet::error::Error::ClassifierError(_) => "CLASSIFIER",
                    speciesnet::error::Error::EnsembleError(_) => "GEOLOCATION",
                    _ => "UNKNOWN",
                };
                Self::new(prediction, Some(vec![failure.to_string()]))
            }
        }
    }
}

impl
    From<(
        Vec<speciesnet::Instance>,
        Vec<Result<Prediction, speciesnet::error::Error>>,
    )> for CliPredictions
{
    fn from(
        (instances, results): (
            Vec<speciesnet::Instance>,
            Vec<Result<Prediction, speciesnet::error::Error>>,
        ),
    ) -> Self {
        let predictions = instances
            .into_iter()
            .zip(results)
            .map(CliPrediction::from)
            .collect();
        Self::new(predictions)
    }
}
