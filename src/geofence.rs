pub mod taxonomy;
#[cfg(test)]
mod tests;

use crate::geofence::taxonomy::{TaxonomyError, get_ancestor_at_level};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::iter::zip;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum GeofenceError {
    #[error("{0}")]
    InvalidValue(String),
}

///
/// Check if label is allowed withing a given country
///
/// # Parameters:
///   - label:
///       Animal label from classification result
///   - country:
///       Country code (in ISO 3166-1 alpha-3 format)
///   - admin1_region:
///       First-level administrative division (in ISO 3166-2 format)
///   - geofence_map:
///       Map that has full class species string as keys, array of `allow` as values
///   - enable_geofence:
///       Whether geofencing is enabled
///
fn should_geofence(
    label: &str,
    country: Option<&str>,
    admin1_region: Option<&str>,
    geofence_map: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>,
    enable_geofence: bool,
) -> Result<bool, TaxonomyError> {
    // Do not geofence if not enabled
    if !enable_geofence {
        return Ok(false);
    }

    // Do not geofence if country not given
    let _country = match country {
        Some(c) => c,
        None => return Ok(false),
    };

    // Get full string, exclude uuid and scientific name
    let full_string_class = taxonomy::get_full_class_string(label)?;
    let geofence_from_full_string = match geofence_map.get(&full_string_class) {
        Some(g) => g,
        None => return Ok(false),
    };

    // Get `allow` countries from given geofence_map
    match geofence_from_full_string.get("allow") {
        Some(allowed_countries) => {
            // Do geofence if given country not in allowed country
            if !allowed_countries.is_empty() {
                if !allowed_countries.contains_key(_country) {
                    return Ok(true);
                } else {
                    // Get states from given country
                    match allowed_countries.get(_country) {
                        Some(allowed_admin1_region) => {
                            match admin1_region {
                                Some(ar) => {
                                    // Do geofence if admin1_region not in allowed admin1_region.
                                    if !allowed_admin1_region.is_empty()
                                        && !allowed_admin1_region.contains(&ar.to_string())
                                    {
                                        return Ok(true);
                                    }
                                }
                                None => (),
                            };
                        }
                        None => (),
                    }
                }
            }
        }
        None => (),
    }

    // Get `block` countries from given geofence_map
    match geofence_from_full_string.get("block") {
        Some(blocked_countries) => {
            // Do geofence if given country in blocked country
            if !blocked_countries.is_empty() {
                if blocked_countries.contains_key(_country) {
                    match blocked_countries.get(_country) {
                        Some(blocked_admin1_regions) => {
                            if blocked_admin1_regions.is_empty() {
                                return Ok(true);
                            }
                            match admin1_region {
                                Some(ar) => {
                                    // Do geofence if given admin1_region in blocked admin1_region
                                    if blocked_admin1_regions.contains(&ar.to_string()) {
                                        return Ok(true);
                                    };
                                    ()
                                }
                                None => (),
                            };
                        }
                        None => (),
                    }
                }
            }
        }
        None => (),
    }
    Ok(false)
}

///
/// Rolls up prediction labels to the first taxonomy level above given threshold
///
/// # Parameters:
///   - labels:
///       List of classification labels
///   - scores:
///       List of classification scores
///   - country:
///       Country code (in ISO 3166-1 alpha-3 format)
///       Optional
///   - admin1_region:
///       First-level administrative division (in ISO 3166-2 format)
///       Optional
///   - target_taxonomy_levels:
///       Ordered list of taxonomy levels at which to roll up classification
///       labels and check if the cumulative score passes the given threshold.
///       Levels must be a subset of: `species`, `genus`, `family`, `order`,
///       `class`, `kingdom`.
///   - non_blank_threshold:
///       Min threshold at which the cumulative score is good enough to consider
///       the rollup successful.
///   - taxonomy_map:
///       Map that map taxa to labels.
///   - geofence_map:
///       Map that has full class species string as keys, array of `allow` as values
///   - enable_geofence:
///       Whether geofencing is enabled
///
fn roll_up_labels_to_first_matching_level(
    labels: &Vec<&str>,
    scores: &Vec<f32>,
    country: Option<&str>,
    admin1_region: Option<&str>,
    target_taxonomy_levels: &Vec<&str>,
    non_blank_threshold: &f32,
    taxonomy_map: &HashMap<String, String>,
    geofence_map: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>,
    enable_geofence: bool,
) -> Result<Option<(String, f32, String)>, Box<dyn Error>> {
    // Find if there is invalid taxonomy level
    let expected_target_taxonomy_levels =
        vec!["species", "genus", "family", "order", "class", "kingdom"];
    let set_expected_target_taxonomy_levels: HashSet<_> =
        expected_target_taxonomy_levels.iter().collect();
    let set_target_taxonomy_levels: HashSet<_> = target_taxonomy_levels.iter().collect();
    let has_unknown_taxonomy: HashSet<_> = set_target_taxonomy_levels
        .difference(&set_expected_target_taxonomy_levels)
        .into_iter()
        .collect();
    if !has_unknown_taxonomy.is_empty() {
        return Err(GeofenceError::InvalidValue(format!(
            "Unexpected target taxonomy level(s): {:?}. Expected only from the set: {:?}",
            has_unknown_taxonomy, expected_target_taxonomy_levels
        ))
        .into());
    };

    for taxonomy_level in target_taxonomy_levels {
        let mut accumulated_scores = HashMap::new();
        for (label, score) in labels.into_iter().zip(scores.into_iter()) {
            let roll_up_label = get_ancestor_at_level(label, taxonomy_level, taxonomy_map)?;
            if let Some(r_label) = roll_up_label {
                let new_score = accumulated_scores.get(&r_label).unwrap_or(&0.0f32) + score;
                accumulated_scores.insert(r_label, new_score);
            }
        }

        let mut max_rollup_label = "";
        let mut max_rollup_score = 0.0f32;
        for (r_label, r_score) in &accumulated_scores {
            if r_score > &max_rollup_score
                && !should_geofence(
                    r_label,
                    country,
                    admin1_region,
                    geofence_map,
                    enable_geofence,
                )?
            {
                max_rollup_label = r_label;
                max_rollup_score = *r_score;
            }
        }

        if max_rollup_score > *non_blank_threshold && !max_rollup_label.is_empty() {
            return Ok(Some((
                max_rollup_label.to_string(),
                max_rollup_score,
                format!("classifier+rollup_to_{}", taxonomy_level),
            )));
        }
    }

    Ok(None)
}

fn geofence_classification() {}
