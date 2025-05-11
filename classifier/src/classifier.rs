use std::{
    cmp::Ordering::Equal,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use ndarray::ArrayView1;
use speciesnet_core::{
    classifier::{Classification, ClassificationBundle},
    io::Prediction,
};

use crate::error::Error;

#[cfg(test)]
mod tests;

pub fn softmax(scores: ArrayView1<f32>) -> Vec<f64> {
    let exp_values: Vec<f64> = scores.iter().map(|&score| score.exp() as f64).collect();
    let sum_exp_values: f64 = exp_values.iter().sum();
    exp_values
        .iter()
        .map(|&exp_value| exp_value / sum_exp_values)
        .collect()
}

pub fn map_labels_to_classifications(
    labels: &[String],
    classifications: &[f64],
) -> Vec<Classification> {
    labels
        .iter()
        .zip(classifications.iter())
        .map(|(label, &score)| Classification::new(label.clone(), score))
        .collect()
}

pub fn pick_top_n_from(mut classifications: Vec<Classification>, n: usize) -> Vec<Classification> {
    classifications.sort_by(|a, b| b.score().partial_cmp(a.score()).unwrap_or(Equal));
    let sorted_classifications: Vec<Classification> = classifications.into_iter().take(n).collect();
    sorted_classifications
}

pub fn to_chunks(outputs: &[f32], chunk_size: usize) -> Vec<Vec<f32>> {
    outputs
        .chunks(chunk_size)
        .map(|chunk| chunk.to_vec())
        .collect()
}

pub fn transform<P: AsRef<Path>>(
    file_path: P,
    outputs: ArrayView1<f32>,
    labels: &[String],
) -> Prediction {
    let softmax_result = softmax(outputs);
    let mapped_result = map_labels_to_classifications(labels, &softmax_result);
    let top5_result = pick_top_n_from(mapped_result, 5);
    let labels = top5_result.iter().map(|c| c.label().clone()).collect();
    let scores = top5_result.iter().map(|c| *c.score()).collect();
    Prediction::from_classifications(
        file_path.as_ref().to_path_buf(),
        ClassificationBundle::new(labels, scores),
    )
}

pub fn read_labels_from_file<P: AsRef<Path>>(file_path: P) -> Result<Vec<String>, Error> {
    let label_file = File::open(file_path)?;
    let label_reader = BufReader::new(label_file);
    let labels: Vec<String> = label_reader.lines().map_while(Result::ok).collect();
    Ok(labels)
}
