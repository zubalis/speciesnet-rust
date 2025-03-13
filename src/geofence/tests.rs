use std::cell::LazyCell;
use super::should_geofence;
use crate::geofence::taxonomy::TaxonomyError;
use serde_json::{from_value, json};
use std::collections::HashMap;
use std::error::Error;

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

const GEOFENCE_MAP: LazyCell<HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>> =
    LazyCell::new(|| {
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
            }
        );
        let unwrap_json: HashMap<String, HashMap<String, HashMap<String, Vec<String>>>> =
            from_value(json).unwrap();
        unwrap_json
    });

#[test]
fn test_should_geofence_fn() -> Result<(), Box<dyn Error>> {
    // Test country is None
    assert_eq!(should_geofence(LION, None, None, &GEOFENCE_MAP)?, false);

    // Test label not in geofence json
    assert_eq!(
        should_geofence(PUMA, Some("USA"), None, &GEOFENCE_MAP)?,
        false
    );
    assert_eq!(
        should_geofence(PUMA, Some("USA"), Some("CA"), &GEOFENCE_MAP)?,
        false
    );

    // Test `allow` rule in geofence json
    assert_eq!(
        should_geofence(LION, Some("GBR"), None, &GEOFENCE_MAP)?,
        true
    );
    assert_eq!(
        should_geofence(LION, Some("KEN"), None, &GEOFENCE_MAP)?,
        false
    );
    assert_eq!(
        should_geofence(PANTHERA_GENUS, Some("USA"), None, &GEOFENCE_MAP)?,
        false
    );
    assert_eq!(
        should_geofence(PANTHERA_GENUS, Some("USA"), Some("NY"), &GEOFENCE_MAP)?,
        true
    );
    assert_eq!(
        should_geofence(PANTHERA_GENUS, Some("USA"), Some("CA"), &GEOFENCE_MAP)?,
        false
    );

    // Test `block` rule in geofence json
    assert_eq!(
        should_geofence(FELIDAE_FAMILY, Some("FRA"), None, &GEOFENCE_MAP)?,
        true
    );
    assert_eq!(
        should_geofence(FELIDAE_FAMILY, Some("TZA"), None, &GEOFENCE_MAP)?,
        false
    );
    assert_eq!(
        should_geofence(FELIDAE_FAMILY, Some("USA"), Some("CA"), &GEOFENCE_MAP)?,
        false
    );
    assert_eq!(
        should_geofence(FELIDAE_FAMILY, Some("USA"), Some("NY"), &GEOFENCE_MAP)?,
        true
    );
    assert_eq!(
        should_geofence(SAND_CAT, Some("GBR"), None, &GEOFENCE_MAP)?,
        false
    );
    assert_eq!(
        should_geofence(SAND_CAT, Some("AUS"), None, &GEOFENCE_MAP)?,
        true
    );

    // Test invalid labels
    {
        let invalid_label = "uuid;class;order;family;genus;species";
        let invalid_label_parts = invalid_label.split(";").collect::<Vec<_>>();
        assert_eq!(
            should_geofence(invalid_label, Some("AUS"), None, &GEOFENCE_MAP),
            Err(TaxonomyError::InvalidLabel(
                invalid_label_parts.len().to_string(),
                invalid_label.to_string()
            ))
        );
    }

    Ok(())
}
