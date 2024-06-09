use anyhow::Result;
use std::sync::Arc;

use zvariant::OwnedObjectPath;

use zbus::{Connection, Proxy};

use crate::netowrk::Network;

const INTERFACE: &str = "net.connman.iwd.Station";

#[derive(Debug, Clone)]
pub struct Station {
    pub(crate) connection: Arc<Connection>,
    pub(crate) dbus_path: OwnedObjectPath,
}

pub enum StationState {
    Connected,
    Disconnected,
    Connectimg,
    Disconnecting,
    Roaming,
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
            INTERFACE,
        )
        .await
    }

    pub async fn is_scanning(&self) -> Result<bool> {
        let proxy = self.proxy().await?;
        let is_scanning: bool = proxy.get_property("Scanning").await?;
        Ok(is_scanning)
    }

    pub async fn state(&self) -> Result<String> {
        let proxy = self.proxy().await?;
        let state: String = proxy.get_property("State").await?;
        Ok(state)
    }

    pub async fn connected_network(&self) -> Result<Option<Network>> {
        let state = self.state().await?;
        if state == "connected" {
            let proxy = self.proxy().await?;
            let network_path: OwnedObjectPath = proxy.get_property("ConnectedNetwork").await?;
            let network = Network::new(self.connection.clone(), network_path);
            return Ok(Some(network));
        }
        Ok(None)
    }

    pub async fn scan(&self) -> Result<()> {
        let proxy = self.proxy().await?;
        proxy.call("Scan", &()).await?;
        Ok(())
    }

    pub async fn disconnect(&self) -> Result<()> {
        let proxy = self.proxy().await?;
        proxy.call("Disconnect", &()).await?;
        Ok(())
    }

    pub async fn discovered_networks(&self) -> Result<Vec<(Network, i16)>> {
        let proxy = self.proxy().await?;
        let netowrks = proxy.call_method("GetOrderedNetworks", &()).await?;

        let body = netowrks.body();
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
}
