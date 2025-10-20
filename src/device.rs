use zbus::{Connection, Proxy, Result};
use zvariant::OwnedObjectPath;

use crate::{
    adapter::Adapter,
    iwd_interface::{IwdInterface, iwd_interface_impl},
    modes::Mode,
};

iwd_interface_impl!(Device, "net.connman.iwd.Device");

impl Device {
    pub async fn name(&self) -> Result<String> {
        self.proxy.get_property("Name").await
    }

    pub async fn address(&self) -> Result<String> {
        self.proxy.get_property("Address").await
    }

    pub async fn adapter(&self) -> Result<Adapter> {
        let adapter_path: OwnedObjectPath = self.proxy.get_property("Adapter").await?;
        let adapter = Adapter::new(self.proxy.connection().clone(), adapter_path).await?;
        Ok(adapter)
    }

    pub async fn get_mode(&self) -> Result<Mode> {
        let mode: String = self.proxy.get_property("Mode").await?;

        match mode.as_str() {
            "station" => Ok(Mode::Station),
            "ap" => Ok(Mode::Ap),
            _ => unimplemented!(),
        }
    }

    pub async fn is_powered(&self) -> Result<bool> {
        let is_powered: bool = self.proxy.get_property("Powered").await?;
        Ok(is_powered)
    }

    pub async fn set_mode(&self, mode: Mode) -> Result<()> {
        self.proxy.set_property("Mode", mode.to_string()).await?;
        Ok(())
    }

    pub async fn set_power(&self, mode: bool) -> Result<()> {
        self.proxy.set_property("Powered", mode).await?;
        Ok(())
    }
}
