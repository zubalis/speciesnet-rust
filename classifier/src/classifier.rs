#[cfg(test)]
mod tests;

use std::cmp::Ordering::Equal;
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use tensorflow::{Graph, SavedModelBundle, SessionOptions, SessionRunArgs, Tensor};
use crate::image::ProceededImages;

pub struct ClassifierConfig {
    pub model_path: String,
    pub input_layer: String,
    pub output_layer: String,
}
pub struct Classifier {
    bundle: SavedModelBundle,
    graph: Graph,
    input_layer: String,
    output_layer: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ClassificationBundle {
    pub file_path: String,
    pub labels: Vec<String>,
    pub scores: Vec<f32>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Classification {
    label: String,
    score: f32,
}

impl Classifier {
    /// Create classifier from given config
    pub fn new(config: ClassifierConfig) -> Result<Self, Box<dyn Error>> {
        let model_path = Path::new(&config.model_path);
        let mut graph = Graph::new();
        let bundle = SavedModelBundle::load(
            &SessionOptions::new(),
            &["serve"],
            &mut graph,
            model_path,
        )?;
        Ok(Classifier {
            bundle,
            graph,
            input_layer: config.input_layer,
            output_layer: config.output_layer,
        })
    }

    /// run a classification from given input
    pub fn classify(&self, input_tensor: &Tensor<f32>) -> Result<Vec<f32>, Box<dyn Error>> {
        let session = &self.bundle.session;
        let mut args = SessionRunArgs::new();

        let input_op = self.graph.operation_by_name_required(&self.input_layer)?;
        let output_op = self.graph.operation_by_name_required(&self.output_layer)?;

        args.add_feed(&input_op, 0, &input_tensor);

        let output_token = args.request_fetch(&output_op, 0);
        session.run(&mut args)?;

        let output_tensor: Tensor<f32> = args.fetch(output_token)?;
        let o_vec = output_tensor.to_vec();

        Ok(o_vec)
    }
}

pub fn softmax(scores: &Vec<f32>) -> Vec<f32> {
    let exp_values: Vec<f32> = scores.iter().map(|&score| score.exp()).collect();
    let sum_exp_values: f32 = exp_values.iter().sum();
    exp_values
        .iter()
        .map(|&exp_value| exp_value / sum_exp_values)
        .collect()
}

pub fn map_labels_to_classifications(
    labels: &Vec<String>,
    classifications: &Vec<f32>,
) -> Vec<Classification> {
    labels
        .iter()
        .zip(classifications.iter())
        .map(|(label, &score)| Classification {
            label: label.clone(),
            score,
        })
        .collect()
}

pub fn pick_top_n_from(
    mut classifications: Vec<Classification>,
    n: usize,
) -> Vec<Classification> {
    classifications.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(Equal)
    });
    let sorted_classifications: Vec<Classification> = classifications.into_iter().take(n).collect();
    sorted_classifications
}

pub fn to_chunks(outputs: &Vec<f32>, chunk_size: usize) -> Vec<Vec<f32>> {
    outputs.chunks(chunk_size)
        .map(|chunk| chunk.to_vec())  // Convert slice to Vec<T>
        .collect()
}

pub fn transform(file_paths: &Vec<String>, outputs: &Vec<f32>, labels: &Vec<String>) -> HashMap<String, ClassificationBundle> {
    let chunks = to_chunks(outputs, labels.len());
    let mut bundle = HashMap::new();
    for (chunk, path) in chunks.iter().zip(file_paths.iter()) {
        let softmax_chunk = softmax(chunk);
        let mapped_chunks = map_labels_to_classifications(&labels, &softmax_chunk);
        let top5_chunks = pick_top_n_from(mapped_chunks, 5);
        let labels = top5_chunks.iter().map(|c| c.label.clone()).collect();
        let scores = top5_chunks.iter().map(|c| c.score.clone()).collect();
        bundle.insert(path.clone(), ClassificationBundle {
            file_path: path.clone(),
            labels,
            scores
        });
    }
    bundle
}
