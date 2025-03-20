use std::fmt::Display;

use crate::error::Error;

#[derive(Debug, Clone, Copy)]
pub enum Category {
    Animal,
    Human,
    Vehicle,
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
