use thiserror::Error;

/// Error returned when the [`Session::with_monitor`] monitor future terminates.
///
/// [`Session::with_monitor`]: crate::session::Session::with_monitor
#[derive(Debug, Error)]
pub enum MonitorError {
    /// The `InterfacesAdded` signal stream ended unexpectedly.
    #[error("InterfacesAdded signal stream ended")]
    InterfacesAddedStreamEnded,

    /// The `InterfacesRemoved` signal stream ended unexpectedly.
    #[error("InterfacesRemoved signal stream ended")]
    InterfacesRemovedStreamEnded,

    /// A D-Bus error occurred while processing a signal.
    #[error("D-Bus error: {0}")]
    ZbusError(#[from] zbus::Error),
}
