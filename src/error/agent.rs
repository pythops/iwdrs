use std::fmt::Display;

use thiserror::Error;

#[derive(Debug, Error)]
pub struct Canceled();

impl Display for Canceled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Canceled")
    }
}

impl From<Canceled> for zbus::fdo::Error {
    fn from(value: Canceled) -> Self {
        zbus::fdo::Error::Failed(value.to_string())
    }
}
