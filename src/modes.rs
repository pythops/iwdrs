use std::fmt::Display;

use strum::EnumString;

#[derive(Debug, Clone, Copy, PartialEq, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum Mode {
    Station,
    Ap,
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Ap => write!(f, "ap"),
            Mode::Station => write!(f, "station"),
        }
    }
}
