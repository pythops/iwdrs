use std::ops::{Bound, RangeBounds};

use uuid::Uuid;
use zbus::{Connection, interface};
use zvariant::OwnedObjectPath;

use crate::{iwd_interface::IwdInterface, station::Station};

pub trait SignalLevelAgent: Send + Sync + 'static {
    /// This method gets called when the service daemon unregisters the agent. An agent can use it to do
    /// cleanup tasks. There is no need to unregister the agent, because when this method gets called it has
    /// already been unregistered.
    fn release(&self) {}

    fn changed(&self, station: &Station, signal_level: impl RangeBounds<i16>);
}

pub struct SignalLevelInterface<A> {
    pub(super) agent: A,
    pub(super) connection: Connection,
    pub(super) levels: Vec<i16>,
}

#[interface(name = "net.connman.iwd.SignalLevelAgent")]
impl<A: SignalLevelAgent> SignalLevelInterface<A> {
    #[zbus(name = "Release")]
    fn release(&self) {
        self.agent.release();
    }

    /// This method gets called when the signal strength measurement for the device's connected network changes
    /// enough to go from one level to another out of the N ranges defined by the array of (N-1) threshold values
    /// passed to RegisterSignalLevelAgent().  It also gets registered.  The level parameter is in the range from 0
    /// called immediately after the signal level agent is to N, 0 being the strongest signal or above the first
    /// threshold value in the array, and N being the weakest and below the last threshold value.  For example if
    /// RegisterSignalLevelAgent was called with the array [-40, -50, -60], the 'level' parameter of 0 would mean signal
    /// is received at -40 or more dBm and 3 would mean below -60 dBm and might correspond to 1 out of 4 bars on a UI
    /// signal meter.
    #[zbus(name = "Changed")]
    async fn changed(&self, station_path: OwnedObjectPath, level_idx: u8) -> zbus::fdo::Result<()> {
        let station = Station::new(self.connection.clone(), station_path).await?;

        let level_idx = usize::from(level_idx);

        let max_strength = level_idx
            .checked_sub(1)
            .and_then(|level| self.levels.get(level))
            .map(|level| Bound::Excluded(*level))
            .unwrap_or(Bound::Unbounded);
        let min_strength = self
            .levels
            .get(level_idx)
            .map(|level| Bound::Included(*level))
            .unwrap_or(Bound::Unbounded);

        self.agent.changed(&station, (min_strength, max_strength));
        Ok(())
    }
}

pub struct SignalLevelAgentManager {
    pub(crate) dbus_path: OwnedObjectPath,
    pub(crate) station: super::Station,
}

impl SignalLevelAgentManager {
    pub(crate) async fn register_agent(
        station: super::Station,
        interface: SignalLevelInterface<impl SignalLevelAgent>,
    ) -> zbus::Result<Self> {
        let dbus_path = OwnedObjectPath::try_from(format!(
            "/iwdrs/signal_level_agent/{}",
            Uuid::new_v4().as_simple()
        ))?;
        station
            .proxy
            .connection()
            .object_server()
            .at(dbus_path.clone(), interface)
            .await?;

        Ok(Self { dbus_path, station })
    }

    pub async fn unregister(self) -> zbus::Result<()> {
        self.station
            .proxy
            .call_method("UnregisterSignalLevelAgent", &(&self.dbus_path,))
            .await?;
        Ok(())
    }
}
