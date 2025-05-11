use std::path::{Path, PathBuf};

use serde::Deserialize;

/// The type of the `instances.json` file.
#[derive(Debug, Deserialize, Clone)]
pub struct Instances {
    instances: Vec<Instance>,
}

impl Instances {
    pub fn instances(&self) -> &[Instance] {
        &self.instances
    }
}

/// The type of each instance of image that will be passed in to the model.
#[derive(Debug, Deserialize, Clone)]
pub struct Instance {
    /// File path of the given image which is relative to where the instances json file resides.
    #[serde(rename = "filepath")]
    file_path: PathBuf,
    /// Country code in ISO-639-1 to do ensemble on.
    country: Option<String>,
    admin1_region: Option<String>,
}

impl Instance {
    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    pub fn country(&self) -> Option<&str> {
        self.country.as_deref()
    }

    pub fn admin1_region(&self) -> Option<&str> {
        self.admin1_region.as_deref()
    }

    /// Constructs the [`Instance`] from given input.
    pub fn new(file_path: PathBuf, country: Option<String>, admin1_region: Option<String>) -> Self {
        Self {
            file_path,
            country,
            admin1_region,
        }
    }

    /// Constructs the [`Instance`] from only the given file path with other elements set to
    /// [`Option::None`].
    pub fn from_path_buf(file_path: PathBuf) -> Self {
        Self {
            file_path,
            country: None,
            admin1_region: None,
        }
    }
}
