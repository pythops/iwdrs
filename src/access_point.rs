use anyhow::Result;
use std::{collections::HashMap, sync::Arc};

use zvariant::{OwnedObjectPath, Value};

use zbus::{Connection, Proxy};

#[derive(Debug, Clone)]
pub struct AccessPoint {
    pub(crate) connection: Arc<Connection>,
    pub(crate) dbus_path: OwnedObjectPath,
}

impl AccessPoint {
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
            "net.connman.iwd.AccessPoint",
        )
        .await
    }

    // Methods
    pub async fn start(&self, ssid: &str, psk: &str) -> Result<()> {
        let proxy = self.proxy().await?;
        proxy.call_method("Start", &(ssid, psk)).await?;
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let proxy = self.proxy().await?;
        proxy.call_method("Stop", &()).await?;
        Ok(())
    }

    pub async fn start_profile(&self, ssid: &str) -> Result<()> {
        let proxy = self.proxy().await?;
        proxy.call_method("StartProfile", &(ssid)).await?;
        Ok(())
    }

    pub async fn scan(&self) -> Result<()> {
        let proxy = self.proxy().await?;
        proxy.call_method("Scan", &()).await?;
        Ok(())
    }

    pub async fn networks(&self) -> Result<Vec<HashMap<String, String>>> {
        let proxy = self.proxy().await?;
        let networks = proxy.call_method("GetOrderedNetworks", &()).await?;
        let body = networks.body();
        let body: Vec<HashMap<String, Value>> = body.deserialize()?;
        let body = body
            .into_iter()
            .map(|map| {
                map.into_iter()
                    .map(|(k, v)| (k, v.to_string()))
                    .collect::<HashMap<String, String>>()
            })
            .collect::<Vec<HashMap<String, String>>>();

        Ok(body)
    }

    // Proprieties
    pub async fn has_started(&self) -> Result<bool> {
        let proxy = self.proxy().await?;
        let has_started: bool = proxy.get_property("Started").await?;
        Ok(has_started)
    }

    pub async fn frequency(&self) -> Result<Option<u32>> {
        let proxy = self.proxy().await?;
        Ok(proxy.get_property("Frequency").await.ok())
    }

    pub async fn is_scanning(&self) -> Result<bool> {
        let proxy = self.proxy().await?;
        let is_scanning: bool = proxy.get_property("Scanning").await?;
        Ok(is_scanning)
    }

    pub async fn name(&self) -> Result<Option<String>> {
        let proxy = self.proxy().await?;
        Ok(proxy.get_property("Name").await.ok())
    }

    pub async fn pairwise_ciphers(&self) -> Result<Option<Vec<String>>> {
        let proxy = self.proxy().await?;
        Ok(proxy.get_property("PairwiseCiphers").await.ok())
    }

    pub async fn group_cipher(&self) -> Result<Option<String>> {
        let proxy = self.proxy().await?;
        Ok(proxy.get_property("GroupCipher").await.ok())
    }
}

#[derive(Debug, Clone)]
pub struct AccessPointDiagnostic {
    pub(crate) connection: Arc<Connection>,
    pub(crate) dbus_path: OwnedObjectPath,
}

impl AccessPointDiagnostic {
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
            "net.connman.iwd.AccessPointDiagnostic",
        )
        .await
    }

    pub async fn get(&self) -> Result<Vec<HashMap<String, String>>> {
        let proxy = self.proxy().await?;
        let diagnostic = proxy.call_method("GetDiagnostics", &()).await?;

        let body = diagnostic.body();
        let body: Vec<HashMap<String, Value>> = body.deserialize()?;
        let body = body
            .into_iter()
            .map(|map| {
                map.into_iter()
                    .map(|(k, v)| (k, v.to_string()))
                    .collect::<HashMap<String, String>>()
            })
            .collect::<Vec<HashMap<String, String>>>();

        Ok(body)
    }
}
