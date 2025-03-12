use super::{TaxonomyError, get_full_class_string};
use once_cell::sync::Lazy;
use serde_json::{from_value, json};
use std::collections::HashMap;
use std::error::Error;

static BLANK: &'static str = "f1856211-cfb7-4a5b-9158-c0f72fd09ee6;;;;;;blank";
static BLANK_FC: &'static str = ";;;;";
static HUMAN: &'static str =
    "990ae9dd-7a59-4344-afcb-1b7b21368000;mammalia;primates;hominidae;homo;sapiens;human";
static HUMAN_FC: &'static str = "mammalia;primates;hominidae;homo;sapiens";
static VEHICLE: &'static str = "e2895ed5-780b-48f6-8a11-9e27cb594511;;;;;;vehicle";
static VEHICLE_FC: &'static str = ";;;;";
static LION: &'static str =
    "ddf59264-185a-4d35-b647-2785792bdf54;mammalia;carnivora;felidae;panthera;leo;lion";
static LION_FC: &'static str = "mammalia;carnivora;felidae;panthera;leo";
static PANTHERA_GENUS: &'static str =
    "fbb23d07-6677-43db-b650-f99ac452c50f;mammalia;carnivora;felidae;panthera;;panthera species";
static PANTHERA_GENUS_FC: &'static str = "mammalia;carnivora;felidae;panthera;";
static FELIDAE_FAMILY: &'static str =
    "df8514b0-10a5-411f-8ed6-0f415e8153a3;mammalia;carnivora;felidae;;;cat family";
static FELIDAE_FAMILY_FC: &'static str = "mammalia;carnivora;felidae;;";
static CARNIVORA_ORDER: &'static str =
    "eeeb5d26-2a47-4d01-a3de-10b33ec0aee4;mammalia;carnivora;;;;carnivorous mammal";
static CARNIVORA_ORDER_FC: &'static str = "mammalia;carnivora;;;";
static MAMMALIA_CLASS: &'static str = "f2d233e3-80e3-433d-9687-e29ecc7a467a;mammalia;;;;;mammal";
static MAMMALIA_CLASS_FC: &'static str = "mammalia;;;;";
static ANIMAL_KINGDOM: &'static str = "1f689929-883d-4dae-958c-3d57ab5b6c16;;;;;;animal";
static ANIMAL_KINGDOM_FC: &'static str = ";;;;";
static BROWN_BEAR: &'static str =
    "330bb1e9-84d6-4e41-afa9-938aee17ea29;mammalia;carnivora;ursidae;ursus;arctos;brown bear";
static BROWN_BEAR_FC: &'static str = "mammalia;carnivora;ursidae;ursus;arctos";
static POLAR_BEAR: &'static str =
    "e7f83bf6-df2c-4ce0-97fc-2f233df23ec4;mammalia;carnivora;ursidae;ursus;maritimus;polar bear";
static POLAR_BEAR_FC: &'static str = "mammalia;carnivora;ursidae;ursus;maritimus";
static GIANT_PANDA: &'static str = "85662682-67c1-4ecb-ba05-ba12e2df6b65;mammalia;carnivora;ursidae;ailuropoda;melanoleuca;giant panda";
static GIANT_PANDA_FC: &'static str = "mammalia;carnivora;ursidae;ailuropoda;melanoleuca";
static URSUS_GENUS: &'static str =
    "5a0f5e3f-c634-4b86-910a-b105cb526a24;mammalia;carnivora;ursidae;ursus;;ursus species";
static URSUS_GENUS_FC: &'static str = "mammalia;carnivora;ursidae;ursus;";
static URSIDAE_FAMILY: &'static str =
    "ec1a70f4-41c0-4aba-9150-292fb2b7a324;mammalia;carnivora;ursidae;;;bear family";
static URSIDAE_FAMILY_FC: &'static str = "mammalia;carnivora;ursidae;;";
static PUMA: &'static str =
    "9c564562-9429-405c-8529-04cff7752282;mammalia;carnivora;felidae;puma;concolor;puma";
static PUMA_FC: &'static str = "mammalia;carnivora;felidae;puma;concolor";
static SAND_CAT: &'static str =
    "e588253d-d61d-4149-a96c-8c245927a80f;mammalia;carnivora;felidae;felis;margarita;sand cat";
static SAND_CAT_FC: &'static str = "mammalia;carnivora;felidae;felis;margarita";

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
