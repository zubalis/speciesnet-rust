use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::error::Error;

#[cfg(feature = "download-model")]
pub mod download_model;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModelType {
    AlwaysCrop,
    FullImage,
}

/// Struct containing the model's information and where the files are.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ModelInfo {
    /// Version of the loaded model.
    version: String,
    /// Type of the loaded model.
    #[serde(rename = "type")]
    model_type: ModelType,
    /// Path of where the classifier model is.
    classifier: PathBuf,
    /// Path of where the classifier labels is.
    classifier_labels: PathBuf,
    /// Path of where the detector model is.
    detector: PathBuf,
    /// Path of the taxonomy file.
    taxonomy: PathBuf,
    /// Path of the geofence file.
    geofence: PathBuf,
}

impl ModelInfo {
    /// Constructs the [`ModelInfo`] instance from a given folder of an extracted path of the
    /// model.
    pub fn from_path<P>(folder: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let info_json_string = read_to_string(folder.as_ref().join("info.json"))?;
        let info_json: Self = serde_json::from_str(&info_json_string)?;

        let classifier_path = folder.as_ref().join(info_json.classifier());
        let classifier_labels_path = folder.as_ref().join(info_json.classifier_labels());
        let detector_path = folder.as_ref().join(info_json.detector());
        let taxonomy_path = folder.as_ref().join(info_json.taxonomy());
        let geofence_path = folder.as_ref().join(info_json.geofence());

        Ok(Self {
            version: info_json.version,
            model_type: info_json.model_type,
            classifier: classifier_path,
            classifier_labels: classifier_labels_path,
            detector: detector_path,
            taxonomy: taxonomy_path,
            geofence: geofence_path,
        })
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn model_type(&self) -> ModelType {
        self.model_type
    }

    pub fn classifier(&self) -> &Path {
        &self.classifier
    }

    pub fn classifier_labels(&self) -> &Path {
        &self.classifier_labels
    }

    pub fn detector(&self) -> &Path {
        &self.detector
    }

    pub fn taxonomy(&self) -> &Path {
        &self.taxonomy
    }

    pub fn geofence(&self) -> &Path {
        &self.geofence
    }
}
