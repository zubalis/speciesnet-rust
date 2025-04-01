use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use speciesnet_core::instance::Instances;
use tracing::{debug, info};
use walkdir::WalkDir;

use crate::{InputType, file_extension::SUPPORTED_IMAGE_EXTENSIONS};

/// Reads the input from each possible types of input type, works on the given input and puts them
/// in one vector. The value from input type, although is a struct with 5 members but there can
/// only be one input at a time due to clap's guarantee in the command line declaration.
pub fn prepare_image_inputs(input_type: &InputType) -> anyhow::Result<Vec<PathBuf>> {
    info!("Preparing the image inputs.");
    let mut image_paths: Vec<PathBuf> = Vec::new();

    if let Some(instances_json_path) = &input_type.instances_json {
        debug!(
            "Loading the instances file from {}",
            instances_json_path.display()
        );

        let instances_file = BufReader::new(File::open(instances_json_path)?);
        let instance_json_value: Instances = serde_json::from_reader(instances_file)?;

        for v in instance_json_value.instances {
            let instances_file_folder = instances_json_path
                .parent()
                .expect("Instances file's parent path is None.");

            let jointed_image_path = instances_file_folder.join(v.filepath);
            image_paths.push(jointed_image_path);
        }
    }

    if !input_type.filepaths.is_empty() {
        debug!("Loading the filepaths from filepaths option in the CLI.");

        for f in &input_type.filepaths {
            image_paths.push(f.to_path_buf());
        }
    }

    if let Some(filepaths_txt_path) = &input_type.filepaths_txt {
        debug!("Loading the filepaths from given filepaths.txt.");

        let filepaths_txt_file = BufReader::new(File::open(filepaths_txt_path)?);
        let lines = filepaths_txt_file.lines();

        for line in lines {
            let line = line?;
            let path_buf = PathBuf::from(&line);
            image_paths.push(path_buf);
        }
    }

    if !input_type.folders.is_empty() {
        debug!("Loading the files from given folders.");

        for folder in &input_type.folders {
            // Only walk on ok path, skipping any errors.
            for entry in WalkDir::new(folder).into_iter().filter_map(|e| e.ok()) {
                image_paths.push(entry.into_path());
            }
        }
    }

    if let Some(folders_txt_path) = &input_type.folders_txt {
        debug!("Loading the folders path from given folders.txt.");

        let folders_txt_file = BufReader::new(File::open(folders_txt_path)?);
        let lines = folders_txt_file.lines();

        for line in lines {
            let line = line?;
            image_paths.push(PathBuf::from(&line));
        }
    }

    info!(
        "Found {} files from given instances files, folders and directories.",
        image_paths.len()
    );
    info!("Filtering non image paths out from gathered files.");

    let filtered_paths: Vec<_> = image_paths
        .into_iter()
        .filter(|p| match p.extension() {
            Some(file_extension_osstr) => match file_extension_osstr.to_str() {
                Some(file_extension) => {
                    SUPPORTED_IMAGE_EXTENSIONS.contains(&file_extension.to_lowercase().as_str())
                }
                None => false,
            },
            None => false,
        })
        .collect();

    info!(
        "{} images left after filtering finished.",
        filtered_paths.len()
    );

    Ok(filtered_paths)
}
