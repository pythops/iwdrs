use std::sync::Arc;

use zbus::{Connection, Proxy, Result};
use zvariant::OwnedObjectPath;

#[derive(Clone, Debug)]
pub struct Adapter {
    pub(crate) connection: Arc<Connection>,
    pub(crate) dbus_path: OwnedObjectPath,
}

impl Adapter {
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
            "net.connman.iwd.Adapter",
        )
        .await
    }

    pub async fn name(&self) -> Result<String> {
        let proxy = self.proxy().await?;
        let name: String = proxy.get_property("Name").await?;
        Ok(name)
    }

    pub async fn model(&self) -> Result<String> {
        let proxy = self.proxy().await?;
        let model: String = proxy.get_property("Model").await?;
        Ok(model)
    }

    pub async fn vendor(&self) -> Result<String> {
        let proxy = self.proxy().await?;
        let vendor: String = proxy.get_property("Vendor").await?;
        Ok(vendor)
    }

    pub async fn supported_modes(&self) -> Result<Vec<String>> {
        let proxy = self.proxy().await?;
        let modes: Vec<String> = proxy.get_property("SupportedModes").await?;
        Ok(modes)
    }

    pub async fn is_powered(&self) -> Result<bool> {
        let proxy = self.proxy().await?;
        let is_powered: bool = proxy.get_property("Powered").await?;
        Ok(is_powered)
    }

    pub async fn set_power(&self, mode: bool) -> Result<()> {
        let proxy = self.proxy().await?;
        proxy.set_property("Powered", mode).await?;
        Ok(())
    }
}
