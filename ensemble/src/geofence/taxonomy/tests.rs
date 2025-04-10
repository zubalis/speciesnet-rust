use std::collections::HashMap;
use std::sync::LazyLock;

use serde_json::json;

use super::{get_ancestor_at_level, get_full_class_string};
use crate::error::Error;

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
fn test_get_full_class_string_fn() -> Result<(), Error> {
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

        assert!(matches!(
            get_full_class_string(invalid_label),
            Err(Error::InvalidLabel(
                parts_len, label
            )) if parts_len == invalid_label_parts.len().to_string() && label == *invalid_label
        ));
    }

    {
        let invalid_label = "uuid;class;order;family;genus;species;common_name;extra";
        let invalid_label_parts = invalid_label.split(";").collect::<Vec<_>>();

        let result = get_full_class_string(invalid_label);

        assert!(matches!(
            result,
            Err(Error::InvalidLabel(parts_len, label))
            if parts_len == invalid_label_parts.len().to_string()
                && label == *invalid_label
        ));
    }

    Ok(())
}

#[test]
fn test_get_ancestor_at_level_fn() -> Result<(), Error> {
    // Test all ancestors of LION
    assert_eq!(
        get_ancestor_at_level(LION, "species", &TAXONOMY_MAP)?,
        Some(LION.to_string())
    );
    assert_eq!(
        get_ancestor_at_level(LION, "genus", &TAXONOMY_MAP)?,
        Some(PANTHERA_GENUS.to_string())
    );
    assert_eq!(
        get_ancestor_at_level(LION, "family", &TAXONOMY_MAP)?,
        Some(FELIDAE_FAMILY.to_string())
    );
    assert_eq!(
        get_ancestor_at_level(LION, "order", &TAXONOMY_MAP)?,
        Some(CARNIVORA_ORDER.to_string())
    );
    assert_eq!(
        get_ancestor_at_level(LION, "class", &TAXONOMY_MAP)?,
        Some(MAMMALIA_CLASS.to_string())
    );
    assert_eq!(
        get_ancestor_at_level(LION, "kingdom", &TAXONOMY_MAP)?,
        Some(ANIMAL_KINGDOM.to_string())
    );

    // Test all ancestors of PANTHERA_GENUS
    assert_eq!(
        get_ancestor_at_level(PANTHERA_GENUS, "species", &TAXONOMY_MAP)?,
        None
    );
    assert_eq!(
        get_ancestor_at_level(PANTHERA_GENUS, "genus", &TAXONOMY_MAP)?,
        Some(PANTHERA_GENUS.to_string())
    );
    assert_eq!(
        get_ancestor_at_level(PANTHERA_GENUS, "family", &TAXONOMY_MAP)?,
        Some(FELIDAE_FAMILY.to_string())
    );
    assert_eq!(
        get_ancestor_at_level(PANTHERA_GENUS, "order", &TAXONOMY_MAP)?,
        Some(CARNIVORA_ORDER.to_string())
    );
    assert_eq!(
        get_ancestor_at_level(PANTHERA_GENUS, "class", &TAXONOMY_MAP)?,
        Some(MAMMALIA_CLASS.to_string())
    );
    assert_eq!(
        get_ancestor_at_level(PANTHERA_GENUS, "kingdom", &TAXONOMY_MAP)?,
        Some(ANIMAL_KINGDOM.to_string())
    );

    // Test all ancestors of FELIDAE_FAMILY
    assert_eq!(
        get_ancestor_at_level(FELIDAE_FAMILY, "species", &TAXONOMY_MAP)?,
        None
    );
    assert_eq!(
        get_ancestor_at_level(FELIDAE_FAMILY, "genus", &TAXONOMY_MAP)?,
        None
    );
    assert_eq!(
        get_ancestor_at_level(FELIDAE_FAMILY, "family", &TAXONOMY_MAP)?,
        Some(FELIDAE_FAMILY.to_string())
    );
    assert_eq!(
        get_ancestor_at_level(FELIDAE_FAMILY, "order", &TAXONOMY_MAP)?,
        Some(CARNIVORA_ORDER.to_string())
    );
    assert_eq!(
        get_ancestor_at_level(FELIDAE_FAMILY, "class", &TAXONOMY_MAP)?,
        Some(MAMMALIA_CLASS.to_string())
    );
    assert_eq!(
        get_ancestor_at_level(FELIDAE_FAMILY, "kingdom", &TAXONOMY_MAP)?,
        Some(ANIMAL_KINGDOM.to_string())
    );

    // Test all ancestors of CARNIVORA_ORDER
    assert_eq!(
        get_ancestor_at_level(CARNIVORA_ORDER, "species", &TAXONOMY_MAP)?,
        None
    );
    assert_eq!(
        get_ancestor_at_level(CARNIVORA_ORDER, "genus", &TAXONOMY_MAP)?,
        None
    );
    assert_eq!(
        get_ancestor_at_level(CARNIVORA_ORDER, "family", &TAXONOMY_MAP)?,
        None
    );
    assert_eq!(
        get_ancestor_at_level(CARNIVORA_ORDER, "order", &TAXONOMY_MAP)?,
        Some(CARNIVORA_ORDER.to_string())
    );
    assert_eq!(
        get_ancestor_at_level(CARNIVORA_ORDER, "class", &TAXONOMY_MAP)?,
        Some(MAMMALIA_CLASS.to_string())
    );
    assert_eq!(
        get_ancestor_at_level(CARNIVORA_ORDER, "kingdom", &TAXONOMY_MAP)?,
        Some(ANIMAL_KINGDOM.to_string())
    );

    // Test all ancestors of MAMMALIA_CLASS
    assert_eq!(
        get_ancestor_at_level(MAMMALIA_CLASS, "species", &TAXONOMY_MAP)?,
        None
    );
    assert_eq!(
        get_ancestor_at_level(MAMMALIA_CLASS, "genus", &TAXONOMY_MAP)?,
        None
    );
    assert_eq!(
        get_ancestor_at_level(MAMMALIA_CLASS, "family", &TAXONOMY_MAP)?,
        None
    );
    assert_eq!(
        get_ancestor_at_level(MAMMALIA_CLASS, "order", &TAXONOMY_MAP)?,
        None
    );
    assert_eq!(
        get_ancestor_at_level(MAMMALIA_CLASS, "class", &TAXONOMY_MAP)?,
        Some(MAMMALIA_CLASS.to_string())
    );
    assert_eq!(
        get_ancestor_at_level(MAMMALIA_CLASS, "kingdom", &TAXONOMY_MAP)?,
        Some(ANIMAL_KINGDOM.to_string())
    );

    // Test all ancestors of ANIMAL_KINGDOM
    assert_eq!(
        get_ancestor_at_level(ANIMAL_KINGDOM, "species", &TAXONOMY_MAP)?,
        None
    );
    assert_eq!(
        get_ancestor_at_level(ANIMAL_KINGDOM, "genus", &TAXONOMY_MAP)?,
        None
    );
    assert_eq!(
        get_ancestor_at_level(ANIMAL_KINGDOM, "family", &TAXONOMY_MAP)?,
        None
    );
    assert_eq!(
        get_ancestor_at_level(ANIMAL_KINGDOM, "order", &TAXONOMY_MAP)?,
        None
    );
    assert_eq!(
        get_ancestor_at_level(ANIMAL_KINGDOM, "class", &TAXONOMY_MAP)?,
        None
    );
    assert_eq!(
        get_ancestor_at_level(ANIMAL_KINGDOM, "kingdom", &TAXONOMY_MAP)?,
        Some(ANIMAL_KINGDOM.to_string())
    );

    // Test all ancestors of BLANK
    assert_eq!(
        get_ancestor_at_level(BLANK, "species", &TAXONOMY_MAP)?,
        None
    );
    assert_eq!(get_ancestor_at_level(BLANK, "genus", &TAXONOMY_MAP)?, None);
    assert_eq!(get_ancestor_at_level(BLANK, "family", &TAXONOMY_MAP)?, None);
    assert_eq!(get_ancestor_at_level(BLANK, "order", &TAXONOMY_MAP)?, None);
    assert_eq!(get_ancestor_at_level(BLANK, "class", &TAXONOMY_MAP)?, None);
    assert_eq!(
        get_ancestor_at_level(BLANK, "kingdom", &TAXONOMY_MAP)?,
        None
    );

    // Test all ancestors of HUMAN, when its genus, family and order are missing from
    // the mock taxonomy mapping
    assert_eq!(
        get_ancestor_at_level(HUMAN, "species", &TAXONOMY_MAP)?,
        Some(HUMAN.to_string())
    );
    assert_eq!(get_ancestor_at_level(HUMAN, "genus", &TAXONOMY_MAP)?, None);
    assert_eq!(get_ancestor_at_level(HUMAN, "family", &TAXONOMY_MAP)?, None);
    assert_eq!(get_ancestor_at_level(HUMAN, "order", &TAXONOMY_MAP)?, None);
    assert_eq!(
        get_ancestor_at_level(HUMAN, "class", &TAXONOMY_MAP)?,
        Some(MAMMALIA_CLASS.to_string())
    );
    assert_eq!(
        get_ancestor_at_level(HUMAN, "kingdom", &TAXONOMY_MAP)?,
        Some(ANIMAL_KINGDOM.to_string())
    );

    // Test all ancestors of VEHICLE
    assert_eq!(
        get_ancestor_at_level(VEHICLE, "species", &TAXONOMY_MAP)?,
        None
    );
    assert_eq!(
        get_ancestor_at_level(VEHICLE, "genus", &TAXONOMY_MAP)?,
        None
    );
    assert_eq!(
        get_ancestor_at_level(VEHICLE, "family", &TAXONOMY_MAP)?,
        None
    );
    assert_eq!(
        get_ancestor_at_level(VEHICLE, "order", &TAXONOMY_MAP)?,
        None
    );
    assert_eq!(
        get_ancestor_at_level(VEHICLE, "class", &TAXONOMY_MAP)?,
        None
    );
    assert_eq!(
        get_ancestor_at_level(VEHICLE, "kingdom", &TAXONOMY_MAP)?,
        None
    );

    // Test all ancestors of an unseen species
    let unseen_species = "uuid;class;order;family;genus;species;common_name";
    assert_eq!(
        get_ancestor_at_level(unseen_species, "species", &TAXONOMY_MAP)?,
        None
    );
    assert_eq!(
        get_ancestor_at_level(unseen_species, "genus", &TAXONOMY_MAP)?,
        None
    );
    assert_eq!(
        get_ancestor_at_level(unseen_species, "family", &TAXONOMY_MAP)?,
        None
    );
    assert_eq!(
        get_ancestor_at_level(unseen_species, "order", &TAXONOMY_MAP)?,
        None
    );
    assert_eq!(
        get_ancestor_at_level(unseen_species, "class", &TAXONOMY_MAP)?,
        None
    );
    assert_eq!(
        get_ancestor_at_level(unseen_species, "kingdom", &TAXONOMY_MAP)?,
        Some(ANIMAL_KINGDOM.to_string())
    );

    // Test invalid labels
    {
        let invalid_label = "uuid;class;order;family;genus;species";
        let invalid_label_parts = invalid_label.split(";").collect::<Vec<_>>();

        let result = get_ancestor_at_level(invalid_label, "kingdom", &TAXONOMY_MAP);

        assert!(matches!(
            result,
            Err(Error::InvalidLabel(len, label))
            if len == invalid_label_parts.len().to_string() &&
                label == invalid_label
        ));
    }

    {
        let invalid_label = "uuid;class;order;family;genus;species;common_name;extra";
        let invalid_label_parts = invalid_label.split(";").collect::<Vec<_>>();

        let result = get_ancestor_at_level(invalid_label, "kingdom", &TAXONOMY_MAP);

        assert!(matches!(
            result,
            Err(Error::InvalidLabel(len, label))
            if len == invalid_label_parts.len().to_string() &&
                label == invalid_label
        ));
    }

    // Test invalid taxonomy level name
    {
        let invalid_taxonomy_level = "incorrect_name";

        let result = get_ancestor_at_level(LION, invalid_taxonomy_level, &TAXONOMY_MAP);

        assert!(matches!(
            result,
            Err(Error::InvalidTaxonomyLevel(level)) if level == *invalid_taxonomy_level
        ));
    }

    Ok(())
}
