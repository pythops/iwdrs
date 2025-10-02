use std::sync::Arc;

use zvariant::OwnedObjectPath;

use zbus::{Connection, Proxy, Result};

use crate::{adapter::Adapter, modes::Mode};

#[derive(Debug, Clone)]
pub struct Device {
    pub(crate) connection: Arc<Connection>,
    pub(crate) dbus_path: OwnedObjectPath,
}

impl Device {
    pub(crate) fn new(connection: Arc<Connection>, dbus_path: OwnedObjectPath) -> Self {
        Self {
            connection,
            dbus_path,
        }
    }

    pub(crate) async fn proxy<'a>(&self) -> Result<zbus::Proxy<'a>> {
        Proxy::new(
            &self.connection,
            "net.connman.iwd",
            self.dbus_path.clone(),
            "net.connman.iwd.Device",
        )
        .await
    }

    pub async fn name(&self) -> Result<String> {
        let proxy = self.proxy().await?;
        let name: String = proxy.get_property("Name").await?;
        Ok(name)
    }

    pub async fn address(&self) -> Result<String> {
        let proxy = self.proxy().await?;
        let address: String = proxy.get_property("Address").await?;
        Ok(address)
    }

    pub async fn adapter(&self) -> Result<Adapter> {
        let proxy = self.proxy().await?;
        let adapter_path: OwnedObjectPath = proxy.get_property("Adapter").await?;
        let adapter = Adapter::new(self.connection.clone(), adapter_path);
        Ok(adapter)
    }

    pub async fn get_mode(&self) -> Result<Mode> {
        let proxy = self.proxy().await?;
        let mode: String = proxy.get_property("Mode").await?;

        match mode.as_str() {
            "station" => Ok(Mode::Station),
            "ap" => Ok(Mode::Ap),
            _ => unimplemented!(),
        }
    }

    pub async fn is_powered(&self) -> Result<bool> {
        let proxy = self.proxy().await?;
        let is_powered: bool = proxy.get_property("Powered").await?;
        Ok(is_powered)
    }

    pub async fn set_mode(&self, mode: Mode) -> Result<()> {
        let proxy = self.proxy().await?;
        proxy.set_property("Mode", mode.to_string()).await?;
        Ok(())
    }

    pub async fn set_power(&self, mode: bool) -> Result<()> {
        let proxy = self.proxy().await?;
        proxy.set_property("Powered", mode).await?;
        Ok(())
    }
}
