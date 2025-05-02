use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    BoundingBox, Detection, classification::ClassificationBundle, geofence::GeofenceResult,
};

/// The output type of `predictions.json` file.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Predictions {
    predictions: Vec<Prediction>,
}

impl From<Vec<Prediction>> for Predictions {
    fn from(value: Vec<Prediction>) -> Self {
        Predictions { predictions: value }
    }
}

impl Predictions {
    pub fn new(predictions: Vec<Prediction>) -> Self {
        Self { predictions }
    }

    pub fn predictions(&self) -> &[Prediction] {
        &self.predictions
    }
}

/// The possible output of each predictions found during the run.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Prediction {
    #[serde(rename = "filepath")]
    file_path: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    admin1_region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    detections: Option<Vec<Detection>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    classifications: Option<ClassificationBundle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prediction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prediction_score: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prediction_source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model_version: Option<String>,
}

impl Prediction {
    pub fn new(file_path: PathBuf) -> Self {
        Self {
            file_path,
            country: None,
            admin1_region: None,
            detections: None,
            classifications: None,
            prediction: None,
            prediction_score: None,
            prediction_source: None,
            model_version: None,
        }
    }

    pub fn from_detections(file_path: PathBuf, detections: Vec<Detection>) -> Self {
        Self {
            file_path,
            country: None,
            admin1_region: None,
            detections: Some(detections),
            classifications: None,
            prediction: None,
            prediction_score: None,
            prediction_source: None,
            model_version: None,
        }
    }

    pub fn from_classifications(file_path: PathBuf, classifications: ClassificationBundle) -> Self {
        Self {
            file_path,
            country: None,
            admin1_region: None,
            detections: None,
            classifications: Some(classifications),
            prediction: None,
            prediction_score: None,
            prediction_source: None,
            model_version: None,
        }
    }

    pub fn from_ensemble(
        file_path: PathBuf,
        country: Option<String>,
        admin1_region: Option<String>,
        geofence_result: GeofenceResult,
        detections: Vec<Detection>,
        classifications: ClassificationBundle,
    ) -> Self {
        Self {
            file_path,
            country,
            admin1_region,
            detections: Some(detections),
            classifications: Some(classifications),
            prediction: Some(geofence_result.label().to_string()),
            prediction_score: Some(geofence_result.score()),
            prediction_source: Some(geofence_result.source().to_string()),
            model_version: None,
        }
    }

    pub fn set_file_path(&mut self, path: PathBuf) -> &mut Self {
        self.file_path = path;
        self
    }

    pub fn set_country(&mut self, country: Option<String>) -> &mut Self {
        self.country = country;
        self
    }

    pub fn set_admin1_region(&mut self, admin1_region: Option<String>) -> &mut Self {
        self.admin1_region = admin1_region;
        self
    }

    pub fn set_detections(&mut self, detections: Option<Vec<Detection>>) -> &mut Self {
        self.detections = detections;
        self
    }

    pub fn set_classifications(
        &mut self,
        classifications: Option<ClassificationBundle>,
    ) -> &mut Self {
        self.classifications = classifications;
        self
    }

    pub fn set_prediction(&mut self, prediction: Option<String>) -> &mut Self {
        self.prediction = prediction;
        self
    }

    pub fn set_prediction_score(&mut self, prediction_score: Option<f64>) -> &mut Self {
        self.prediction_score = prediction_score;
        self
    }

    pub fn set_prediction_source(&mut self, prediction_source: Option<String>) -> &mut Self {
        self.prediction_source = prediction_source;
        self
    }

    /// Sets the prediction object's model version to a given value.
    // TODO: Model version can be an enum.
    pub fn set_model_version(&mut self, model_version: Option<String>) -> &mut Self {
        self.model_version = model_version;
        self
    }

