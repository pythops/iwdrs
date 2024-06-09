use anyhow::Result;
use std::{future::Future, pin::Pin, sync::Arc};

use zbus::{interface, Connection, Proxy};
use zvariant::OwnedObjectPath;

// AgentManager

#[derive(Debug, Clone)]
pub struct AgentManager {
    pub(crate) connection: Arc<Connection>,
    pub(crate) dbus_path: OwnedObjectPath,
}

impl AgentManager {
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
            "/net/connman/iwd",
            "net.connman.iwd.AgentManager",
        )
        .await
    }

    pub(crate) async fn register_agent(&self, agent: Agent) -> Result<()> {
        let proxy = self.proxy().await?;
        proxy
            .call_method("RegisterAgent", &(self.dbus_path))
            .await?;

        self.connection
            .object_server()
            .at(self.dbus_path.clone(), agent)
            .await?;

        Ok(())
    }
}

// Agent

pub type RequestPassPhraseFn = Box<
    dyn (Fn() -> Pin<Box<dyn Future<Output = Result<String, Box<dyn std::error::Error>>> + Send>>)
        + Send
        + Sync,
>;

pub struct Agent {
    pub request_passphrase_fn: RequestPassPhraseFn,
}

#[interface(name = "net.connman.iwd.Agent")]
impl Agent {
    #[zbus(name = "RequestPassphrase")]
    async fn request_passphrase(
        &self,
        _netowrk_path: OwnedObjectPath,
    ) -> zbus::fdo::Result<String> {
        match (self.request_passphrase_fn)().await {
            Ok(passphrase) => Ok(passphrase),
            Err(e) => Err(zbus::fdo::Error::Failed(e.to_string())),
        }
    }
}
