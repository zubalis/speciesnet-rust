use thiserror::Error;

#[derive(Debug, Error)]
pub enum TaxonomyError {
    #[error("Expected label made of 7 parts, but found only {0}: {1}")]
    InvalidLabel(String, String)
}

pub fn get_full_class_string(label: &str) -> Result<String, TaxonomyError> {
    let label_parts = label.split(';').collect::<Vec<_>>();
    if label_parts.len() != 7 {
        return Err(TaxonomyError::InvalidLabel(label_parts.len().to_string(), label.to_string()));
    }
    Ok(label_parts[1..6].join(";").to_string())
}