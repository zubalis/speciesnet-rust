use std::collections::HashMap;
use tensorflow::{Graph, Session, SessionOptions, SessionRunArgs, SavedModelBundle, Tensor, Scope};
use image::{io::Reader as ImageReader, DynamicImage, GenericImageView};
use std::fs::File;
use std::io::{self, BufRead, Error};
use std::path::Path;

struct Label {
    name: String
}

fn main() -> tensorflow::Result<()> {
    // Load the TensorFlow model
    let export_dir = "/Users/tooseriuz/RustroverProjects/RustTensorflow/model"; // Replace with actual path
    let mut graph = Graph::new();
    let bundle = SavedModelBundle::load(&SessionOptions::new(), &["serve"], &mut graph, export_dir)?;

    let session = &bundle.session;

    // Prepare input tensor
    // input must be 4-dimensional
    // [2, 480, 480, 3] as model expects image 480x480
    let input_tensor: Tensor<f32> = match load_and_preprocess_image("/Users/tooseriuz/RustroverProjects/RustTensorflow/african_elephants.jpg") {
        Ok(tensor) => tensor,
        Err(err) => panic!("{}",err)
    };

    // Get input/output operation names from the model
    let input_op = graph.operation_by_name_required("serving_default_rescaling_input")?;
    let output_op = graph.operation_by_name_required("StatefulPartitionedCall")?;

    // Run the model
    let mut args = SessionRunArgs::new();
    args.add_feed(&input_op, 0, &input_tensor);
    let output_token = args.request_fetch(&output_op, 0);

    session.run(&mut args)?;

    // Get output
    let labels = match read_label_and_load_to_Vec("/Users/tooseriuz/RustroverProjects/RustTensorflow/model/labels.txt") {
        Ok(labels) => labels,
        Err(err) => panic!("{}",err)
    };
    let output_tensor: Tensor<f32> = args.fetch(output_token)?;
    let o_vec = output_tensor.to_vec();
    let output_vec = softmax(o_vec[0..2998].to_vec());
    println!("{}", output_vec.len());

    let mut output_map: Vec<_> = labels.into_iter().zip(output_vec).collect();
    output_map.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    println!("Model Output: {:#?}", output_map.iter().take(10).collect::<Vec<_>>());

    Ok(())
}

fn load_and_preprocess_image(image_path: &str) -> tensorflow::Result<Tensor<f32>> {
    // Load image from file
    let img = match ImageReader::open(image_path) {
        Ok(img) => img.decode().unwrap(),
        Err(err) => panic!("failed to decode image: {}", err)
    };

    // Resize to 480x480 (modify if your model expects a different size)
    let img = img.resize_exact(480, 480, image::imageops::FilterType::Triangle);

    // Convert to RGB and flatten into [480 * 480 * 3] vector
    let img_rgb = img.to_rgb8();
    let pixels: Vec<f32> = img_rgb.pixels()
        .flat_map(|p| p.0) // Extract RGB channels
        .map(|v| v as f32 / 255.0) // Normalize to [0.0, 1.0]
        .collect();

    // Batch data
    let mut batch_data = Vec::with_capacity(2 * 480 * 480 * 3);
    batch_data.extend_from_slice(&pixels);
    batch_data.extend_from_slice(&pixels);

    // Create a Tensor of shape [2, 480, 480, 3]
    let tensor = Tensor::new(&[2, 480, 480, 3]).with_values(&batch_data)?;

    Ok(tensor)
}

fn read_label_and_load_to_Vec(label_path: &str) -> Result<Vec<String>, Error> {
    // Open the file
    let file = File::open(&label_path)?;
    let reader = io::BufReader::new(file);

    // Read the file line by line
    let mut labels: Vec<String> = vec!();
    for line in reader.lines() {
        let line = line?; // Handle errors per line
        let name = line.split(';').collect::<Vec<_>>()[6];
        labels.push(name.to_string());
    }

    Ok(labels)
}

fn softmax(scores: Vec<f32>) -> Vec<f32> {
    let exp_values: Vec<f32> = scores.iter().map(|&score| score.exp()).collect();
    let sum_exp_values: f32 = exp_values.iter().sum();
    exp_values.iter().map(|&exp_value| exp_value / sum_exp_values).collect()
}
