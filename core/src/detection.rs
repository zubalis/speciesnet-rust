use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use crate::{bounding_box::BoundingBox, category::Category};

/// The detection produced from running the model.
#[derive(Debug, Clone)]
pub struct Detection {
    file_path: PathBuf,
    category: Category,
    confidence: f64,
    bounding_box: BoundingBox,
}

impl Display for Detection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "File: {}, Category: {}, Confidence: {}, Bounding box: {}",
            self.file_path.display(),
            self.category,
            self.confidence,
            self.bounding_box
        )
    }
}

impl Detection {
    pub fn new(
        file_path: PathBuf,
        category: Category,
        confidence: f64,
        bounding_box: BoundingBox,
    ) -> Self {
        assert!((0.0f64..1.0f64).contains(&confidence));

        Self {
            file_path,
            category,
            confidence,
            bounding_box,
        }
    }

    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    /// Returns the label of the category.
    pub fn label(&self) -> String {
        self.category.to_string()
    }

    pub fn bounding_box(&self) -> &BoundingBox {
        &self.bounding_box
    }

    pub fn category(&self) -> &Category {
        &self.category
    }

    pub fn confidence(&self) -> f64 {
        self.confidence
    }
}
