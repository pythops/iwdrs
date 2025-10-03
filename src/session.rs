use crate::{
    access_point::{AccessPoint, AccessPointDiagnostics},
    adapter::Adapter,
    agent::{Agent, AgentManager},
    device::Device,
    known_network::KnownNetwork,
    station::{Station, StationDiagnostics},
};
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;
use zbus::{Connection, Proxy};
use zvariant::{OwnedObjectPath, OwnedValue};

#[derive(Debug)]
pub struct Session {
    connection: Arc<Connection>,
    pub(crate) objects: HashMap<OwnedObjectPath, HashMap<String, HashMap<String, OwnedValue>>>,
}

impl Session {
    pub async fn new() -> zbus::Result<Self> {
        let connection = Arc::new(Connection::system().await?);

        let proxy = Proxy::new(
            &connection.clone(),
            "net.connman.iwd",
            "/",
            "org.freedesktop.DBus.ObjectManager",
        )
        .await?;

        let objects: HashMap<OwnedObjectPath, HashMap<String, HashMap<String, OwnedValue>>> =
            proxy.call("GetManagedObjects", &()).await?;

        Ok(Self {
            connection,
            objects,
        })
    }

    pub fn adapters(&self) -> Vec<Adapter> {
        self.objects
            .iter()
            .flat_map(|(path, interfaces)| {
                interfaces
                    .iter()
                    .filter(|(interface, _)| interface.as_str() == "net.connman.iwd.Adapter")
                    .map(|_| Adapter::new(self.connection.clone(), path.clone()))
            })
            .collect()
    }

    pub fn devices(&self) -> Vec<Device> {
        self.objects
            .iter()
            .flat_map(|(path, interfaces)| {
                interfaces
                    .iter()
                    .filter(|(interface, _)| interface.as_str() == "net.connman.iwd.Device")
                    .map(|_| Device::new(self.connection.clone(), path.clone()))
            })
            .collect()
    }

    pub fn stations(&self) -> Vec<Station> {
        self.objects
            .iter()
            .flat_map(|(path, interfaces)| {
                interfaces
                    .iter()
                    .filter(|(interface, _)| interface.as_str() == "net.connman.iwd.Station")
                    .map(|_| Station::new(self.connection.clone(), path.clone()))
            })
            .collect()
    }

    pub fn stations_diagnostics(&self) -> Vec<StationDiagnostics> {
        self.objects
            .iter()
            .flat_map(|(path, interfaces)| {
                interfaces
                    .iter()
                    .filter(|(interface, _)| {
                        interface.as_str() == "net.connman.iwd.StationDiagnostic"
                    })
                    .map(|_| StationDiagnostics::new(self.connection.clone(), path.clone()))
            })
            .collect()
    }

    pub fn access_points(&self) -> Vec<AccessPoint> {
        self.objects
            .iter()
            .flat_map(|(path, interfaces)| {
                interfaces
                    .iter()
                    .filter(|(interface, _)| interface.as_str() == "net.connman.iwd.AccessPoint")
                    .map(|_| AccessPoint::new(self.connection.clone(), path.clone()))
            })
            .collect()
    }

    pub fn access_points_diagnostics(&self) -> Vec<AccessPointDiagnostics> {
        self.objects
            .iter()
            .flat_map(|(path, interfaces)| {
                interfaces
                    .iter()
                    .filter(|(interface, _)| {
                        interface.as_str() == "net.connman.iwd.AccessPointDiagnostic"
                    })
                    .map(|_| AccessPointDiagnostics::new(self.connection.clone(), path.clone()))
            })
            .collect()
    }

    pub async fn register_agent(&self, agent: Agent) -> zbus::Result<AgentManager> {
        let path =
            OwnedObjectPath::try_from(format!("/iwdrs/agent/{}", Uuid::new_v4().as_simple()))?;
        let agent_manager = AgentManager::new(self.connection.clone(), path);
        agent_manager.register_agent(agent).await?;

        Ok(agent_manager)
    }

    pub async fn known_networks(&self) -> Vec<KnownNetwork> {
        self.objects
            .iter()
            .filter_map(|(path, interfaces)| {
                interfaces
                    .get("net.connman.iwd.KnownNetwork")
                    .map(|_net| KnownNetwork::new(self.connection.clone(), path.clone()))
            })
            .collect()
    }
}
