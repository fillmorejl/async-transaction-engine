use crate::types::errors::MonetaryError;
use serde::{de, Deserialize, Deserializer};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::{AddAssign, SubAssign};
use std::str::FromStr;
use tracing::error;

const DECIMAL_PLACES: usize = 4;
const SCALE: i64 = 10i64.pow(DECIMAL_PLACES as u32);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct Monetary(i64);

impl Monetary {
    pub fn new() -> Self {
        Monetary(0)
    }

    pub fn is_negative(&self) -> bool {
        self.0 < 0
    }

    pub fn checked_add(self, rhs: Monetary) -> Option<Monetary> {
        self.0.checked_add(rhs.0).map(Monetary)
    }

    pub fn checked_sub(self, rhs: Monetary) -> Option<Monetary> {
        self.0.checked_sub(rhs.0).map(Monetary)
    }
}

impl AddAssign<Monetary> for Monetary {
    fn add_assign(&mut self, rhs: Monetary) {
        if let Some(new_val) = self.checked_add(rhs) {
            self.0 = new_val.0;
        } else {
            error!("Monetary AddAssign error: Overflow")
        }
    }
}

impl SubAssign<Monetary> for Monetary {
    fn sub_assign(&mut self, rhs: Monetary) {
        if let Some(new_val) = self.checked_sub(rhs) {
            self.0 = new_val.0;
        } else {
            error!("Monetary SubAssign error: Overflow")
        }
    }
}

impl Display for Monetary {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let sign = if self.0 < 0 { "-" } else { "" };
        let abs = self.0.abs();
        let integer = abs / SCALE;
        let fraction = abs % SCALE;
        write!(formatter, "{}{}.{:0width$}", sign, integer, fraction, width = DECIMAL_PLACES)
    }
}

impl FromStr for Monetary {
    type Err = MonetaryError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.trim();

        if value.is_empty() {
            return Err(MonetaryError::InvalidFormat("Value is an empty string".to_string()));
        }

        //NOTE: I do not really like the allocations but, this routine can be further optimized at a later point
        let parts: Vec<&str> = value.split('.').collect();

        if parts.len() > 2 {
            return Err(MonetaryError::InvalidFormat("Value has more than one decimal point".to_string()));
        }

        let integer: i64 = parts[0].parse().map_err(|error| {
            MonetaryError::InvalidFormat(format!("Value has an invalid integer part: {:?}", error))
        })?;

        let fraction: i64 = if parts.len() == 2 {
            if parts[1].len() > DECIMAL_PLACES {
                return Err(MonetaryError::InvalidFormat("Value has too many decimal places".to_string()));
            }

            let padded = format!("{:0<width$}", parts[1], width = DECIMAL_PLACES);

            padded.parse().map_err(|error| {
                MonetaryError::InvalidFormat(format!("Value has an invalid fraction part: {:?}", error))
            })?
        } else {
            0
        };

        let is_negative = value.starts_with('-');
        let sign = if is_negative { -1 } else { 1 };
        let result = integer.checked_mul(SCALE)
            .and_then(|v| v.checked_add(sign * fraction))
            .ok_or(MonetaryError::Overflow)?;

        Ok(Monetary(result))
    }
}

impl<'de> Deserialize<'de> for Monetary {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Monetary::from_str(&value).map_err(de::Error::custom)
    }
}
