use std::{collections::HashMap, sync::Arc};

use zvariant::{OwnedObjectPath, Value};

use zbus::{Connection, Proxy};

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
