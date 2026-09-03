//! Hide credentials from debug output.

use std::{fmt, ops::Deref};

pub struct Hide<T>(T);

impl<T> Deref for Hide<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> fmt::Debug for Hide<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("***")
    }
}

impl<'de, T> serde::Deserialize<'de> for Hide<T>
where
    T: serde::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self(T::deserialize(deserializer)?))
    }
}
