use strum::EnumMessage;
use thiserror::Error;

pub mod access_point;
pub mod agent;
pub mod network;
pub mod station;

pub type Result<T, E> = std::result::Result<T, IWDError<E>>;

#[derive(Debug, Error)]
pub enum IWDError<T: std::str::FromStr + std::error::Error> {
    OperationError(T),
    ZbusError(zbus::Error),
}

impl<T: std::str::FromStr<Err = strum::ParseError> + std::error::Error + EnumMessage>
    From<zbus::Error> for IWDError<T>
{
    fn from(value: zbus::Error) -> Self {
        let zbus::Error::MethodError(error_name, _error_description, _) = &value else {
            return Self::ZbusError(value);
        };
        match T::from_str(error_name.as_str()) {
            Ok(error) => {
                debug_assert!(
                    _error_description
                        .as_ref()
                        .is_some_and(|err| err.as_str() == error.get_detailed_message().unwrap())
                );

                Self::OperationError(error)
            }
            Err(_) => Self::ZbusError(value),
        }
    }
}
