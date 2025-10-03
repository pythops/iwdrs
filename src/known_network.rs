use std::sync::Arc;

use zbus::{Connection, Proxy, Result};
use zvariant::OwnedObjectPath;

use crate::network::NetworkType;

#[derive(Clone, Debug)]
pub struct KnownNetwork {
    pub(crate) connection: Arc<Connection>,
    pub(crate) dbus_path: OwnedObjectPath,
}

impl KnownNetwork {
    pub fn new(connection: Arc<Connection>, dbus_path: OwnedObjectPath) -> Self {
        Self {
            connection,
            dbus_path,
        }
    }

    async fn proxy<'a>(&self) -> Result<zbus::Proxy<'a>> {
        Proxy::new(
            &self.connection,
            "net.connman.iwd",
            self.dbus_path.clone(),
            "net.connman.iwd.KnownNetwork",
        )
        .await
    }

    pub async fn forget(&self) -> Result<()> {
        let proxy = self.proxy().await?;
        proxy.call_method("Forget", &()).await?;
        Ok(())
    }

    pub async fn name(&self) -> Result<String> {
        let proxy = self.proxy().await?;
        let name: String = proxy.get_property("Name").await?;
        Ok(name)
    }

    pub async fn network_type(&self) -> Result<NetworkType> {
        let proxy = self.proxy().await?;
        proxy.get_property("Type").await
    }

    pub async fn hidden(&self) -> Result<bool> {
        let proxy = self.proxy().await?;
        let hidden: bool = proxy.get_property("Hidden").await?;
        Ok(hidden)
    }

    pub async fn last_connected_time(&self) -> Result<String> {
        let proxy = self.proxy().await?;
        let last_time: String = proxy.get_property("LastConnectedTime").await?;
        Ok(last_time)
    }

    pub async fn set_autoconnect(&self, auto_connect: bool) -> Result<()> {
        let proxy = self.proxy().await?;
        proxy.set_property("AutoConnect", auto_connect).await?;
        Ok(())
    }

    pub async fn get_autoconnect(&self) -> Result<bool> {
        let proxy = self.proxy().await?;
        let auto_connect: bool = proxy.get_property("AutoConnect").await?;
        Ok(auto_connect)
    }
}
