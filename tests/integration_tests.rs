use std::collections::HashMap;
use std::env::current_dir;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use speciesnet_rust::classifier::{transform, Classifier, ClassifierConfig};
use speciesnet_rust::geofence::geofence_animal_classification;
use speciesnet_rust::geofence::taxonomy::get_full_class_string;
use speciesnet_rust::image::load_and_preprocess_images;

#[test]
fn test_entire_process() -> Result<(), Box<dyn Error>> {
    /// Load model
    let model_path = current_dir()?.join("assets").join("model").canonicalize()?.to_str().unwrap().to_string();
    let config = ClassifierConfig {
        model_path,
        input_layer: "serving_default_rescaling_input".to_string(),
        output_layer: "StatefulPartitionedCall".to_string(),
    };
    let classifier = Classifier::new(config)?;

    /// Load inputs and preprocess them
    let img_path = current_dir()?.join("assets").join("images").join("african_elephants.jpg").canonicalize()?.to_str().unwrap().to_string();
    let inputs = load_and_preprocess_images(vec![img_path])?;

    /// Run classify inputs
    let outputs = classifier.classify(&inputs.success_images.image_tensor);
    assert!(outputs.is_ok());
    let outputs = outputs?;

    /// Load labels
    let label_path = current_dir()?.join("assets").join("labels.txt");
    let label_file = File::open(label_path)?;
    let label_reader = BufReader::new(label_file);
    let labels: Vec<String> = label_reader.lines().filter_map(Result::ok).collect();

    /// Transform outputs into usable format (softmax, mapping labels, pick top 5)
    let image_paths = inputs.success_images.paths;
    let outputs_map = transform(&image_paths, &outputs, &labels);

    /// Load geofence map from file
    let geofence_path = current_dir()?.join("assets").join("geofence_base.json").canonicalize()?.to_str().unwrap().to_string();
    let geofence_file = File::open(geofence_path)?;
    let geofence_reader = BufReader::new(geofence_file);
    let geofence_map: HashMap<String, HashMap<String, HashMap<String, Vec<String>>>> = serde_json::from_reader(geofence_reader)?;

    /// Load taxonomy map from file
    let taxonomy_path = current_dir()?.join("assets").join("taxonomy_release.txt");
    let taxonomy_file = File::open(taxonomy_path)?;
    let taxonomy_reader = BufReader::new(taxonomy_file);
    let taxonomies: Vec<String> = taxonomy_reader.lines().filter_map(Result::ok).collect();
    let mut taxonomy_map: HashMap<String, String> = HashMap::new();
    for t in taxonomies {
        taxonomy_map.insert(get_full_class_string(&t)?, t.clone());
    }

    /// Geofencing each files and results from classification
    for path in image_paths {
        let bundle = outputs_map.get(&path).unwrap();
        let result = geofence_animal_classification(
            &bundle.labels,
            &bundle.scores,
            Some("THA"),
            None,
            &taxonomy_map,
            &geofence_map,
            true
        );
        assert!(result.is_ok());
    }
    Ok(())
}