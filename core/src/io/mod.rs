//! Module for storing types related to the input and output required for running the model.

pub mod instance;
pub mod prediction;

pub use instance::{Instance, Instances};
pub use prediction::{Prediction, Predictions};
