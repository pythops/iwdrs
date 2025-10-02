use std::fmt::Display;

use strum::{EnumMessage, EnumString};
use thiserror::Error;

#[derive(Debug, EnumString, EnumMessage, Error)]
pub enum AccessPointStartError {
    #[strum(
        serialize = "net.connman.iwd.Failed",
        message = "Failed",
        detailed_message = "Operation failed"
    )]
    Failed,
    #[strum(
        serialize = "net.connman.iwd.InvalidArguments",
        message = "InvalidArguments",
        detailed_message = "Argument type is wrong"
    )]
    InvalidArguments,
    #[strum(
        serialize = "net.connman.iwd.AlreadyExists",
        message = "AlreadyExists",
        detailed_message = "Object already exists"
    )]
    AlreadyExists,
}

impl Display for AccessPointStartError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_detailed_message().unwrap())
    }
}

#[derive(Debug, EnumString, EnumMessage, Error)]
pub enum AccessPointStopError {
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
        serialize = "net.connman.iwd.InvalidArguments",
        message = "InvalidArguments",
        detailed_message = "Argument type is wrong"
    )]
    InvalidArguments,
}

impl Display for AccessPointStopError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_detailed_message().unwrap())
    }
}

#[derive(Debug, EnumString, EnumMessage, Error)]
pub enum StartProfileError {
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
        serialize = "net.connman.iwd.InvalidArguments",
        message = "InvalidArguments",
        detailed_message = "Argument type is wrong"
    )]
    InvalidArguments,
    #[strum(
        serialize = "net.connman.iwd.AlreadyExists",
        message = "AlreadyExists",
        detailed_message = "Object already exists"
    )]
    AlreadyExists,
    #[strum(
        serialize = "net.connman.iwd.NotFound",
        message = "NotFound",
        detailed_message = "Object not found"
    )]
    NotFound,
}

impl Display for StartProfileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_detailed_message().unwrap())
    }
}

#[derive(Debug, EnumString, EnumMessage, Error)]
pub enum ScanError {
    #[strum(
        serialize = "net.connman.iwd.NotAvailable",
        message = "NotAvailable",
        detailed_message = "Operation not available"
    )]
    NotAvailable,
    #[strum(
        serialize = "net.connman.iwd.NotSupported",
        message = "NotSupported",
        detailed_message = "Operation not supported"
    )]
    NotSupported,
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
