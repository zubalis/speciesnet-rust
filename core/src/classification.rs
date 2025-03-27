use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize};
use serde::de::{MapAccess, Visitor};

#[derive(Debug, PartialEq, Clone)]
pub struct ClassificationBundle {
    pub labels: Vec<String>,
    pub scores: Vec<f32>,
}
#[derive(Debug, PartialEq, Clone)]
pub struct Classification {
    pub label: String,
    pub score: f32,
}

impl Serialize for ClassificationBundle {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("ClassificationBundle", 2)?;

        s.serialize_field("label", &self.labels)?;
        s.serialize_field("scores", &self.scores)?;

        s.end()
    }
}

impl<'de> Deserialize<'de> for ClassificationBundle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        struct ClassificationBundleVisitor;

        impl<'de> Visitor<'de> for ClassificationBundleVisitor {
            type Value = ClassificationBundle;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a JSON object with labels and scores")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut labels: Option<Vec<String>> = None;
                let mut scores: Option<Vec<f32>> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "labels" => labels = Some(map.next_value()?),
                        "scores" => scores = Some(map.next_value()?),
                        _ => {}
                    }
                }

                let labels = labels.ok_or_else(|| serde::de::Error::missing_field("labels"))?;
                let scores = scores.ok_or_else(|| serde::de::Error::missing_field("scores"))?;
                
                if labels.len() != scores.len() {
                    Err(serde::de::Error::custom("`labels` size and `scores` size have to be the same"))?;
                }

                Ok(ClassificationBundle { labels, scores })
            }
        }

        deserializer.deserialize_map(ClassificationBundleVisitor)
    }
}