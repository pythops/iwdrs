use std::{str::FromStr, sync::Arc};

use futures_lite::{Stream, StreamExt};
use strum::EnumString;
use zbus::{Connection, Proxy, Result as ZbusResult};
use zvariant::{OwnedObjectPath, OwnedValue};

use crate::{
    device::Device,
    error::{IWDError, network::ConnectError},
    known_network::KnownNetwork,
};

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
    pub async fn connect(&self) -> Result<(), IWDError<ConnectError>> {
        let proxy = self.proxy().await?;
        proxy.call_method("Connect", &()).await?;
        Ok(())
    }

    // Properties

    pub async fn name(&self) -> ZbusResult<String> {
        let proxy = self.proxy().await?;
        let name: String = proxy.get_property("Name").await?;
        Ok(name)
    }

    pub async fn connected(&self) -> ZbusResult<bool> {
        self.connected_stream()
            .await?
            .next()
            .await
            .ok_or_else(|| zbus::Error::Unsupported)?
    }

    pub async fn connected_stream(
        &self,
    ) -> zbus::Result<impl Stream<Item = zbus::Result<bool>> + Unpin> {
        let proxy = self.proxy().await?;
        crate::property_stream(proxy, "Connected").await
    }

    pub async fn device(&self) -> ZbusResult<Device> {
        let proxy = self.proxy().await?;
        let device_path: OwnedObjectPath = proxy.get_property("Device").await?;

        let device = Device::new(self.connection.clone(), device_path);
        Ok(device)
    }

    pub async fn network_type(&self) -> ZbusResult<NetworkType> {
        let proxy = self.proxy().await?;
        proxy.get_property("Type").await
    }

    pub async fn known_network(&self) -> ZbusResult<Option<KnownNetwork>> {
        let proxy = self.proxy().await?;
        if let Ok(known_network_path) = proxy.get_property::<OwnedObjectPath>("KnownNetwork").await
        {
            let network = KnownNetwork::new(self.connection.clone(), known_network_path);
            return Ok(Some(network));
        }
        Ok(None)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum NetworkType {
    Open,
    Wep,
    Psk,
    #[strum(serialize = "8021x")]
    Eap,
}

impl TryFrom<OwnedValue> for NetworkType {
    type Error = zvariant::Error;

    fn try_from(value: OwnedValue) -> Result<Self, Self::Error> {
        let state_string: String = value.try_into()?;
        Self::from_str(&state_string).map_err(|_| zvariant::Error::IncorrectType)
    }
}
