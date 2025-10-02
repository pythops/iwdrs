use std::{
    cmp::Reverse,
    collections::HashMap,
    ops::{Bound, RangeBounds},
    sync::Arc,
};

use zvariant::{OwnedObjectPath, Value};

use zbus::{Connection, Proxy, interface};

use crate::{
    error::{
        Result as IWDResult,
        station::{DisconnectError, ScanError},
    },
    network::Network,
};

#[derive(Debug, Clone)]
pub struct Station {
    pub(crate) connection: Arc<Connection>,
    pub(crate) dbus_path: OwnedObjectPath,
}

#[derive(Debug, Clone)]
pub struct StationDiagnostics {
    pub(crate) connection: Arc<Connection>,
    pub(crate) dbus_path: OwnedObjectPath,
}

impl Station {
    pub(crate) fn new(connection: Arc<Connection>, dbus_path: OwnedObjectPath) -> Self {
        Self {
            connection,
            dbus_path,
        }
    }

    pub(crate) async fn proxy<'a>(&self) -> Result<zbus::Proxy<'a>, zbus::Error> {
        Proxy::new(
            &self.connection,
            "net.connman.iwd",
            self.dbus_path.clone(),
            "net.connman.iwd.Station",
        )
        .await
    }

    pub async fn is_scanning(&self) -> zbus::Result<bool> {
        let proxy = self.proxy().await?;
        let is_scanning: bool = proxy.get_property("Scanning").await?;
        Ok(is_scanning)
    }

    pub async fn state(&self) -> zbus::Result<String> {
        let proxy = self.proxy().await?;
        let state: String = proxy.get_property("State").await?;
        Ok(state)
    }

    pub async fn connected_network(&self) -> zbus::Result<Option<Network>> {
        let state = self.state().await?;
        if state == "connected" {
            let proxy = self.proxy().await?;
            let network_path: OwnedObjectPath = proxy.get_property("ConnectedNetwork").await?;
            let network = Network::new(self.connection.clone(), network_path);
            return Ok(Some(network));
        }
        Ok(None)
    }

    pub async fn scan(&self) -> IWDResult<(), ScanError> {
        let proxy = self.proxy().await?;
        proxy.call_method("Scan", &()).await?;
        Ok(())
    }

    pub async fn disconnect(&self) -> IWDResult<(), DisconnectError> {
        let proxy = self.proxy().await?;
        proxy.call_method("Disconnect", &()).await?;
        Ok(())
    }

    pub async fn discovered_networks(&self) -> zbus::Result<Vec<(Network, i16)>> {
        let proxy = self.proxy().await?;
        let networks = proxy.call_method("GetOrderedNetworks", &()).await?;

        let body = networks.body();
        let objects: Vec<(OwnedObjectPath, i16)> = body.deserialize()?;

        let networks: Vec<(Network, i16)> = objects
            .iter()
            .map(|(path, signal_strength)| {
                let network = Network::new(self.connection.clone(), path.clone());
                (network, signal_strength.to_owned())
            })
            .collect();

        Ok(networks)
    }

    /// Register the agent object to receive signal strength level change notifications on the provided agent.
    /// The "levels" parameters decides the thresholds in dBm that will generate a call to the
    /// [`SignalLevelAgent::changed`] method whenever current RSSI crosses any of the values.  The number and distance between
    /// requested threshold values is a compromise between resolution and the frequency of system wakeups and
    /// context-switches that are going to be occurring to update the client's signal meter.  Only one agent
    /// can be registered at any time.
    pub async fn register_signal_level_agent(
        &self,
        mut levels: Vec<i16>,
        agent: impl SignalLevelAgent,
    ) -> zbus::Result<()> {
        // Signal level boundaries should be sorted
        levels.sort_by_key(|signal_level| Reverse(*signal_level));

        let proxy = self.proxy().await?;

        let interface = SignalLevelInterface {
            agent,
            connection: self.connection.clone(),
            levels: levels.clone(),
        };

        self.connection
            .object_server()
            .at(self.dbus_path.clone(), interface)
            .await?;

        proxy
            .call_method("RegisterSignalLevelAgent", &(&self.dbus_path, levels))
            .await?;
        Ok(())
    }
}

impl StationDiagnostics {
    pub(crate) fn new(connection: Arc<Connection>, dbus_path: OwnedObjectPath) -> Self {
        Self {
            connection,
            dbus_path,
        }
    }

    pub(crate) async fn proxy<'a>(&self) -> zbus::Result<zbus::Proxy<'a>> {
        Proxy::new(
            &self.connection,
            "net.connman.iwd",
            self.dbus_path.clone(),
            "net.connman.iwd.StationDiagnostic",
        )
        .await
    }

    pub async fn get(&self) -> zbus::Result<HashMap<String, String>> {
        let proxy = self.proxy().await?;
        let diagnostic = proxy.call_method("GetDiagnostics", &()).await?;

        let body = diagnostic.body();
        let body: HashMap<String, Value> = body.deserialize()?;
        let body = body
            .into_iter()
            .map(|(k, v)| match k.as_str() {
                "Frequency" => {
                    let v: u32 = v.try_into().unwrap();
                    (k, v.to_string())
                }
                _ => (k, v.to_string()),
            })
            .collect::<HashMap<String, String>>();

        Ok(body)
    }
}

pub trait SignalLevelAgent: Send + Sync + 'static {
    /// This method gets called when the service daemon unregisters the agent. An agent can use it to do
    /// cleanup tasks. There is no need to unregister the agent, because when this method gets called it has
    /// already been unregistered.
    fn release(&self) {}

    fn changed(&self, station: &Station, signal_level: impl RangeBounds<i16>);
}

pub struct SignalLevelInterface<A> {
    agent: A,
    connection: Arc<Connection>,
    levels: Vec<i16>,
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
    fn changed(&self, station_path: OwnedObjectPath, level_idx: u8) {
        let station = Station {
            connection: self.connection.clone(),
            dbus_path: station_path,
        };

        let level_idx = usize::from(level_idx);
        let max_strength = self
            .levels
            .get(level_idx - 1)
            .map(|level| Bound::Excluded(*level))
            .unwrap_or(Bound::Unbounded);
        let min_strength = self
            .levels
            .get(level_idx)
            .map(|level| Bound::Included(*level))
            .unwrap_or(Bound::Unbounded);

        self.agent.changed(&station, (min_strength, max_strength))
    }
}
