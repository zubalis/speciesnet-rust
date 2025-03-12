use super::{TaxonomyError, get_full_class_string};
use once_cell::sync::Lazy;
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

static TAXONOMY_MAP: Lazy<HashMap<String, String>> = Lazy::new(|| {
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
    let taxonomy_json = from_value(json).unwrap();
    taxonomy_json
});

#[test]
fn test_get_full_class_string_fn() -> Result<(), Box<dyn Error>> {
    // Test BLANK/HUMAN/VEHICLE.
    assert_eq!(get_full_class_string(BLANK)?, BLANK_FC);
    assert_eq!(get_full_class_string(HUMAN)?, HUMAN_FC);
    assert_eq!(get_full_class_string(VEHICLE)?, VEHICLE_FC);

    // Test valid labels
    assert_eq!(get_full_class_string(LION)?, LION_FC);
    assert_eq!(get_full_class_string(PANTHERA_GENUS)?, PANTHERA_GENUS_FC);
    assert_eq!(get_full_class_string(FELIDAE_FAMILY)?, FELIDAE_FAMILY_FC);
    assert_eq!(get_full_class_string(CARNIVORA_ORDER)?, CARNIVORA_ORDER_FC);
    assert_eq!(get_full_class_string(MAMMALIA_CLASS)?, MAMMALIA_CLASS_FC);
    assert_eq!(get_full_class_string(ANIMAL_KINGDOM)?, ANIMAL_KINGDOM_FC);

    // Test invalid labels
    {
        let invalid_label = "uuid;class;order;family;genus;species";
        let invalid_label_parts = invalid_label.split(";").collect::<Vec<_>>();
        assert_eq!(
            get_full_class_string(invalid_label),
            Err(TaxonomyError::InvalidLabel(
                invalid_label_parts.len().to_string(),
                invalid_label.to_string()
            ))
        );
    }
    {
        let invalid_label = "uuid;class;order;family;genus;species;common_name;extra";
        let invalid_label_parts = invalid_label.split(";").collect::<Vec<_>>();
        assert_eq!(
            get_full_class_string(invalid_label),
            Err(TaxonomyError::InvalidLabel(
                invalid_label_parts.len().to_string(),
                invalid_label.to_string()
            ))
        );
    }
    Ok(())
}
