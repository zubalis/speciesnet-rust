use std::path::PathBuf;

use serde::Deserialize;

/// The type of the `instances.json` file.
#[derive(Debug, Deserialize)]
pub struct Instances {
    pub instances: Vec<Instance>,
}

/// The type of each instance of image that will be passed in to the model.
#[derive(Debug, Deserialize)]
pub struct Instance {
    /// File path of the given image which is relative to where the instances json file resides.
    pub filepath: PathBuf,
    /// Country code in ISO-639-1 to do ensemble on.
    pub country: Option<String>,
    pub admin1_region: Option<String>,
}

impl Instance {
    pub fn from_path_buf(filepath: PathBuf) -> Self {
        Self {
            filepath,
            country: None,
            admin1_region: None,
        }
    }
}
