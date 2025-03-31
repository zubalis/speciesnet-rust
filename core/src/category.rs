use std::{fmt::Display, str::FromStr};

use serde::{
    Deserialize, Serialize,
    de::{self, Visitor},
};

use crate::{
    error::Error,
    macros::{category_try_from_floats, category_try_from_integers},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Animal,
    Human,
    Vehicle,
}

impl<'de> Deserialize<'de> for Category {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct CategoryVisitor;

        impl Visitor<'_> for CategoryVisitor {
            type Value = Category;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("`1`, `2`, or `3`.")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match v {
                    "1" => Ok(Category::Animal),
                    "2" => Ok(Category::Human),
                    "3" => Ok(Category::Vehicle),
                    _other => Err(de::Error::unknown_variant(v, &["1", "2", "3"])),
                }
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v.as_str() {
                    "1" => Ok(Category::Animal),
                    "2" => Ok(Category::Human),
                    "3" => Ok(Category::Vehicle),
                    _other => Err(de::Error::unknown_variant(&v, &["1", "2", "3"])),
                }
            }
        }

        deserializer.deserialize_identifier(CategoryVisitor)
    }
}

impl Serialize for Category {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let serialized_str = serializer.serialize_str(&self.to_string())?;
        Ok(serialized_str)
    }
}

impl FromStr for Category {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "1" => Ok(Self::Animal),
            "2" => Ok(Self::Human),
            "3" => Ok(Self::Vehicle),
            _other => Err(Error::CategoryParseError(_other.to_string())),
        }
    }
}

category_try_from_integers!(i8, i16, i32, i64, u8, u16, u32, u64);
category_try_from_floats!(f32, f64);

impl Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Animal => "animal",
                Self::Human => "human",
                Self::Vehicle => "vehicle",
            }
        )
    }
}

impl Category {
    pub fn index(&self) -> String {
        match self {
            Self::Animal => "1".to_string(),
            Self::Human => "2".to_string(),
            Self::Vehicle => "3".to_string(),
        }
    }
}
