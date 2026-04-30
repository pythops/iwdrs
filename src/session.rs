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
use futures_lite::{FutureExt, StreamExt};
use std::{collections::HashMap, future};
use uuid::Uuid;
use zbus::{
    Connection, Proxy,
    fdo::{
        InterfacesAddedArgs, InterfacesAddedStream, InterfacesRemovedStream, ObjectManagerProxy,
    },
    names::OwnedInterfaceName,
};
use zvariant::{OwnedObjectPath, OwnedValue};

type OwnedPropertiesMap = HashMap<String, OwnedValue>;
type OwnedInterfaceMap = HashMap<OwnedInterfaceName, OwnedPropertiesMap>;

#[derive(Debug)]
pub struct Session {
    connection: Connection,
    _object_manager: ObjectManagerProxy<'static>,
    cache: async_lock::Mutex<ObjectCache>,
}

impl Session {
    pub async fn new() -> zbus::Result<Self> {
        let connection = Connection::system().await?;

        let proxy = Proxy::new_owned(
            connection.clone(),
            "net.connman.iwd",
            "/",
            "org.freedesktop.DBus.ObjectManager",
        )
        .await?;
        let object_manager = ObjectManagerProxy::from(proxy);

        let objects: HashMap<OwnedObjectPath, OwnedInterfaceMap> =
            object_manager.get_managed_objects().await?;
        let interfaces_added = object_manager.receive_interfaces_added().await?;
        let interfaces_removed = object_manager.receive_interfaces_removed().await?;

        Ok(Self {
            connection,
            _object_manager: object_manager,
            cache: async_lock::Mutex::new(ObjectCache {
                interfaces_added,
                interfaces_removed,
                objects,
            }),
        })
    }

    async fn collect_interface<Output: iwd_interface::IwdInterface>(
        &self,
    ) -> zbus::Result<Vec<Output>> {
        let mut cache = self.cache.lock().await;
        cache.update_objects_cache().await?;

        let paths: Vec<_> = cache.object_type(Output::INTERFACE).into_iter().collect();
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

#[derive(Debug)]
struct ObjectCache {
    interfaces_added: InterfacesAddedStream,
    interfaces_removed: InterfacesRemovedStream,
    objects: HashMap<OwnedObjectPath, OwnedInterfaceMap>,
}

impl ObjectCache {
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

    async fn update_objects_cache(&mut self) -> zbus::Result<()> {
        while let Some(removal) = self.interfaces_removed.next().or(future::ready(None)).await {
            let args = removal.args()?;
            self.objects.remove(args.object_path());
        }

        while let Some(added) = self.interfaces_added.next().or(future::ready(None)).await {
            let args = added.args()?;
            let InterfacesAddedArgs {
                object_path,
                interfaces_and_properties,
                ..
            } = args;
            let object_path: OwnedObjectPath = object_path.into_owned().into();
            let interfaces_and_properties: OwnedInterfaceMap = interfaces_and_properties
                .into_iter()
                .map(|(iface, props)| {
                    let iface = iface.into_owned().into();
                    let props: OwnedPropertiesMap = props
                        .into_iter()
                        .filter_map(|(key, val)| {
                            let Ok(val) = val.try_into_owned() else {
                                return None;
                            };
                            Some((key.to_string(), val))
                        })
                        .collect();
                    (iface, props)
                })
                .collect();
            self.objects.insert(object_path, interfaces_and_properties);
        }

        Ok(())
    }
}
