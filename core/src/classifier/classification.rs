use serde::{
    Deserialize, Deserializer, Serialize,
    de::{MapAccess, Visitor},
    ser::SerializeStruct,
};

/// A list of classifications stored separately as a list of labels and scores.
#[derive(Debug, PartialEq, Clone)]
pub struct ClassificationBundle {
    labels: Vec<String>,
    scores: Vec<f64>,
}

/// Struct for storing a classification from the model.
#[derive(Debug, PartialEq, Clone)]
pub struct Classification {
    label: String,
    score: f64,
}

impl ClassificationBundle {
    pub fn new(labels: Vec<String>, scores: Vec<f64>) -> Self {
        Self { labels, scores }
    }
    pub fn labels(&self) -> &Vec<String> {
        &self.labels
    }

    pub fn scores(&self) -> &Vec<f64> {
        &self.scores
    }
}

impl Classification {
    pub fn new(label: String, score: f64) -> Self {
        Self { label, score }
    }
    pub fn label(&self) -> &String {
        &self.label
    }

    pub fn score(&self) -> &f64 {
        &self.score
    }
}

impl Serialize for ClassificationBundle {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("ClassificationBundle", 2)?;

        s.serialize_field("classes", &self.labels)?;
        s.serialize_field("scores", &self.scores)?;

        s.end()
    }
}

impl<'de> Deserialize<'de> for ClassificationBundle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
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
                let mut scores: Option<Vec<f64>> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "classes" => labels = Some(map.next_value()?),
                        "scores" => scores = Some(map.next_value()?),
                        _ => {}
                    }
                }

                let labels = labels.ok_or_else(|| serde::de::Error::missing_field("classes"))?;
                let scores = scores.ok_or_else(|| serde::de::Error::missing_field("scores"))?;

                if labels.len() != scores.len() {
                    Err(serde::de::Error::custom(
                        "`labels` size and `scores` size have to be the same.",
                    ))?;
                }

                Ok(ClassificationBundle { labels, scores })
            }
        }

        deserializer.deserialize_map(ClassificationBundleVisitor)
    }
}
