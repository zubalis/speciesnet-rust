use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use speciesnet_core::{
    classifier::ClassificationBundle,
    constants::{classification, source},
    detector::{Category, Detection},
    ensemble::GeofenceResult,
};

use crate::{
    error::Error,
    geofence::{
        fix_geofence_base, geofence_animal_classification, roll_up_labels_to_first_matching_level,
        taxonomy::get_full_class_string,
    },
};

pub mod error;
pub mod geofence;
pub mod input;

#[derive(Debug, Clone)]
pub struct SpeciesNetEnsemble {
    geofence_map: HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>,
    taxonomy_map: HashMap<String, String>,
}

impl SpeciesNetEnsemble {
    pub fn new<P: AsRef<Path>>(
        geofence_base_path: P,
        taxonomy_path: P,
        geofence_fix_path: Option<P>,
    ) -> Result<Self, Error> {
        // Load geofence and fix
        let geofence_file = File::open(geofence_base_path)?;
        let geofence_reader = BufReader::new(geofence_file);
        let geofence_map: HashMap<String, HashMap<String, HashMap<String, Vec<String>>>> =
            serde_json::from_reader(geofence_reader)?;

        let fixed_geofence_map = match geofence_fix_path {
            Some(p) => fix_geofence_base(&geofence_map, p)?,
            None => geofence_map,
        };

        // Load taxonomy
        let taxonomy_file = File::open(taxonomy_path)?;
        let taxonomy_reader = BufReader::new(taxonomy_file);
        let taxonomies: Vec<String> = taxonomy_reader.lines().map_while(Result::ok).collect();
        let mut taxonomy_map: HashMap<String, String> = HashMap::new();
        for t in taxonomies {
            if ![
                classification::BLANK.to_string(),
                classification::VEHICLE.to_string(),
                classification::UNKNOWN.to_string(),
            ]
            .contains(&t)
            {
                taxonomy_map.insert(get_full_class_string(&t)?, t.clone());
            }
        }

        Ok(Self {
            geofence_map: fixed_geofence_map,
            taxonomy_map,
        })
    }

    pub fn ensemble(
        &self,
        detections: &[Detection],
        classifications: &ClassificationBundle,
        country: Option<String>,
        admin1_region: Option<String>,
    ) -> Result<GeofenceResult, Error> {
        if classifications.scores().is_empty() || classifications.labels().is_empty() {
            return Err(Error::EmptyClassifications);
        }

        let top_classification_class = classifications.labels().first().unwrap();
        let classes = classifications.labels();
        let top_classification_score = *classifications.scores().first().unwrap();
        let scores = classifications.scores();
        let top_detection_class = if detections.is_empty() {
            Category::Animal
        } else {
            *detections.first().unwrap().category()
        };
        let top_detection_score = if detections.is_empty() {
            0.0
        } else {
            detections.first().unwrap().confidence()
        };

        if top_detection_class == Category::Human {
            // Threshold #1a: high-confidence HUMAN detections.
            if top_detection_score > 0.7 {
                return Ok(GeofenceResult::new(
                    classification::HUMAN.to_string(),
                    top_detection_score,
                    source::DETECTOR.to_string(),
                ));
            }

            // Threshold #1b: mid-confidence HUMAN detections + high-confidence
            // HUMAN/VEHICLE classifications.
            if top_detection_score > 0.2
                && [
                    classification::HUMAN.to_string(),
                    classification::VEHICLE.to_string(),
                ]
                .contains(top_classification_class)
                && top_classification_score > 0.5
            {
                return Ok(GeofenceResult::new(
                    classification::HUMAN.to_string(),
                    top_classification_score,
                    source::CLASSIFIER.to_string(),
                ));
            }
        }

        if top_detection_class == Category::Vehicle {
            // Threshold #2a: mid-confidence VEHICLE detections + high-confidence HUMAN
            // classifications.
            if top_detection_score > 0.2
                && top_classification_class == &classification::HUMAN.to_string()
                && top_classification_score > 0.5
            {
                return Ok(GeofenceResult::new(
                    classification::HUMAN.to_string(),
                    top_classification_score,
                    source::CLASSIFIER.to_string(),
                ));
            }

            // Threshold #2b: high-confidence VEHICLE detections.
            if top_detection_score > 0.7 {
                return Ok(GeofenceResult::new(
                    classification::VEHICLE.to_string(),
                    top_detection_score,
                    source::DETECTOR.to_string(),
                ));
            }

            // Threshold #2c: mid-confidence VEHICLE detections + high-confidence VEHICLE
            // classifications.
            if top_detection_score > 0.2
                && top_classification_class == &classification::VEHICLE.to_string()
                && top_classification_score > 0.4
            {
                return Ok(GeofenceResult::new(
                    classification::VEHICLE.to_string(),
                    top_classification_score,
                    source::CLASSIFIER.to_string(),
                ));
            }
        }

        // Threshold #3a: high-confidence BLANK "detections" + high-confidence BLANK
        // classifications.
        if top_detection_score < 0.2
            && top_classification_class == &classification::BLANK.to_string()
            && top_classification_score > 0.5
        {
            return Ok(GeofenceResult::new(
                classification::BLANK.to_string(),
                top_classification_score,
                source::CLASSIFIER.to_string(),
            ));
        }

        // Threshold #3b: extra-high-confidence BLANK classifications.
        if top_classification_class == &classification::BLANK.to_string()
            && top_classification_score > 0.99
        {
            return Ok(GeofenceResult::new(
                classification::BLANK.to_string(),
                top_classification_score,
                source::CLASSIFIER.to_string(),
            ));
        }

        if ![
            classification::BLANK.to_string(),
            classification::HUMAN.to_string(),
            classification::VEHICLE.to_string(),
        ]
        .contains(top_classification_class)
        {
            // Threshold #4a: extra-high-confidence ANIMAL classifications.
            if top_classification_score > 0.8 {
                return geofence_animal_classification(
                    classes,
                    scores,
                    country.as_deref(),
                    admin1_region.as_deref(),
                    &self.taxonomy_map,
                    &self.geofence_map,
                    true,
                );
            }

            // Threshold #4b: high-confidence ANIMAL classifications + mid-confidence
            // ANIMAL detections.
            if top_classification_score > 0.65
                && top_detection_class == Category::Animal
                && top_detection_score > 0.2
            {
                return geofence_animal_classification(
                    classes,
                    scores,
                    country.as_deref(),
                    admin1_region.as_deref(),
                    &self.taxonomy_map,
                    &self.geofence_map,
                    true,
                );
            }
        }

        // Threshold #5a: high-confidence ANIMAL rollups.
        let roll_up = roll_up_labels_to_first_matching_level(
            classes,
            scores,
            country.as_deref(),
            admin1_region.as_deref(),
            &vec![
                "genus".to_string(),
                "family".to_string(),
                "order".to_string(),
                "class".to_string(),
                "kingdom".to_string(),
            ],
            &0.65,
            &self.taxonomy_map,
            &self.geofence_map,
            true,
        )?;

        if let Some((label, score, source)) = roll_up {
            return Ok(GeofenceResult::new(label, score, source));
        }

        // Threshold #5b: mid-confidence ANIMAL detections.
        if top_detection_class == Category::Animal && top_detection_score > 0.5 {
            return Ok(GeofenceResult::new(
                classification::ANIMAL.to_string(),
                top_detection_score,
                source::DETECTOR.to_string(),
            ));
        }

        Ok(GeofenceResult::new(
            classification::UNKNOWN.to_string(),
            top_classification_score,
            source::CLASSIFIER.to_string(),
        ))
    }
}
