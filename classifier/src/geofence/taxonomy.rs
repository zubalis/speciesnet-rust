use std::collections::HashMap;

use crate::{constants::classification, error::Error};

#[cfg(test)]
mod tests;

///
/// Finds the taxonomy item corresponding to a label's ancestor at a given level
///
/// e.g. The ancestor at family level for
///
///   `uuid;class;order;family;genus;species;common_name` is
///   `another_uuid;class;order;family;;;another_common_name`
///
/// # Parameters:
///   - label:
///       String slice label for to extract the full class string.
///   - taxonomy_level:
///       One of `species`, `genus`, `family`, `order`, `class` or `kingdom`,
///       indicating the taxonomy level at which to find a label's ancestor
///   - taxonomy_map:
///       Map that map taxa to labels
///
pub fn get_ancestor_at_level(
    label: &str,
    taxonomy_level: &str,
    taxonomy_map: &HashMap<String, String>,
) -> Result<Option<String>, Error> {
    let label_parts = label.split(';').collect::<Vec<_>>();
    if label_parts.len() != 7 {
        return Err(Error::InvalidLabel(
            label_parts.len().to_string(),
            label.to_string(),
        ));
    }

    let mut ancestor_parts: Vec<&str>;
    match taxonomy_level {
        "species" => {
            ancestor_parts = label_parts[1..6].to_vec();
            match ancestor_parts.get(4) {
                Some(ancestor) => {
                    if ancestor.is_empty() {
                        return Ok(None);
                    }
                }
                None => return Ok(None),
            }
        }
        "genus" => {
            ancestor_parts = label_parts[1..5].to_vec();
            let mut blank_vec = vec![""];
            ancestor_parts.append(&mut blank_vec);
            match ancestor_parts.get(3) {
                Some(ancestor) => {
                    if ancestor.is_empty() {
                        return Ok(None);
                    }
                }
                None => return Ok(None),
            }
        }
        "family" => {
            ancestor_parts = label_parts[1..4].to_vec();
            let mut blank_vec = vec!["", ""];
            ancestor_parts.append(&mut blank_vec);
            match ancestor_parts.get(2) {
                Some(ancestor) => {
                    if ancestor.is_empty() {
                        return Ok(None);
                    }
                }
                None => return Ok(None),
            }
        }
        "order" => {
            ancestor_parts = label_parts[1..3].to_vec();
            let mut blank_vec = vec!["", "", ""];
            ancestor_parts.append(&mut blank_vec);
            match ancestor_parts.get(1) {
                Some(ancestor) => {
                    if ancestor.is_empty() {
                        return Ok(None);
                    }
                }
                None => return Ok(None),
            }
        }
        "class" => {
            ancestor_parts = label_parts[1..2].to_vec();
            let mut blank_vec = vec!["", "", "", ""];
            ancestor_parts.append(&mut blank_vec);
            match ancestor_parts.first() {
                Some(ancestor) => {
                    if ancestor.is_empty() {
                        return Ok(None);
                    }
                }
                None => return Ok(None),
            }
        }
        "kingdom" => {
            ancestor_parts = vec!["", "", "", "", ""];
            match label_parts.get(1) {
                Some(p) => {
                    if p.is_empty() && label != classification::ANIMAL {
                        return Ok(None);
                    }
                }
                None => return Ok(None),
            }
        }
        _ => {
            return Err(Error::InvalidTaxonomyLevel(taxonomy_level.to_string()));
        }
    }

    let ancestor = ancestor_parts.join(";");
    match taxonomy_map.get(&ancestor) {
        Some(taxonomy) => Ok(Some(taxonomy.to_string())),
        None => Ok(None),
    }
}

///
/// Extract full class string from a given label
///
/// e.g. The full class string for the label
///
///   `uuid;class;order;family;genus;species;common_name` is
///   `class;order;family;genus;species`
///
/// # Parameters:
///   - label:
///       String slice label for to extract the full class string.
///
pub fn get_full_class_string(label: &str) -> Result<String, Error> {
    let label_parts = label.split(';').collect::<Vec<_>>();
    if label_parts.len() != 7 {
        return Err(Error::InvalidLabel(
            label_parts.len().to_string(),
            label.to_string(),
        ));
    }
    Ok(label_parts[1..6].join(";").to_string())
}
