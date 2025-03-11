use std::collections::HashMap;
use super::should_geofence;
#[test]
fn can_geofence() {
    let label = "62bab9ee-acd2-467a-b405-bf941927fc45;aves;passeriformes;muscicapidae;cercotrichas;;cercotrichas species";
    let country = Some("ABC");
    let admin1_region = None;
    let mut geofence_map = HashMap::new();
    let mut allow_map = HashMap::new();
    let mut country_map: HashMap<String, Vec<String>> = HashMap::new();
    country_map.insert("CAN".to_string(), vec![]);
    country_map.insert("MEX".to_string(), vec![]);
    country_map.insert("USA".to_string(), vec![]);
    allow_map.insert("allow".to_string(), country_map);
    geofence_map.insert("aves;passeriformes;muscicapidae;cercotrichas;".to_string(), allow_map);

    let result = should_geofence(&label, country, admin1_region, &geofence_map);

    match result {
        Ok(r) => {
            assert_eq!(r, true);
        }
        Err(e) => {
            assert!(false);
        }
    }
}