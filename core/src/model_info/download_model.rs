use std::{
    fs::{File, create_dir_all},
    io::{BufWriter, copy},
    path::PathBuf,
};

use directories::BaseDirs;
use tracing::info;
use zip::ZipArchive;

use crate::error::Error;

use super::ModelInfo;

/// The directory for storing the downloaded model.
const MODEL_DIRECTORY: &str = "speciesnet-rust/models/";
/// The name of the folder for storing the downloaded model.
const DEFAULT_MODEL_FOLDER: &str = "speciesnet-onnx-v4.0.0a";
/// The file name of the default model.
const DEFAULT_MODEL_FILE_NAME: &str = "speciesnet-onnx-v4.0.0a.zip";
/// The url of the default model.
const DEFAULT_MODEL_URL: &str =
    "https://drive.usercontent.google.com/download?id=1dAGnnJvOiNku6i2Zv82p0Rzidtsk02fy&confirm";

impl ModelInfo {
    /// Constructs the [`ModelInfo`] instance from a default model url, this function will download the
    /// file from the given url, then unzips it and put at the `speciesnet-rust/models/` folder.
    pub fn from_default_url() -> Result<ModelInfo, Error> {
        let base_dir = BaseDirs::new().ok_or_else(|| Error::BaseDirInitFailed)?;
        let cache_dir = base_dir.cache_dir();

        info!("Cache directory is {}.", cache_dir.display());
        info!(
            "Creating the directory {} for putting the model.",
            MODEL_DIRECTORY
        );

        // make a directory in the retrieved cache folder of the model.
        let model_dir = cache_dir.join(MODEL_DIRECTORY);
        create_dir_all(&model_dir)?;

        info!(
            "Checking if the model has been downloaded at {}.",
            model_dir.join(DEFAULT_MODEL_FOLDER).display()
        );

        // check if the model folder exists, or not.
        let possible_model_path = model_dir.join(DEFAULT_MODEL_FOLDER);

        if possible_model_path.exists() {
            return ModelInfo::from_path(possible_model_path);
        }

        info!("Downloading the model from {}", DEFAULT_MODEL_URL);

        // download the model from the url.
        let response = ureq::get(DEFAULT_MODEL_URL).call()?;

        if response.status() != 200 {
            return Err(Error::RequestFailed(response.status().as_u16()));
        }

        let (_, body) = response.into_parts();
        let mut body_reader = body.into_reader();

        // This block forces a drop of the writer.
        {
            let model_zip_file_write = File::create(model_dir.join(DEFAULT_MODEL_FILE_NAME))?;
            let mut writer = BufWriter::new(model_zip_file_write);

            copy(&mut body_reader, &mut writer)?;
        }

        info!(
            "Unzipping the contents inside {} into {}",
            model_dir.join(DEFAULT_MODEL_FILE_NAME).display(),
            model_dir.join(DEFAULT_MODEL_FOLDER).display(),
        );

        // Unzip the file and put it in the models folder.
        let model_zip_file_read = File::open(model_dir.join(DEFAULT_MODEL_FILE_NAME))?;

        let mut zip_file = ZipArchive::new(model_zip_file_read)?;
        let extract_dir = model_dir.join(DEFAULT_MODEL_FOLDER);
        zip_file.extract(&extract_dir)?;

        ModelInfo::from_path(extract_dir)
    }
}
