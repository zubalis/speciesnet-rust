#[derive(Clone, Debug, PartialEq)]
pub struct GeofenceResult {
    label: String,
    score: f64,
    source: String,
}

impl GeofenceResult {
    pub fn new(label: String, score: f64, source: String) -> Self {
        Self { label, score, source }
    }
    pub fn label(&self) -> &str {
        &self.label
    }
    pub fn score(&self) -> f64 {
        self.score
    }
    pub fn source(&self) -> &str {
        &self.source
    }
}