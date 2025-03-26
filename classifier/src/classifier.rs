use std::cmp::Ordering::Equal;
use std::collections::HashMap;
use std::path::PathBuf;

#[cfg(test)]
mod tests;

#[derive(Debug, PartialEq, Clone)]
pub struct ClassificationBundle {
    pub file_path: PathBuf,
    pub labels: Vec<String>,
    pub scores: Vec<f32>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Classification {
    label: String,
    score: f32,
}

pub fn softmax(scores: &[f32]) -> Vec<f32> {
    let exp_values: Vec<f32> = scores.iter().map(|&score| score.exp()).collect();
    let sum_exp_values: f32 = exp_values.iter().sum();
    exp_values
        .iter()
        .map(|&exp_value| exp_value / sum_exp_values)
        .collect()
}

pub fn map_labels_to_classifications(
    labels: &[String],
    classifications: &[f32],
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

pub fn pick_top_n_from(mut classifications: Vec<Classification>, n: usize) -> Vec<Classification> {
    classifications.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Equal));
    let sorted_classifications: Vec<Classification> = classifications.into_iter().take(n).collect();
    sorted_classifications
}

pub fn to_chunks(outputs: &[f32], chunk_size: usize) -> Vec<Vec<f32>> {
    outputs
        .chunks(chunk_size)
        .map(|chunk| chunk.to_vec()) // Convert slice to Vec<T>
        .collect()
}

pub fn transform(
    file_paths: &[PathBuf],
    outputs: &[f32],
    labels: &[String],
) -> HashMap<PathBuf, ClassificationBundle> {
    let chunks = to_chunks(outputs, labels.len());
    let mut bundle = HashMap::new();
    for (chunk, path) in chunks.iter().zip(file_paths.iter()) {
        let softmax_chunk = softmax(chunk);
        let mapped_chunks = map_labels_to_classifications(labels, &softmax_chunk);
        let top5_chunks = pick_top_n_from(mapped_chunks, 5);
        let labels = top5_chunks.iter().map(|c| c.label.clone()).collect();
        let scores = top5_chunks.iter().map(|c| c.score).collect();
        bundle.insert(
            path.clone(),
            ClassificationBundle {
                file_path: path.clone(),
                labels,
                scores,
            },
        );
    }
    bundle
}
