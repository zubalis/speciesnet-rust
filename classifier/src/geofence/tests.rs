#![allow(clippy::bool_assert_comparison)]

use std::collections::HashMap;
use std::env::current_dir;
use std::sync::LazyLock;

use serde_json::json;

use crate::{constants::classification, error::Error};

use super::{
    GeofenceResult, fix_geofence_base, geofence_animal_classification,
    roll_up_labels_to_first_matching_level, should_geofence,
};

const BLANK: &str = "f1856211-cfb7-4a5b-9158-c0f72fd09ee6;;;;;;blank";
const BLANK_FC: &str = ";;;;";
const HUMAN: &str =
    "990ae9dd-7a59-4344-afcb-1b7b21368000;mammalia;primates;hominidae;homo;sapiens;human";
const HUMAN_FC: &str = "mammalia;primates;hominidae;homo;sapiens";
const VEHICLE: &str = "e2895ed5-780b-48f6-8a11-9e27cb594511;;;;;;vehicle";
const VEHICLE_FC: &str = ";;;;";
const LION: &str =
    "ddf59264-185a-4d35-b647-2785792bdf54;mammalia;carnivora;felidae;panthera;leo;lion";
const LION_FC: &str = "mammalia;carnivora;felidae;panthera;leo";
const PANTHERA_GENUS: &str =
    "fbb23d07-6677-43db-b650-f99ac452c50f;mammalia;carnivora;felidae;panthera;;panthera species";
const PANTHERA_GENUS_FC: &str = "mammalia;carnivora;felidae;panthera;";
const FELIDAE_FAMILY: &str =
    "df8514b0-10a5-411f-8ed6-0f415e8153a3;mammalia;carnivora;felidae;;;cat family";
const FELIDAE_FAMILY_FC: &str = "mammalia;carnivora;felidae;;";
const CARNIVORA_ORDER: &str =
    "eeeb5d26-2a47-4d01-a3de-10b33ec0aee4;mammalia;carnivora;;;;carnivorous mammal";
const CARNIVORA_ORDER_FC: &str = "mammalia;carnivora;;;";
const MAMMALIA_CLASS: &str = "f2d233e3-80e3-433d-9687-e29ecc7a467a;mammalia;;;;;mammal";
const MAMMALIA_CLASS_FC: &str = "mammalia;;;;";
const ANIMAL_KINGDOM: &str = "1f689929-883d-4dae-958c-3d57ab5b6c16;;;;;;animal";
const ANIMAL_KINGDOM_FC: &str = ";;;;";
const BROWN_BEAR: &str =
    "330bb1e9-84d6-4e41-afa9-938aee17ea29;mammalia;carnivora;ursidae;ursus;arctos;brown bear";
const BROWN_BEAR_FC: &str = "mammalia;carnivora;ursidae;ursus;arctos";
const POLAR_BEAR: &str =
    "e7f83bf6-df2c-4ce0-97fc-2f233df23ec4;mammalia;carnivora;ursidae;ursus;maritimus;polar bear";
const POLAR_BEAR_FC: &str = "mammalia;carnivora;ursidae;ursus;maritimus";
const GIANT_PANDA: &str = "85662682-67c1-4ecb-ba05-ba12e2df6b65;mammalia;carnivora;ursidae;ailuropoda;melanoleuca;giant panda";
const GIANT_PANDA_FC: &str = "mammalia;carnivora;ursidae;ailuropoda;melanoleuca";
const URSUS_GENUS: &str =
    "5a0f5e3f-c634-4b86-910a-b105cb526a24;mammalia;carnivora;ursidae;ursus;;ursus species";
const URSUS_GENUS_FC: &str = "mammalia;carnivora;ursidae;ursus;";
const URSIDAE_FAMILY: &str =
    "ec1a70f4-41c0-4aba-9150-292fb2b7a324;mammalia;carnivora;ursidae;;;bear family";
const URSIDAE_FAMILY_FC: &str = "mammalia;carnivora;ursidae;;";
const PUMA: &str =
    "9c564562-9429-405c-8529-04cff7752282;mammalia;carnivora;felidae;puma;concolor;puma";
const PUMA_FC: &str = "mammalia;carnivora;felidae;puma;concolor";
const SAND_CAT: &str =
    "e588253d-d61d-4149-a96c-8c245927a80f;mammalia;carnivora;felidae;felis;margarita;sand cat";
