use std::env::current_dir;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

use speciesnet_classifier::SpeciesNetClassifier;
use speciesnet_classifier::classifier::transform;
use speciesnet_classifier::image::{preprocess};
use speciesnet_classifier::input::ClassifierInput;

#[test]
fn test_entire_process() -> Result<(), Box<dyn Error>> {
    // Load model
    let model_dir_path = current_dir()?.join("..").join("assets").join("model");
    let classifier = SpeciesNetClassifier::new(model_dir_path)?;

    // Load inputs and preprocess them
    let img_path = current_dir()?
        .join("..")
        .join("assets")
        .join("images")
        .join("african_elephants.jpg");
    let classifier_input = ClassifierInput {
        file_path: img_path,
        bbox: None,
    };
    let inputs = preprocess(&classifier_input)?;

    // Run classify inputs
    let outputs = classifier.classify(&inputs.image_tensor);
    assert!(outputs.is_ok());
    let outputs = outputs?;

    // Load labels
    let label_path = current_dir()?.join("..").join("assets").join("model").join("labels.txt");
    let label_file = File::open(label_path)?;
    let label_reader = BufReader::new(label_file);
    let labels: Vec<String> = label_reader.lines().map_while(Result::ok).collect();

    // Transform outputs into usable format (softmax, mapping labels, pick top 5)
    let image_path = inputs.path;
    let output = transform(image_path, &outputs, &labels);
    
    assert!(output.classifications().is_some());

    Ok(())
}
