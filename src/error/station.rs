use std::fmt::Display;

use strum::{EnumMessage, EnumString};
use thiserror::Error;

#[derive(Debug, EnumString, EnumMessage, Error)]
pub enum ScanError {
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
}

impl Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_detailed_message().unwrap())
    }
}

#[derive(Debug, EnumString, EnumMessage, Error)]
pub enum DisconnectError {
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
        serialize = "net.connman.iwd.NotConnected",
        message = "NotConnected",
        detailed_message = "Not connected"
    )]
    NotConnected,
}

impl Display for DisconnectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_detailed_message().unwrap())
    }
}

#[derive(Debug, EnumString, EnumMessage, Error)]
pub enum StationDiagnosticsError {
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
        serialize = "net.connman.iwd.NotConnected",
        message = "NotConnected",
        detailed_message = "Not connected"
    )]
    NotConnected,
}

impl Display for StationDiagnosticsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_detailed_message().unwrap())
    }
}
