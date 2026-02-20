use crate::{
    access_point::{AccessPoint, AccessPointDiagnostics},
    adapter::Adapter,
    agent::{Agent, AgentManager},
    daemon::Daemon,
    device::Device,
    error::monitor::MonitorError,
    iwd_interface::{self, IwdInterface},
    known_network::KnownNetwork,
    monitor,
    station::{Station, StationDiagnostics},
};
use std::{
    future::Future,
    sync::{Arc, RwLock},
};
use uuid::Uuid;
use zbus::{
    Connection,
    fdo::{ManagedObjects, ObjectManagerProxy},
};
use zvariant::OwnedObjectPath;

#[derive(Debug)]
pub struct Session {
    connection: Connection,
    objects: Arc<RwLock<ManagedObjects>>,
}

impl Session {
    /// Creates a new Session by querying iwd's current D-Bus objects.
    ///
    /// The object cache is populated once at creation time and is not
    /// automatically updated. For a session that stays up to date, use
    /// [`Session::with_monitor`].
    pub async fn new() -> zbus::Result<Self> {
        let connection = Connection::system().await?;

        let object_manager = ObjectManagerProxy::new(&connection, "net.connman.iwd", "/").await?;

        let objects = object_manager.get_managed_objects().await?;

        Ok(Self {
            connection,
            objects: Arc::new(RwLock::new(objects)),
        })
    }

    /// Creates a new Session and returns an async future that keeps the
    /// internal object cache up to date by monitoring D-Bus
    /// [`InterfacesAdded`] and [`InterfacesRemoved`] signals.
    ///
    /// The caller must spawn the returned future in their async runtime.
    /// The future runs indefinitely until a D-Bus error occurs or the
    /// signal streams end.
    ///
    /// If the monitor future is not spawned, the session behaves
    /// identically to one created with [`Session::new`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use iwdrs::session::Session;
    ///
    /// let (session, monitor) = Session::with_monitor().await?;
    ///
    /// // Spawn with tokio:
    /// // tokio::spawn(monitor);
    ///
    /// // Or with async-std / smol:
    /// // async_std::task::spawn(monitor);
    ///
    /// // The session's object cache now stays up to date.
    /// let networks = session.known_networks().await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`InterfacesAdded`]: https://dbus.freedesktop.org/doc/dbus-specification.html#standard-interfaces-objectmanager
    /// [`InterfacesRemoved`]: https://dbus.freedesktop.org/doc/dbus-specification.html#standard-interfaces-objectmanager
    pub async fn with_monitor()
    -> zbus::Result<(Self, impl Future<Output = Result<(), MonitorError>> + Send)> {
        let connection = Connection::system().await?;

        let object_manager = ObjectManagerProxy::new(&connection, "net.connman.iwd", "/").await?;

        let (objects, monitor_future) = monitor::setup_monitored_objects(object_manager).await?;

        let session = Self {
            connection,
            objects,
        };

        Ok((session, monitor_future))
    }

    fn object_type(&self, interface_type: &'static str) -> Vec<OwnedObjectPath> {
        let objects = self.objects.read().expect("objects lock poisoned");
        objects
            .iter()
            .filter(|(_, interfaces)| interfaces.contains_key(interface_type))
            .map(|(path, _)| path.clone())
            .collect()
    }

    async fn collect_interface<Output: iwd_interface::IwdInterface>(
        &self,
    ) -> zbus::Result<Vec<Output>> {
        let paths = self.object_type(Output::INTERFACE);
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
