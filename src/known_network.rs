use zbus::{Connection, Proxy, Result};
use zvariant::OwnedObjectPath;

use crate::{iwd_interface::iwd_interface_impl, network::NetworkType};

iwd_interface_impl!(KnownNetwork, "net.connman.iwd.AccessPoint");

impl KnownNetwork {
    pub async fn forget(&self) -> Result<()> {
        self.proxy.call_method("Forget", &()).await?;
        Ok(())
    }

    pub async fn name(&self) -> Result<String> {
        self.proxy.get_property("Name").await
    }

    pub async fn network_type(&self) -> Result<NetworkType> {
        self.proxy.get_property("Type").await
    }

    pub async fn hidden(&self) -> Result<bool> {
        let hidden: bool = self.proxy.get_property("Hidden").await?;
        Ok(hidden)
    }

    pub async fn last_connected_time(&self) -> Result<String> {
        self.proxy.get_property("LastConnectedTime").await
    }

    pub async fn set_autoconnect(&self, auto_connect: bool) -> Result<()> {
        self.proxy.set_property("AutoConnect", auto_connect).await?;
        Ok(())
    }

    pub async fn get_autoconnect(&self) -> Result<bool> {
        let auto_connect: bool = self.proxy.get_property("AutoConnect").await?;
        Ok(auto_connect)
    }
}