const SAND_CAT_FC: &str = "mammalia;carnivora;felidae;felis;margarita";
const UNKNOWN: &str = "unknown;unknown;abc;def;;;";
const UNKNOWN_FC: &str = "unknown;abc;def;;";

static GEOFENCE_MAP: LazyLock<HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>> =
    LazyLock::new(|| {
        let json = json!(
            {
                LION_FC: {
                    "allow": {
                        "KEN": [],
                        "TZA": [],
                    }
                },
                PANTHERA_GENUS_FC: {
                    "allow": {
                        "KEN": [],
                        "TZA": [],
                        "USA": ["AK", "CA"],
                    }
                },
                FELIDAE_FAMILY_FC: {
                    "allow": {
                        "FRA": [],
                        "KEN": [],
                        "TZA": [],
                        "USA": [],
                    },
                    "block": {
                        "FRA": [],
                        "USA": ["NY"],
                    },
                },
                SAND_CAT_FC: {
                    "block": {
                        "AUS": [],
                    },
                },
                URSIDAE_FAMILY_FC: {
                    "block": {
                        "GBR": [],
                    },
                },
                UNKNOWN_FC: {
                    "block": {
                        "USA": [],
                    }
                }
            }
        );

        serde_json::from_value(json).unwrap()
    });

static TAXONOMY_MAP: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    let json = json!(
        {
            BLANK_FC: BLANK,
            HUMAN_FC: HUMAN,
            VEHICLE_FC: VEHICLE,
            LION_FC: LION,
            PANTHERA_GENUS_FC: PANTHERA_GENUS,
            FELIDAE_FAMILY_FC: FELIDAE_FAMILY,
            CARNIVORA_ORDER_FC: CARNIVORA_ORDER,
            MAMMALIA_CLASS_FC: MAMMALIA_CLASS,
            ANIMAL_KINGDOM_FC: ANIMAL_KINGDOM,
            BROWN_BEAR_FC: BROWN_BEAR,
            POLAR_BEAR_FC: POLAR_BEAR,
            GIANT_PANDA_FC: GIANT_PANDA,
            URSUS_GENUS_FC: URSUS_GENUS,
            URSIDAE_FAMILY_FC: URSIDAE_FAMILY,
        }
    );

    serde_json::from_value(json).unwrap()
});

#[test]
fn test_should_geofence_fn() -> Result<(), Error> {
    // Test disable geofencing
    assert_eq!(
        should_geofence(LION, None, None, &GEOFENCE_MAP, false)?,
        false
    );

    // Test country is None
    assert_eq!(
        should_geofence(LION, None, None, &GEOFENCE_MAP, true)?,
        false
    );

    // Test label not in geofence json
    assert_eq!(
        should_geofence(PUMA, Some("USA"), None, &GEOFENCE_MAP, true)?,
        false
    );
    assert_eq!(
        should_geofence(PUMA, Some("USA"), Some("CA"), &GEOFENCE_MAP, true)?,
        false
    );

    // Test `allow` rule in geofence json
    assert_eq!(
        should_geofence(LION, Some("GBR"), None, &GEOFENCE_MAP, true)?,
        true
    );
    assert_eq!(
        should_geofence(LION, Some("KEN"), None, &GEOFENCE_MAP, true)?,
        false
    );
    assert_eq!(
        should_geofence(PANTHERA_GENUS, Some("USA"), None, &GEOFENCE_MAP, true)?,
        false
    );
    assert_eq!(
        should_geofence(PANTHERA_GENUS, Some("USA"), Some("NY"), &GEOFENCE_MAP, true)?,
        true
    );
    assert_eq!(
        should_geofence(PANTHERA_GENUS, Some("USA"), Some("CA"), &GEOFENCE_MAP, true)?,
        false
    );

    // Test `block` rule in geofence json
    assert_eq!(
        should_geofence(FELIDAE_FAMILY, Some("FRA"), None, &GEOFENCE_MAP, true)?,
        true
    );
    assert_eq!(
        should_geofence(FELIDAE_FAMILY, Some("TZA"), None, &GEOFENCE_MAP, true)?,
        false
    );
    assert_eq!(
        should_geofence(FELIDAE_FAMILY, Some("USA"), Some("CA"), &GEOFENCE_MAP, true)?,
        false
    );
    assert_eq!(
        should_geofence(FELIDAE_FAMILY, Some("USA"), Some("NY"), &GEOFENCE_MAP, true)?,
        true
    );
    assert_eq!(
        should_geofence(SAND_CAT, Some("GBR"), None, &GEOFENCE_MAP, true)?,
        false
    );
    assert_eq!(
        should_geofence(SAND_CAT, Some("AUS"), None, &GEOFENCE_MAP, true)?,
        true
    );

    // Test invalid labels
    {
        let invalid_label = "uuid;class;order;family;genus;species";
        let invalid_label_parts = invalid_label.split(";").collect::<Vec<_>>();
        assert!(matches!(
            should_geofence(invalid_label, Some("AUS"), None, &GEOFENCE_MAP, true),
            Err(
                Error::InvalidLabel(
                    label_parts,
                    label
                )
            ) if label_parts == invalid_label_parts.len().to_string() && label == *invalid_label
        ));
    }

    Ok(())
}

