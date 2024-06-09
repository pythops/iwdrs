use anyhow::Result;
use std::sync::Arc;

use zbus::{Connection, Proxy};
use zvariant::OwnedObjectPath;

use crate::{device::Device, known_netowk::KnownNetwork};

#[derive(Clone, Debug)]
pub struct Network {
    pub(crate) connection: Arc<Connection>,
    pub(crate) dbus_path: OwnedObjectPath,
}

impl Network {
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
            "net.connman.iwd.Network",
        )
        .await
    }

    // Methods
    pub async fn connect(&self) -> Result<()> {
        let proxy = self.proxy().await?;
        proxy.call_method("Connect", &()).await?;
        Ok(())
    }

    // Properties

    pub async fn name(&self) -> Result<String> {
        let proxy = self.proxy().await?;
        let name: String = proxy.get_property("Name").await?;
        Ok(name)
    }

    pub async fn connected(&self) -> Result<bool> {
        let proxy = self.proxy().await?;
        let is_connected: bool = proxy.get_property("Connected").await?;
        Ok(is_connected)
    }

    pub async fn device(&self) -> Result<Device> {
        let proxy = self.proxy().await?;
        let device_path: OwnedObjectPath = proxy.get_property("Device").await?;

        let device = Device::new(self.connection.clone(), device_path);
        Ok(device)
    }

    pub async fn network_type(&self) -> Result<String> {
        let proxy = self.proxy().await?;
        let network_type: String = proxy.get_property("Type").await?;
        Ok(network_type)
    }

    pub async fn known_network(&self) -> Result<Option<KnownNetwork>> {
        let proxy = self.proxy().await?;
        if let Ok(known_network_path) = proxy.get_property::<OwnedObjectPath>("KnownNetwork").await
        {
            let network = KnownNetwork::new(self.connection.clone(), known_network_path);
            return Ok(Some(network));
        }
        Ok(None)
    }
}
