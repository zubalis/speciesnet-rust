#![allow(clippy::redundant_closure)]

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::Path;

use csv::Reader;
use serde::Deserialize;
use speciesnet_core::constants;
use speciesnet_core::ensemble::GeofenceResult;

use crate::error::Error;
use crate::geofence::taxonomy::get_ancestor_at_level;

pub mod taxonomy;
#[cfg(test)]
mod tests;

#[derive(Debug, Deserialize)]
pub struct GeofenceFix {
    species: String,
    rule: String,
    country_code: String,
    admin1_region_code: Option<String>,
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
) -> Result<bool, Error> {
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
    if let Some(allowed_countries) = geofence_from_full_string.get("allow") {
        // Do geofence if given country not in allowed country
        if !allowed_countries.is_empty() {
            if !allowed_countries.contains_key(_country) {
                return Ok(true);
            } else {
                // Get states from given country
                if let Some(allowed_admin1_region) = allowed_countries.get(_country) {
                    if let Some(ar) = admin1_region {
                        // Do geofence if admin1_region not in allowed admin1_region.
                        if !allowed_admin1_region.is_empty()
                            && !allowed_admin1_region.contains(&ar.to_string())
                        {
                            return Ok(true);
                        }
                    };
                }
            }
        }
    }

    // Get `block` countries from given geofence_map
    if let Some(blocked_countries) = geofence_from_full_string.get("block") {
        // Do geofence if given country in blocked country
        if !blocked_countries.is_empty() && blocked_countries.contains_key(_country) {
            if let Some(blocked_admin1_regions) = blocked_countries.get(_country) {
                if blocked_admin1_regions.is_empty() {
                    return Ok(true);
                }
                if let Some(ar) = admin1_region {
                    // Do geofence if given admin1_region in blocked admin1_region
                    if blocked_admin1_regions.contains(&ar.to_string()) {
                        return Ok(true);
                    };
                };
            }
        }
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
pub fn roll_up_labels_to_first_matching_level(
    labels: &[String],
    scores: &[f64],
    country: Option<&str>,
    admin1_region: Option<&str>,
    target_taxonomy_levels: &Vec<String>,
    non_blank_threshold: &f64,
    taxonomy_map: &HashMap<String, String>,
    geofence_map: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>,
    enable_geofence: bool,
) -> Result<Option<(String, f64, String)>, Error> {
    // Find if there is invalid taxonomy level
    let expected_target_taxonomy_levels = vec![
        "species".to_string(),
        "genus".to_string(),
        "family".to_string(),
        "order".to_string(),
        "class".to_string(),
        "kingdom".to_string(),
    ];
    let set_expected_target_taxonomy_levels: HashSet<_> =
        expected_target_taxonomy_levels.iter().collect();
    let set_target_taxonomy_levels: HashSet<_> = target_taxonomy_levels.iter().collect();
    let has_unknown_taxonomy: HashSet<_> = set_target_taxonomy_levels
        .difference(&set_expected_target_taxonomy_levels)
        .collect();
    if !has_unknown_taxonomy.is_empty() {
        return Err(Error::GeofenceInvalidValue(format!(
            "Unexpected target taxonomy level(s): {:?}. Expected only from the set: {:?}",
            has_unknown_taxonomy, expected_target_taxonomy_levels
        )));
    };

    for taxonomy_level in target_taxonomy_levels {
        let mut accumulated_scores = HashMap::new();
        for (label, score) in labels.iter().zip(scores.iter()) {
            let roll_up_label = get_ancestor_at_level(label, taxonomy_level, taxonomy_map)?;
            if let Some(r_label) = roll_up_label {
                let new_score = accumulated_scores.get(&r_label).unwrap_or(&0.0) + score;
                accumulated_scores.insert(r_label, new_score);
            }
        }

        let mut max_rollup_label = "";
        let mut max_rollup_score = 0.0;
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

///
/// Geofences animal prediction in a country or admin1_region
///
/// Under the hood, this also rolls up the labels every time it encounters a
/// geofenced label.
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
///   - taxonomy_map:
///       Map that map taxa to labels.
///   - geofence_map:
///       Map that has full class species string as keys, array of `allow` as values
///   - enable_geofence:
///       Whether geofencing is enabled
///
pub fn geofence_animal_classification(
    labels: &[String],
    scores: &[f64],
    country: Option<&str>,
    admin1_region: Option<&str>,
    taxonomy_map: &HashMap<String, String>,
    geofence_map: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>,
    enable_geofence: bool,
) -> Result<GeofenceResult, Error> {
    if should_geofence(
        &labels[0],
        country,
        admin1_region,
        geofence_map,
        enable_geofence,
    )? {
        let rollup = roll_up_labels_to_first_matching_level(
            labels,
            scores,
            country,
            admin1_region,
            &vec![
                "family".to_string(),
                "order".to_string(),
                "class".to_string(),
                "kingdom".to_string(),
            ],
            &(scores[0] - 1e-10),
            taxonomy_map,
            geofence_map,
            enable_geofence,
        )?;
        if let Some((r_label, r_score, r_source)) = rollup {
            Ok(GeofenceResult::new(
                r_label,
                r_score,
                format!("classifier+geofence+{}", &r_source[11..]),
            ))
        } else {
            Ok(GeofenceResult::new(
                constants::classification::UNKNOWN.to_string(),
                scores[0],
                "classifier+geofence+rollup_failed".to_string(),
            ))
        }
    } else {
        Ok(GeofenceResult::new(
            labels[0].to_string(),
            scores[0],
            "classifier".to_string(),
        ))
    }
}

pub fn fix_geofence_base<P: AsRef<Path>>(
    base_map: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>,
    csv_path: P,
) -> Result<HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>, Error> {
    let mut geofence = base_map.clone();

    let file = File::open(csv_path)?;
    let mut reader = Reader::from_reader(file);

    for result in reader.deserialize() {
        let fix: GeofenceFix = result?;
        let label = fix.species.to_lowercase();
        let label_parts: Vec<String> = label.split(";").map(|x| x.to_string()).collect();
        if label_parts.len() != 5 {
            return Err(Error::GeofenceInvalidValue(
                "Fixes should provide only species-level rules.".to_string(),
            ));
        }
        let rule = fix.rule.to_lowercase();
        if !["allow", "block"].contains(&rule.as_str()) {
            return Err(Error::GeofenceInvalidValue(
                "Rule types should be either `allow` or `block`.".to_string(),
            ));
        }
        let country = fix.country_code;
        let state = fix.admin1_region_code.unwrap_or("".to_string());
        if rule == "allow" {
            if !geofence.contains_key(&label) {
                continue;
            }
            if geofence.get(&label).and_then(|v| v.get("allow")).is_none() {
                continue;
            }
            if state.is_empty() {
                if let Some(map) = geofence.get_mut(&label).and_then(|v| v.get_mut("allow")) {
                    map.entry(country).or_insert_with(|| vec![]);
                }
            } else {
                let allow_map = geofence
                    .get_mut(&label)
                    .and_then(|v| v.get_mut("allow"))
                    .and_then(|v| v.get_mut(&country));
                match allow_map {
                    Some(rule) => {
                        if rule.is_empty() {
                            continue;
                        } else {
                            let set: HashSet<String> = rule.clone().into_iter().collect();
                            let new_set: HashSet<String> = vec![state].into_iter().collect();
                            *rule = set.union(&new_set).cloned().collect();
                        }
                    }
                    None => {
                        geofence
                            .entry(label)
                            .or_default()
                            .entry("allow".to_string())
                            .or_default()
                            .entry(country.clone())
                            .or_insert_with(|| vec![state]);
                    }
                }
            }
        } else {
            if !geofence.contains_key(&label)
                || geofence.get(&label).and_then(|v| v.get("block")).is_none()
            {
                geofence
                    .entry(label.clone())
                    .or_default()
                    .entry("block".to_string())
                    .or_default()
                    .entry(country.clone())
                    .or_insert_with(|| {
                        if state.is_empty() {
                            vec![]
                        } else {
                            vec![state.clone()]
                        }
                    });
            }
            if state.is_empty() {
                if let Some(map) = geofence.get_mut(&label).and_then(|v| v.get_mut("block")) {
                    map.entry(country).or_insert_with(|| vec![]);
                }
            } else {
                let allow_map = geofence
                    .get_mut(&label)
                    .and_then(|v| v.get_mut("block"))
                    .and_then(|v| v.get_mut(&country));
                match allow_map {
                    Some(rule) => {
                        if rule.is_empty() {
                            continue;
                        } else {
                            let set: HashSet<String> = rule.clone().into_iter().collect();
                            let new_set: HashSet<String> = vec![state].into_iter().collect();
                            *rule = set.union(&new_set).cloned().collect();
                        }
                    }
                    None => {
                        geofence
                            .entry(label)
                            .or_default()
                            .entry("block".to_string())
                            .or_default()
                            .entry(country.clone())
                            .or_insert_with(|| vec![state]);
                    }
                }
            }
        }
    }

    Ok(geofence)
}
