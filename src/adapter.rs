use zbus::{Connection, Proxy, Result};
use zvariant::OwnedObjectPath;

use crate::iwd_interface::iwd_interface_impl;

iwd_interface_impl!(Adapter, "net.connman.iwd.Adapter");

impl Adapter {
    pub async fn name(&self) -> Result<String> {
        self.proxy.get_property("Name").await
    }

    pub async fn model(&self) -> Result<String> {
        self.proxy.get_property("Model").await
    }

    pub async fn vendor(&self) -> Result<String> {
        self.proxy.get_property("Vendor").await
    }

    pub async fn supported_modes(&self) -> Result<Vec<String>> {
        self.proxy.get_property("SupportedModes").await
    }

    pub async fn is_powered(&self) -> Result<bool> {
        let is_powered: bool = self.proxy.get_property("Powered").await?;
        Ok(is_powered)
    }

    pub async fn set_power(&self, mode: bool) -> Result<()> {
        self.proxy.set_property("Powered", mode).await?;
        Ok(())
    }
}
