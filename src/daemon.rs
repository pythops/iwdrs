use std::{collections::HashMap, path::PathBuf};

use crate::iwd_interface::iwd_interface_impl;
use zvariant::{OwnedObjectPath, Value};

use zbus::{Connection, Proxy};

iwd_interface_impl!(Daemon, "net.connman.iwd.Daemon");

impl Daemon {
    pub async fn get_info(&self) -> zbus::Result<DaemonInfo> {
        let info = self.proxy.call_method("GetInfo", &()).await?;
        let info = info.body();
        let info: HashMap<String, Value> = info.deserialize()?;
        DaemonInfo::from_zbus_map(info)
    }
}

#[derive(Debug, Clone)]
pub struct DaemonInfo {
    pub state_dir: PathBuf,
    pub version: String,
    pub network_config_enabled: bool,
}

impl DaemonInfo {
    pub(crate) fn from_zbus_map(body: HashMap<String, Value>) -> zbus::Result<Self> {
        let state_dir: zvariant::Str = body.get("StateDirectory").unwrap().try_into()?;
        let state_dir: PathBuf = PathBuf::from(state_dir.as_str());

        let version: zvariant::Str = body.get("Version").unwrap().try_into()?;
        let version = version.to_string();

        Ok(Self {
            state_dir,
            version,
            network_config_enabled: body
                .get("NetworkConfigurationEnabled")
                .unwrap()
                .try_into()?,
        })
    }
}
