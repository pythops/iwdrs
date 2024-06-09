use anyhow::Result;
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

use zbus::{Connection, Proxy};
use zvariant::{OwnedObjectPath, OwnedValue};

use crate::{
    agent::{Agent, AgentManager},
    device::Device,
    known_netowk::KnownNetwork,
    station::Station,
};

#[derive(Debug)]
pub struct Session {
    connection: Arc<Connection>,
    pub(crate) objects: HashMap<OwnedObjectPath, HashMap<String, HashMap<String, OwnedValue>>>,
}

impl Session {
    pub async fn new() -> Result<Self> {
        let connection = Arc::new(Connection::system().await?);

        let proxy = Proxy::new(
            &connection.clone(),
            "net.connman.iwd",
            "/",
            "org.freedesktop.DBus.ObjectManager",
        )
        .await
        .unwrap();

        let objects: HashMap<OwnedObjectPath, HashMap<String, HashMap<String, OwnedValue>>> =
            proxy.call("GetManagedObjects", &()).await.unwrap();

        Ok(Self {
            connection,
            objects,
        })
    }

    pub fn device(&self) -> Option<Device> {
        let device: Option<Device> = self
            .objects
            .iter()
            .flat_map(|(path, interfaces)| {
                interfaces
                    .iter()
                    .filter(|(interface, _)| interface.as_str() == "net.connman.iwd.Device")
                    .map(|_| Device::new(self.connection.clone(), path.clone()))
            })
            .next();

        device
    }

    pub fn station(&self) -> Option<Station> {
        let station: Option<Station> = self
            .objects
            .iter()
            .flat_map(|(path, interfaces)| {
                interfaces
                    .iter()
                    .filter(|(interface, _)| interface.as_str() == "net.connman.iwd.Station")
                    .map(|_| Station::new(self.connection.clone(), path.clone()))
            })
            .next();
        station
    }

    pub async fn register_agent(&self, agent: Agent) -> Result<AgentManager> {
        let path =
            OwnedObjectPath::try_from(format!("/iwdrs/agent/{}", Uuid::new_v4().as_simple()))?;
        let agent_manager = AgentManager::new(self.connection.clone(), path);
        agent_manager.register_agent(agent).await?;

        Ok(agent_manager)
    }

    pub async fn known_networks(&self) -> Vec<KnownNetwork> {
        let known_networks: Vec<KnownNetwork> = self
            .objects
            .iter()
            .filter_map(|(path, interfaces)| {
                interfaces
                    .get("net.connman.iwd.KnownNetwork")
                    .map(|_net| KnownNetwork::new(self.connection.clone(), path.clone()))
            })
            .collect();

        known_networks
    }
}
