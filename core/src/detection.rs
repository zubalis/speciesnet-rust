use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::{bounding_box::BoundingBox, category::Category};

/// The detection produced from running the model.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Detection {
    category: Category,
    confidence: f64,
    bounding_box: BoundingBox,
}

impl Display for Detection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Category: {}, Confidence: {}, Bounding box: {}",
            self.category, self.confidence, self.bounding_box
        )
    }
}

impl Detection {
    /// Initialize the [`Detection`] struct.
    ///
    /// # Panics
    ///
    /// The initialization could panic if the confidence is not in between `0` and `1`.
    pub fn new(category: Category, confidence: f64, bounding_box: BoundingBox) -> Self {
        assert!((0.0f64..1.0f64).contains(&confidence));

        Self {
            category,
            confidence,
            bounding_box,
        }
    }

    /// Returns the label of the category.
    pub fn label(&self) -> String {
        self.category.to_string()
    }

    /// Returns the [`BoundingBox`] of the detection.
    pub fn bounding_box(&self) -> &BoundingBox {
        &self.bounding_box
    }

    /// Returns the [`Category`] of the detection.
    pub fn category(&self) -> &Category {
        &self.category
    }

    /// Returns the confidence value of the detection.
    pub fn confidence(&self) -> f64 {
        self.confidence
    }
}