#[test]
fn test_roll_up_labels_to_first_matching_level_fn() -> Result<(), Error> {
    let labels = vec![
        BROWN_BEAR.to_string(),
        POLAR_BEAR.to_string(),
        GIANT_PANDA.to_string(),
        BLANK.to_string(),
        LION.to_string(),
        HUMAN.to_string(),
        ANIMAL_KINGDOM.to_string(),
    ];
    // Test rollups to species level
    let rollup_fn = |scores| {
        roll_up_labels_to_first_matching_level(
            &labels,
            scores,
            None,
            None,
            &vec!["species".to_string()],
            &0.9,
            &TAXONOMY_MAP,
            &GEOFENCE_MAP,
            true,
        )
    };
    assert_eq!(rollup_fn(&[0.8, 0.2, 0.0, 0.0, 0.0, 0.0, 0.0])?, None);
    assert_eq!(rollup_fn(&[0.9, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0])?, None);
    assert_eq!(
        rollup_fn(&[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])?,
        Some((
            BROWN_BEAR.to_string(),
            1.0,
            "classifier+rollup_to_species".to_string()
        ))
    );

    // Test rollups to genus level
    let rollup_fn = |scores| {
        roll_up_labels_to_first_matching_level(
            &labels,
            scores,
            None,
            None,
            &vec!["genus".to_string()],
            &0.9,
            &TAXONOMY_MAP,
            &GEOFENCE_MAP,
            true,
        )
    };
    assert_eq!(rollup_fn(&[0.6, 0.2, 0.01, 0.01, 0.01, 0.01, 0.01])?, None);
    assert_eq!(
        rollup_fn(&[0.7, 0.25, 0.01, 0.01, 0.01, 0.01, 0.01])?,
        Some((
            URSUS_GENUS.to_string(),
            0.95,
            "classifier+rollup_to_genus".to_string()
        ))
    );

    // Test rollups to family level
    let rollup_fn = |scores| {
        roll_up_labels_to_first_matching_level(
            &labels,
            scores,
            None,
            None,
            &vec!["family".to_string()],
            &0.8,
            &TAXONOMY_MAP,
            &GEOFENCE_MAP,
            true,
        )
    };
    assert_eq!(rollup_fn(&[0.4, 0.1, 0.1, 0.1, 0.1, 0.0, 0.0])?, None);
    assert_eq!(
        rollup_fn(&[0.4, 0.21, 0.2, 0.0, 0.0, 0.0, 0.0])?,
        Some((
            URSIDAE_FAMILY.to_string(),
            0.81,
            "classifier+rollup_to_family".to_string()
        ))
    );

    // Test rollups to order level
    let rollup_fn = |scores| {
        roll_up_labels_to_first_matching_level(
            &labels,
            scores,
            None,
            None,
            &vec!["order".to_string()],
            &0.8,
            &TAXONOMY_MAP,
            &GEOFENCE_MAP,
            true,
        )
    };
    assert_eq!(rollup_fn(&[0.3, 0.2, 0.1, 0.1, 0.1, 0.0, 0.0])?, None);
    // expect result using sum to avoid floating point accuracy error
    assert_eq!(
        rollup_fn(&[0.3, 0.2, 0.1, 0.1, 0.23, 0.0, 0.0])?,
        Some((
            CARNIVORA_ORDER.to_string(),
            [0.3, 0.2, 0.1, 0.23].iter().sum(),
            "classifier+rollup_to_order".to_string()
        ))
    );

    // Test rollups to class level
    let rollup_fn = |scores| {
        roll_up_labels_to_first_matching_level(
            &labels,
            scores,
            None,
            None,
            &vec!["class".to_string()],
            &0.8,
            &TAXONOMY_MAP,
            &GEOFENCE_MAP,
            true,
        )
    };
    assert_eq!(rollup_fn(&[0.2, 0.2, 0.1, 0.1, 0.1, 0.1, 0.0])?, None);
    // expect result using sum to avoid floating point accuracy error
    assert_eq!(
        rollup_fn(&[0.2, 0.2, 0.1, 0.1, 0.22, 0.1, 0.0])?,
        Some((
            MAMMALIA_CLASS.to_string(),
            [0.2, 0.2, 0.1, 0.1, 0.22].iter().sum(),
            "classifier+rollup_to_class".to_string()
        ))
    );

    // Test rollups to kingdom level
    let rollup_fn = |scores| {
        roll_up_labels_to_first_matching_level(
            &labels,
            scores,
            None,
            None,
            &vec!["kingdom".to_string()],
            &0.81,
            &TAXONOMY_MAP,
            &GEOFENCE_MAP,
            true,
        )
    };
    assert_eq!(rollup_fn(&[0.2, 0.2, 0.1, 0.1, 0.1, 0.1, 0.1])?, None);
    // expect result using sum to avoid floating point accuracy error
    assert_eq!(
        rollup_fn(&[0.2, 0.2, 0.1, 0.1, 0.23, 0.1, 0.1])?,
        Some((
            ANIMAL_KINGDOM.to_string(),
            [0.2, 0.2, 0.1, 0.1, 0.23, 0.1].iter().sum(),
            "classifier+rollup_to_kingdom".to_string()
        ))
    );

    // Test rollups when multiple taxonomy levels are specified
    let rollup_fn = |scores| {
        roll_up_labels_to_first_matching_level(
            &labels,
            scores,
            None,
            None,
            &vec![
                "genus".to_string(),
                "family".to_string(),
                "order".to_string(),
                "class".to_string(),
                "kingdom".to_string(),
            ],
            &0.75,
            &TAXONOMY_MAP,
            &GEOFENCE_MAP,
            true,
        )
    };
    // expect result using sum to avoid floating point accuracy error
    assert_eq!(
        rollup_fn(&[0.6, 0.1, 0.1, 0.1, 0.1, 0.0, 0.0])?,
        Some((
            URSIDAE_FAMILY.to_string(),
            [0.6, 0.1, 0.1].iter().sum(),
            "classifier+rollup_to_family".to_string()
        ))
    );

    // Test rollups when multiple score sums pass the non blank threshold
    let rollup_fn = |scores| {
        roll_up_labels_to_first_matching_level(
            &labels,
            scores,
            None,
            None,
            &vec!["species".to_string()],
            &0.1,
            &TAXONOMY_MAP,
            &GEOFENCE_MAP,
            true,
        )
    };
    assert_eq!(
        rollup_fn(&[0.2, 0.3, 0.15, 0.0, 0.35, 0.0, 0.0])?,
        Some((
            LION.to_string(),
            0.35,
            "classifier+rollup_to_species".to_string()
        ))
    );

    // Test rollups with geofencing
    let rollup_fn = |scores| {
        roll_up_labels_to_first_matching_level(
            &labels,
            scores,
            Some("GBR"),
            None,
            &vec![
                "species".to_string(),
                "genus".to_string(),
                "family".to_string(),
                "order".to_string(),
                "class".to_string(),
            ],
            &0.4,
            &TAXONOMY_MAP,
            &GEOFENCE_MAP,
            true,
        )
    };
    // expect result using sum to avoid floating point accuracy error
    assert_eq!(
        rollup_fn(&[0.1, 0.2, 0.2, 0.45, 0.0, 0.0, 0.0])?,
        Some((
            CARNIVORA_ORDER.to_string(),
            [0.1, 0.2, 0.2].iter().sum(),
            "classifier+rollup_to_order".to_string()
        ))
    );

    // Test rollups to invalid levels
    let rollup_fn = |scores| {
        roll_up_labels_to_first_matching_level(
            &labels,
            scores,
            Some("GBR"),
            None,
            &vec!["invalid_level".to_string()],
            &0.3,
            &TAXONOMY_MAP,
            &GEOFENCE_MAP,
            true,
        )
    };
    assert!(rollup_fn(&[0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1]).is_err());
    Ok(())
}

