pub mod taxonomy;
#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::error::Error;
use thiserror::Error;

#[derive(Debug, Error)]
enum GeofenceError {
    #[error("Given full class is not found in Geofence map")]
    NotFound,
    #[error("Geofence map is invalid")]
    InvalidFormat,
}

/**
* Check if label is allowed withing a given country
* Parameters:
*   label:
*       Animal label from classification result
*   country:
*       Country code (in ISO 3166-1 alpha-3 format)
*   admin1_region:
*       First-level administrative division (in ISO 3166-2 format)
*   geofence_map:
*       Map that has full class species string as keys, array of `allow` as values
**/
fn should_geofence(
    label: &str,
    country: Option<&str>,
    admin1_region: Option<&str>,
    geofence_map: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>,
) -> Result<bool, Box<dyn Error>> {
    // Do not geofence if country not given
    let _country = match country {
        Some(c) => c,
        None => return Ok(false),
    };

    // Get full string, exclude uuid and scientific name
    let full_string_class = taxonomy::get_full_class_string(label)?;
    if geofence_map.get(&full_string_class).is_none() {
        return Ok(false);
    };

    let geofence_from_full_string = geofence_map
        .get(&full_string_class)
        .ok_or_else(|| GeofenceError::NotFound)?;

    // Get `allow` countries from given geofence_map
    match geofence_from_full_string.get("allow") {
        Some(allowed_countries) => {
            // Do geofence if given country not in allowed country
            if !allowed_countries.is_empty() {
                if !allowed_countries.contains_key(_country) {
                    return Ok(true);
                } else {
                    // Get states from given country
                    let allowed_admin1_region = allowed_countries
                        .get(_country)
                        .ok_or_else(|| GeofenceError::InvalidFormat)?;
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
                    let blocked_admin1_regions = blocked_countries
                        .get(_country)
                        .ok_or_else(|| GeofenceError::InvalidFormat)?;
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
            }
        }
        None => (),
    }
    Ok(false)
}

fn roll_up_labels_to_first_matching_level() {}

fn geofence_classification() {}
