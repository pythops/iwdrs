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
use zbus::{
    Connection, fdo::ObjectManagerProxy, names::OwnedInterfaceName, proxy::CacheProperties,
};
use zvariant::{OwnedObjectPath, OwnedValue};

type OwnedPropertiesMap = HashMap<String, OwnedValue>;
type OwnedInterfaceMap = HashMap<OwnedInterfaceName, OwnedPropertiesMap>;

#[derive(Debug)]
pub struct Session {
    connection: Connection,
    object_manager: ObjectManagerProxy<'static>,
}

impl Session {
    pub async fn new() -> zbus::Result<Self> {
        let connection = Connection::system().await?;

        let object_manager = ObjectManagerProxy::builder(&connection)
            .destination("net.connman.iwd")?
            .path("/")?
            .cache_properties(CacheProperties::No)
            .build()
            .await?;

        Ok(Self {
            connection,
            object_manager,
        })
    }

    async fn managed_objects(&self) -> zbus::Result<HashMap<OwnedObjectPath, OwnedInterfaceMap>> {
        Ok(self.object_manager.get_managed_objects().await?)
    }

    async fn collect_interface<Output: iwd_interface::IwdInterface>(
        &self,
    ) -> zbus::Result<Vec<Output>> {
        let objects = self.managed_objects().await?;

        let paths = objects
            .into_iter()
            .filter(|(_, interfaces)| {
                interfaces
                    .keys()
                    .any(|interface| interface.as_str() == Output::INTERFACE)
            })
            .map(|(path, _)| path);

        let mut results = Vec::new();
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
