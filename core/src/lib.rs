pub mod bounding_box;
pub mod category;
pub mod detection;
pub mod error;
pub mod instance;
pub mod prediction;
pub mod shape;
pub mod classification;

pub use crate::bounding_box::BoundingBox;
pub use crate::category::Category;
pub use crate::detection::Detection;
pub use crate::instance::{Instance, Instances};