#[test]
fn test_geofence_animal_classification_fn() -> Result<(), Error> {
    let labels = vec![
        LION.to_string(),
        POLAR_BEAR.to_string(),
        BLANK.to_string(),
        FELIDAE_FAMILY.to_string(),
    ];

    let unknown_labels = vec![UNKNOWN.to_string()];

    // Test when no geofencing is needed
    let geofence_classification_fn = |scores| {
        geofence_animal_classification(
            &labels,
            scores,
            Some("TZA"),
            None,
            &TAXONOMY_MAP,
            &GEOFENCE_MAP,
            true,
        )
    };
    assert_eq!(
        geofence_classification_fn(&[0.4, 0.3, 0.2, 0.1])?,
        Some(GeofenceResult {
            label: LION.to_string(),
            score: 0.4,
            source: "classifier".to_string()
        })
    );

    //Test with geofencing and rollup to family level or above
    let geofence_classification_fn = |scores| {
        geofence_animal_classification(
            &labels,
            scores,
            Some("USA"),
            None,
            &TAXONOMY_MAP,
            &GEOFENCE_MAP,
            true,
        )
    };
    assert_eq!(
        geofence_classification_fn(&[0.4, 0.3, 0.2, 0.1])?,
        Some(GeofenceResult {
            label: FELIDAE_FAMILY.to_string(),
            score: 0.5,
            source: "classifier+geofence+rollup_to_family".to_string()
        })
    );
    let geofence_classification_fn = |scores| {
        geofence_animal_classification(
            &labels,
            scores,
            Some("USA"),
            Some("NY"),
            &TAXONOMY_MAP,
            &GEOFENCE_MAP,
            true,
        )
    };
    assert_eq!(
        geofence_classification_fn(&[0.4, 0.3, 0.2, 0.1])?,
        Some(GeofenceResult {
            label: CARNIVORA_ORDER.to_string(),
            score: [0.4, 0.3, 0.1].iter().sum(),
            source: "classifier+geofence+rollup_to_order".to_string()
        })
    );

    // Test with geofencing and rollup to unknown
    let geofence_classification_fn = |scores| {
        geofence_animal_classification(
            &unknown_labels,
            scores,
            Some("USA"),
            None,
            &TAXONOMY_MAP,
            &GEOFENCE_MAP,
            true,
        )
    };
    assert_eq!(
        geofence_classification_fn(&[0.4, 0.3, 0.2, 0.1])?,
        Some(GeofenceResult {
            label: classification::UNKNOWN.to_string(),
            score: 0.4,
            source: "classifier+geofence+rollup_failed".to_string()
        })
    );

    Ok(())
}

#[test]
fn test_fix_geofence_base_fn() -> Result<(), Error> {
    let fix_path = current_dir()?
        .join("..")
        .join("assets")
        .join("geofence_fixes_test.csv")
        .canonicalize()?
        .to_str()
        .unwrap()
        .to_string();

    let fixed_base = fix_geofence_base(&GEOFENCE_MAP, fix_path.as_str())?;

    assert_eq!(
        fixed_base
            .get(LION_FC)
            .unwrap()
            .get("allow")
            .unwrap()
            .contains_key("USA"),
        true
    );
    assert_eq!(
        fixed_base
            .get(LION_FC)
            .unwrap()
            .get("block")
            .unwrap()
            .contains_key("THA"),
        true
    );
    assert_eq!(
        fixed_base
            .get(LION_FC)
            .unwrap()
            .get("block")
            .unwrap()
            .contains_key("TZA"),
        true
    );
    assert_eq!(
        fixed_base
            .get(LION_FC)
            .unwrap()
            .get("block")
            .unwrap()
            .get("USA")
            .unwrap(),
        &vec!["JFC"]
    );
    Ok(())
}
