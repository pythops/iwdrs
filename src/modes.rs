use std::fmt::Display;

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
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

impl TryFrom<&str> for Mode {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "station" => Ok(Mode::Station),
            "ap" => Ok(Mode::Ap),
            _ => Err("Unkown mode"),
        }
    }
}
