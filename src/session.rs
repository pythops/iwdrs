use crate::{
    access_point::{AccessPoint, AccessPointDiagnostics},
    adapter::Adapter,
    agent::{Agent, AgentManager},
    daemon::Daemon,
    device::Device,
    iwd_interface::{self, IwdInterface},
    known_network::KnownNetwork,
    station::{Station, StationDiagnostics},
};
use std::collections::HashMap;
use uuid::Uuid;
use zbus::{Connection, Proxy};
use zvariant::{OwnedObjectPath, OwnedValue};

#[derive(Debug)]
pub struct Session {
    connection: Connection,
    pub(crate) objects: HashMap<OwnedObjectPath, HashMap<String, HashMap<String, OwnedValue>>>,
}

impl Session {
    pub async fn new() -> zbus::Result<Self> {
        let connection = Connection::system().await?;

        let proxy = Proxy::new(
            &connection,
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

    fn object_type(
        &self,
        interface_type: &'static str,
    ) -> impl IntoIterator<Item = OwnedObjectPath> {
        self.objects.iter().flat_map(move |(path, interfaces)| {
            let path = path.clone();
            interfaces
                .iter()
                .filter(move |(interface, _)| interface.as_str() == interface_type)
                .map(move |_| path.clone())
        })
    }

    async fn collect_interface<Output: iwd_interface::IwdInterface>(
        &self,
    ) -> zbus::Result<Vec<Output>> {
        let paths: Vec<_> = self.object_type(Output::INTERFACE).into_iter().collect();
        let mut results = Vec::with_capacity(paths.len());
        for path in paths {
            results.push(Output::new(self.connection.clone(), path).await?);
        }
        Ok(results)
    }

    pub async fn adapters(&self) -> zbus::Result<Vec<Adapter>> {
        self.collect_interface().await
    }

    pub async fn daemon(&self) -> zbus::Result<Daemon> {
        let path = OwnedObjectPath::try_from("/net/connman/iwd")?;
        Daemon::new(self.connection.clone(), path).await
    }

    pub async fn devices(&self) -> zbus::Result<Vec<Device>> {
        self.collect_interface().await
    }

    pub async fn stations(&self) -> zbus::Result<Vec<Station>> {
        self.collect_interface().await
    }

    pub async fn stations_diagnostics(&self) -> zbus::Result<Vec<StationDiagnostics>> {
        self.collect_interface().await
    }

    pub async fn access_points(&self) -> zbus::Result<Vec<AccessPoint>> {
        self.collect_interface().await
    }

    pub async fn access_points_diagnostics(&self) -> zbus::Result<Vec<AccessPointDiagnostics>> {
        self.collect_interface().await
    }

    pub async fn register_agent(&self, agent: impl Agent) -> zbus::Result<AgentManager> {
        let path =
            OwnedObjectPath::try_from(format!("/iwdrs/agent/{}", Uuid::new_v4().as_simple()))?;
        let agent_manager = AgentManager::new(self.connection.clone(), path);
        agent_manager.register_agent(agent).await?;

        Ok(agent_manager)
    }

    pub async fn known_networks(&self) -> zbus::Result<Vec<KnownNetwork>> {
        self.collect_interface().await
    }
}
