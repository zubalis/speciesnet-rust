pub mod bounding_box;
pub mod category;
pub mod classification;
pub mod constants;
pub mod detection;
pub mod error;
pub mod geofence;
pub mod instance;
mod macros;
pub mod prediction;
pub mod shape;

pub use crate::bounding_box::BoundingBox;
pub use crate::category::Category;
pub use crate::detection::Detection;
pub use crate::instance::{Instance, Instances};
