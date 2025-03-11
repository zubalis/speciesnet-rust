use std::collections::HashMap;
use std::error::Error;
use serde_json::{from_str};
use super::should_geofence;
#[test]
fn can_geofence() -> Result<(), Box<dyn Error>> {
    let label = "62bab9ee-acd2-467a-b405-bf941927fc45;aves;passeriformes;muscicapidae;cercotrichas;;cercotrichas species";
    let country = Some("ABC");
    let admin1_region = None;
    let geofence_json = r#"{
        "aves;passeriformes;muscicapidae;cercotrichas;": {
            "allow": {
                "CAN": [],
                "MEX": [],
                "USA": []
            }
        }
    }"#;
    let geofence_map: HashMap<String, HashMap<String, HashMap<String, Vec<String>>>> = from_str(geofence_json)?;

    let result = should_geofence(&label, country, admin1_region, &geofence_map);

    assert_eq!(result?, true);
    Ok(())
}