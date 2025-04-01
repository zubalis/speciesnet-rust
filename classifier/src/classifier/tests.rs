use std::cell::LazyCell;
use std::path::PathBuf;

use crate::classifier::{
    Classification, map_labels_to_classifications, pick_top_n_from, softmax, to_chunks, transform,
};
use speciesnet_core::prediction::Prediction;

const LABELS: LazyCell<Vec<String>> = LazyCell::new(|| {
    vec![
        "lion".to_string(),
        "elephant".to_string(),
        "cat".to_string(),
        "dog".to_string(),
        "bird".to_string(),
        "bat".to_string(),
    ]
});

const SCORES: LazyCell<Vec<f32>> = LazyCell::new(|| {
    vec![
        0.81172436,
        0.10985495,
        0.040413376,
        0.024511952,
        0.009017443,
        0.0044779293,
    ]
});

const CLASSIFICATIONS: LazyCell<Vec<Classification>> = LazyCell::new(|| {
    vec![
        Classification {
            label: LABELS[0].clone(),
            score: SCORES[0],
        },
        Classification {
            label: LABELS[1].clone(),
            score: SCORES[1],
        },
        Classification {
            label: LABELS[2].clone(),
            score: SCORES[2],
        },
        Classification {
            label: LABELS[3].clone(),
            score: SCORES[3],
        },
        Classification {
            label: LABELS[4].clone(),
            score: SCORES[4],
        },
        Classification {
            label: LABELS[5].clone(),
            score: SCORES[5],
        },
    ]
});

#[test]
fn test_softmax_fn() {
    let scores: Vec<f32> = vec![4.0, 2.0, 1.0, 0.5, -0.5, -1.2];
    let expected_scores = SCORES.clone();

    let result = softmax(&scores);

    assert_eq!(result, expected_scores)
}

#[test]
fn test_map_labels_to_classifications_fn() {
    let expected = CLASSIFICATIONS.clone();

    let result = map_labels_to_classifications(&LABELS, &SCORES);

    assert_eq!(result, expected)
}

#[test]
fn test_pick_top_n_from_fn() {
    let classifications = CLASSIFICATIONS.clone();
    let expected: Vec<Classification> = vec![
        Classification {
            label: LABELS[0].to_string(),
            score: SCORES[0],
        },
        Classification {
            label: LABELS[1].to_string(),
            score: SCORES[1],
        },
        Classification {
            label: LABELS[2].to_string(),
            score: SCORES[2],
        },
    ];

    let result = pick_top_n_from(classifications, 3);
    assert_eq!(result, expected);
}

#[test]
fn test_to_chunk_fn() {
    let scores: Vec<f32> = vec![4.0, 2.0, 1.0, 0.5, -0.5, -1.2];

    let chunks = to_chunks(&scores, 3);

    assert_eq!(chunks.len(), 2);
    for chunk in chunks {
        assert_eq!(chunk.len(), 3);
    }
}

#[test]
fn test_transform_fn() {
    let file_paths = vec![
        PathBuf::from("path/to/file/1.png"),
        PathBuf::from("path/to/file/2.png"),
        PathBuf::from("path/to/file/3.png"),
    ];
    let scores: Vec<Vec<f32>> = vec![
        vec![4.0, 2.0, 1.0, 0.5, -0.5, -1.2, 3.0, -1.1],
        vec![1.0, 2.0, 5.0, 0.5, -0.5, -1.2, 1.2, -2.1],
        vec![3.0, 2.0, 1.0, 9.5, -0.5, -1.2, 5.0, -1.2],
    ];
    let labels = vec![
        "lion".to_string(),
        "elephant".to_string(),
        "cat".to_string(),
        "dog".to_string(),
        "bird".to_string(),
        "tiger".to_string(),
        "ant".to_string(),
        "fish".to_string(),
    ];

    let results: Vec<Prediction> = file_paths
        .iter()
        .zip(scores.iter())
        .map(|(p, s)| transform(p, s, &labels))
        .collect();

    let first = &results.get(0).unwrap();
    let first_classifications = first.classifications().as_ref().unwrap();
    assert_eq!(
        first_classifications.scores,
        vec![0.62269545, 0.22907685, 0.08427267, 0.031002179, 0.018803772]
    );
    assert_eq!(
        first_classifications.labels,
        vec![
            "lion".to_string(),
            "ant".to_string(),
            "elephant".to_string(),
            "cat".to_string(),
            "dog".to_string()
        ]
    );
    let second = &results.get(1).unwrap();
    let second_classifications = second.classifications().as_ref().unwrap();
    assert_eq!(
        second_classifications.scores,
        vec![
            0.90210056,
            0.04491294,
            0.020180685,
            0.016522547,
            0.010021431
        ]
    );
    assert_eq!(
        second_classifications.labels,
        vec![
            "cat".to_string(),
            "elephant".to_string(),
            "ant".to_string(),
            "lion".to_string(),
            "dog".to_string()
        ]
    );
    let third = &results.get(2).unwrap();
    let third_classifications = third.classifications().as_ref().unwrap();
    assert_eq!(
        third_classifications.scores,
        vec![
            0.9867193,
            0.010961462,
            0.0014834725,
            0.000545739,
            0.00020076617
        ]
    );
    assert_eq!(
        third_classifications.labels,
        vec![
            "dog".to_string(),
            "ant".to_string(),
            "lion".to_string(),
            "elephant".to_string(),
            "cat".to_string()
        ]
    );
}
