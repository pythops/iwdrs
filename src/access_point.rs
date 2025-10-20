use std::collections::HashMap;
use zvariant::{OwnedObjectPath, Value};

use zbus::{Connection, Proxy};

use crate::{
    error::{
        Result as IWDResult,
        access_point::{AccessPointStartError, AccessPointStopError, ScanError, StartProfileError},
    },
    iwd_interface::iwd_interface_impl,
};

iwd_interface_impl!(AccessPoint, "net.connman.iwd.AccessPoint");

impl AccessPoint {
    // Methods
    pub async fn start(&self, ssid: &str, psk: &str) -> IWDResult<(), AccessPointStartError> {
        self.proxy.call_method("Start", &(ssid, psk)).await?;
        Ok(())
    }

    pub async fn stop(&self) -> IWDResult<(), AccessPointStopError> {
        self.proxy.call_method("Stop", &()).await?;
        Ok(())
    }

    pub async fn start_profile(&self, ssid: &str) -> IWDResult<(), StartProfileError> {
        self.proxy.call_method("StartProfile", &(ssid)).await?;
        Ok(())
    }

    pub async fn scan(&self) -> IWDResult<(), ScanError> {
        self.proxy.call_method("Scan", &()).await?;
        Ok(())
    }

    pub async fn networks(&self) -> zbus::Result<Vec<HashMap<String, String>>> {
        let networks = self.proxy.call_method("GetOrderedNetworks", &()).await?;
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
    pub async fn has_started(&self) -> zbus::Result<bool> {
        let has_started: bool = self.proxy.get_property("Started").await?;
        Ok(has_started)
    }

    pub async fn frequency(&self) -> zbus::Result<Option<u32>> {
        Ok(self.proxy.get_property("Frequency").await.ok())
    }

    pub async fn is_scanning(&self) -> zbus::Result<bool> {
        let is_scanning: bool = self.proxy.get_property("Scanning").await?;
        Ok(is_scanning)
    }

    pub async fn name(&self) -> zbus::Result<Option<String>> {
        Ok(self.proxy.get_property("Name").await.ok())
    }

    pub async fn pairwise_ciphers(&self) -> zbus::Result<Option<Vec<String>>> {
        Ok(self.proxy.get_property("PairwiseCiphers").await.ok())
    }

    pub async fn group_cipher(&self) -> zbus::Result<Option<String>> {
        Ok(self.proxy.get_property("GroupCipher").await.ok())
    }
}

iwd_interface_impl!(
    AccessPointDiagnostics,
    "net.connman.iwd.AccessPointDiagnostic"
);

impl AccessPointDiagnostics {
    pub async fn get(&self) -> zbus::Result<Vec<HashMap<String, String>>> {
        let diagnostic = self.proxy.call_method("GetDiagnostics", &()).await?;

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
