use std::str::FromStr;

use futures_lite::Stream;
use strum::EnumString;
use zbus::{Connection, Proxy, Result as ZbusResult};
use zvariant::{OwnedObjectPath, OwnedValue};

use crate::{
    device::Device,
    error::{IWDError, network::ConnectError},
    iwd_interface::{IwdInterface, iwd_interface_impl},
    known_network::KnownNetwork,
};

iwd_interface_impl!(Network, "net.connman.iwd.Network");

impl Network {
    // Methods
    pub async fn connect(&self) -> Result<(), IWDError<ConnectError>> {
        self.proxy.call_method("Connect", &()).await?;
        Ok(())
    }

    // Properties

    pub async fn name(&self) -> ZbusResult<String> {
        let name: String = self.proxy.get_property("Name").await?;
        Ok(name)
    }

    pub async fn connected(&self) -> ZbusResult<bool> {
        self.proxy.get_property("Connected").await
    }

    pub async fn connected_stream(
        &self,
    ) -> zbus::Result<impl Stream<Item = zbus::Result<bool>> + Unpin + 'static> {
        crate::property_stream(self.proxy.clone(), self.connected().await, "Connected").await
    }

    pub async fn device(&self) -> ZbusResult<Device> {
        let device_path: OwnedObjectPath = self.proxy.get_property("Device").await?;

        Device::new(self.proxy.connection().clone(), device_path).await
    }

    pub async fn network_type(&self) -> ZbusResult<NetworkType> {
        self.proxy.get_property("Type").await
    }

    pub async fn known_network(&self) -> ZbusResult<Option<KnownNetwork>> {
        if let Ok(known_network_path) = self
            .proxy
            .get_property::<OwnedObjectPath>("KnownNetwork")
            .await
        {
            let network =
                KnownNetwork::new(self.proxy.connection().clone(), known_network_path).await?;
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
