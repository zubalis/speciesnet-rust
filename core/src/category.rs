use std::{fmt::Display, str::FromStr};

use serde::{
    Deserialize,
    de::{self, Visitor},
};

use crate::error::Error;

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

impl TryFrom<i64> for Category {
    type Error = Error;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Animal),
            2 => Ok(Self::Human),
            3 => Ok(Self::Vehicle),
            _other => Err(Error::CategoryIndexOutOfRange(_other as f64)),
        }
    }
}

impl TryFrom<f64> for Category {
    type Error = Error;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        match value {
            1f64 => Ok(Self::Animal),
            2f64 => Ok(Self::Human),
            3f64 => Ok(Self::Vehicle),
            _other => Err(Error::CategoryIndexOutOfRange(_other)),
        }
    }
}

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
