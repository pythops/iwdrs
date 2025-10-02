use std::fmt::Display;

use strum::{EnumMessage, EnumString};
use thiserror::Error;

#[derive(Debug, EnumString, EnumMessage, Error)]
pub enum ConnectError {
    #[strum(
        serialize = "net.connman.iwd.Aborted",
        message = "Aborted",
        detailed_message = "Operation aborted"
    )]
    Aborted,
    #[strum(
        serialize = "net.connman.iwd.Busy",
        message = "InProgress",
        detailed_message = "Operation already in progress"
    )]
    Busy,
    #[strum(
        serialize = "net.connman.iwd.Failed",
        message = "Failed",
        detailed_message = "Operation failed"
    )]
    Failed,
    #[strum(
        serialize = "net.connman.iwd.NoAgent",
        message = "NoAgent",
        detailed_message = "No Agent registered"
    )]
    NoAgent,
    #[strum(
        serialize = "net.connman.iwd.NotSupported",
        message = "NotSupported",
        detailed_message = "Operation not supported"
    )]
    NotSupported,
    #[strum(
        serialize = "net.connman.iwd.InProgress",
        message = "InProgress",
        detailed_message = "Operation already in progress"
    )]
    InProgress,
    #[strum(
        serialize = "net.connman.iwd.NotConfigured",
        message = "NotConfigured",
        detailed_message = "Not configured"
    )]
    NotConfigured,
    #[strum(
        serialize = "net.connman.iwd.InvalidFormat",
        message = "InvalidFormat",
        detailed_message = "Argument format is invalid"
    )]
    InvalidFormat,
}

impl Display for ConnectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_detailed_message().unwrap())
    }
}
