use std::{cmp::Reverse, collections::HashMap, str::FromStr};

use futures_lite::{Stream, StreamExt, stream};
use strum::EnumString;
use zvariant::{OwnedObjectPath, OwnedValue, Value};

use zbus::{Connection, Proxy};

use crate::{
    error::{
        Result as IWDResult,
        station::{DisconnectError, ScanError},
    },
    iwd_interface::{IwdInterface, iwd_interface_impl},
    network::Network,
};

use signal_level_agent::SignalLevelAgentManager;
pub mod signal_level_agent;

iwd_interface_impl!(Station, "net.connman.iwd.Station");

impl Station {
    pub async fn is_scanning(&self) -> zbus::Result<bool> {
        let is_scanning: bool = self.proxy.get_property("Scanning").await?;
        Ok(is_scanning)
    }

    pub async fn wait_for_scan_complete(&self) -> zbus::Result<()> {
        let _ = crate::property_stream::<bool>(self.proxy.clone(), "Scanning")
            .await?
            .skip_while(|scanning| scanning.as_ref().is_ok_and(|scanning| *scanning))
            .next()
            .await
            .ok_or_else(|| zbus::Error::InvalidReply)??;
        Ok(())
    }

    pub async fn state(&self) -> zbus::Result<State> {
        self.state_stream()
            .await?
            .next()
            .await
            .ok_or_else(|| zbus::Error::Unsupported)?
    }

    pub async fn state_stream(
        &self,
    ) -> zbus::Result<impl Stream<Item = zbus::Result<State>> + Unpin + 'static> {
        crate::property_stream(self.proxy.clone(), "State").await
    }

    pub async fn connected_network(&self) -> zbus::Result<Option<Network>> {
        let state = self.state().await?;
        if matches!(state, State::Connected) {
            let network_path: OwnedObjectPath = self.proxy.get_property("ConnectedNetwork").await?;
            let network = Network::new(self.proxy.connection().clone(), network_path).await?;
            return Ok(Some(network));
        }
        Ok(None)
    }

    pub async fn scan(&self) -> IWDResult<(), ScanError> {
        self.proxy.call_method("Scan", &()).await?;
        Ok(())
    }

    pub async fn disconnect(&self) -> IWDResult<(), DisconnectError> {
        self.proxy.call_method("Disconnect", &()).await?;
        Ok(())
    }

    pub async fn discovered_networks(&self) -> zbus::Result<Vec<(Network, i16)>> {
        let networks = self.proxy.call_method("GetOrderedNetworks", &()).await?;

        let body = networks.body();
        let objects: Vec<(OwnedObjectPath, i16)> = body.deserialize()?;

        let connection = self.proxy.connection().clone();
        let networks = stream::iter(objects)
            .then(|(path, signal_strength)| {
                let connection = connection.clone();
                async move {
                    let network = Network::new(connection, path.clone()).await?;
                    Ok::<_, zbus::Error>((network, signal_strength.to_owned()))
                }
            })
            .try_collect()
            .await?;

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
        agent: impl signal_level_agent::SignalLevelAgent,
    ) -> zbus::Result<SignalLevelAgentManager> {
        // Signal level boundaries should be sorted
        levels.sort_by_key(|signal_level| Reverse(*signal_level));

        let interface = signal_level_agent::SignalLevelInterface {
            agent,
            connection: self.proxy.connection().clone(),
            levels: levels.clone(),
        };

        let manager = SignalLevelAgentManager::register_agent(self.clone(), interface).await?;

        self.proxy
            .call_method("RegisterSignalLevelAgent", &(&manager.dbus_path, levels))
            .await?;
        Ok(manager)
    }
}

iwd_interface_impl!(StationDiagnostics, "net.connman.iwd.StationDiagnostic");

impl StationDiagnostics {
    pub async fn get(&self) -> zbus::Result<HashMap<String, String>> {
        let diagnostic = self.proxy.call_method("GetDiagnostics", &()).await?;

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

#[derive(Debug, Clone, Copy, PartialEq, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum State {
    Connected,
    Disconnected,
    Connecting,
    Disconnecting,
    Roaming,
}

impl TryFrom<OwnedValue> for State {
    type Error = zvariant::Error;

    fn try_from(value: OwnedValue) -> Result<Self, Self::Error> {
        let state_string: String = value.try_into()?;
        Self::from_str(&state_string).map_err(|_| zvariant::Error::IncorrectType)
    }
}