    /// Merges 2 [`Prediction`] structs together, where the other [`Prediction`] would override the
    /// initial predictions value if there are values in the [`Some`] variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::{Path, PathBuf};
    ///
    /// use crate::{
    ///     category::Category,
    ///     classification::ClassificationBundle,
    ///     detection::Detection,
    ///     prediction::Prediction,
    /// };
    ///
    /// let mut prediction1 = Prediction::default();
    /// prediction1.set_file_path(PathBuf::from("./abd12333/20241212_121922.jpeg"));
    /// prediction1.set_detections(Some(vec![Detection::new(
    ///     Category::Human,
    ///     0.93,
    ///     BoundingBox::new(0.1, 0.2, 0.3, 0.3),
    /// )]));
    ///
    /// let mut prediction2 = Prediction::default();
    ///
    /// prediction2.set_classifications(Some(ClassificationBundle::new(
    ///     vec![
    ///     "00049ff0-2ffa-4d82-8cf3-c861fbbfa9d5;mammalia;rodentia;muridae;rattus;;rattus species"
    ///         .to_owned(),
    ///     "006959ea-d591-404e-b457-505aaa7d80dc;aves;passeriformes;sittidae;;;sittidae family"
    ///         .to_owned(),
    /// ],
    ///     vec![0.88333, 0.9331],
    /// )));
    ///
    /// prediction1.merge(prediction2);
    ///
    /// assert_eq!(
    ///     prediction1.file_path().to_string_lossy(),
    ///     PathBuf::from("./abd12333/20241212_121922.jpeg").to_string_lossy()
    /// );
    /// assert!(prediction1.detections().is_some());
    /// assert_eq!(
    ///     prediction1
    ///         .detections()
    ///         .unwrap()
    ///         .first()
    ///         .unwrap()
    ///         .category(),
    ///     &Category::Human,
    /// );
    ///
    /// assert_eq!(
    ///     prediction1
    ///         .detections()
    ///         .unwrap()
    ///         .first()
    ///         .unwrap()
    ///         .bounding_box(),
    ///     &BoundingBox::new(0.1, 0.2, 0.3, 0.3)
    /// );
    ///
    /// assert!(prediction1.classifications().is_some());
    /// ```
    pub fn merge(&mut self, other: Self) -> &mut Self {
        if let Some(country) = other.country {
            self.country = Some(country);
        }

        if let Some(admin1) = other.admin1_region {
            self.admin1_region = Some(admin1);
        }

        if let Some(detections) = other.detections {
            self.detections = Some(detections);
        }

        if let Some(classifications) = other.classifications {
            self.classifications = Some(classifications);
        }

        if let Some(prediction) = other.prediction {
            self.prediction = Some(prediction);
        }

        if let Some(prediction_score) = other.prediction_score {
            self.prediction_score = Some(prediction_score);
        }

        if let Some(prediction_source) = other.prediction_source {
            self.prediction_source = Some(prediction_source);
        }

        if let Some(model_version) = other.model_version {
            self.model_version = Some(model_version);
        }

        self
    }

    pub fn detections(&self) -> &Option<Vec<Detection>> {
        &self.detections
    }

    pub fn classifications(&self) -> &Option<ClassificationBundle> {
        &self.classifications
    }

    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    /// Extracts the `id` part of the predicted prediction as a [`Uuid`] from the labels string in format
    /// of.
    ///
    /// ```
    /// id;class;order;family;genus;species;common name
    /// ```
    pub fn prediction_id(&self) -> Option<Uuid> {
        if let Some(prediction) = &self.prediction {
            let id_string = prediction.split(';').next().unwrap();
            let id = uuid::Uuid::try_parse(id_string).unwrap();

            return Some(id);
        };

        None
    }

    pub fn prediction_reference(&self) -> Option<&str> {
        self.prediction.as_deref()
    }

    /// Retrieves the confidence value of the prediction.
    pub fn prediction_score(&self) -> Option<f64> {
        self.prediction_score
    }

    /// Copies the bounding boxes in the detection and returns it as a vector of [`BoundingBox`]es.
    pub fn bounding_boxes(&self) -> Option<Vec<BoundingBox>> {
        self.detections
            .as_ref()
            .map(|det| det.iter().map(|d| *d.bounding_box()).collect::<Vec<_>>())
    }
}
