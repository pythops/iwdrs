use std::{future::Future, str::FromStr};

use strum::EnumString;
use zbus::{Connection, Proxy, interface};
use zvariant::OwnedObjectPath;

use crate::{error::agent::Canceled, iwd_interface::IwdInterface, network::Network};

// AgentManager

#[derive(Debug, Clone)]
pub struct AgentManager {
    pub(crate) connection: Connection,
    pub(crate) dbus_path: OwnedObjectPath,
}

impl AgentManager {
    pub(crate) fn new(connection: Connection, dbus_path: OwnedObjectPath) -> Self {
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

    pub(crate) async fn register_agent(&self, agent: impl Agent) -> zbus::Result<()> {
        let proxy = self.proxy().await?;
        proxy
            .call_method("RegisterAgent", &(self.dbus_path))
            .await?;

        let interface = AgentInterface {
            agent,
            connection: self.connection.clone(),
        };

        self.connection
            .object_server()
            .at(self.dbus_path.clone(), interface)
            .await?;

        Ok(())
    }
}

#[derive(Debug, EnumString)]
pub enum CancellationReason {
    #[strum(serialize = "out-of-range")]
    OutOfRange,
    #[strum(serialize = "user-canceled")]
    UserCanceled,
    #[strum(serialize = "timed-out")]
    Timeout,
    #[strum(serialize = "shutdown")]
    Shutdown,
}

pub trait Agent: Send + Sync + 'static {
    /// This method gets called when the service daemon unregisters the agent. An agent can use it to do
    /// cleanup tasks. There is no need to unregister the agent, because when this method gets called it has
    /// already been unregistered.
    fn release(&self) {}

    ///This method gets called when trying to connect to a network and passphrase is required.
    fn request_passphrase(
        &self,
        network: &Network,
    ) -> impl Future<Output = Result<String, Canceled>> + Send;

    /// This method gets called when connecting to a network that requires authentication using a
    /// locally-stored encrypted private key file, to obtain that private key's encryption passphrase.
    fn request_private_key_passphrase(
        &self,
        network: &Network,
    ) -> impl Future<Output = Result<String, Canceled>> + Send;

    /// This method gets called when connecting to a network that requires authentication using a
    /// user name and password.
    fn request_user_name_and_passphrase(
        &self,
        network: &Network,
    ) -> impl Future<Output = Result<(String, String), Canceled>> + Send;

    /// This method gets called when connecting to a network that requires authentication with a
    /// user password.  The user name is optionally passed in the parameter.
    fn request_user_password(
        &self,
        network: &Network,
        user_name: Option<&String>,
    ) -> impl Future<Output = Result<String, Canceled>> + Send;

    /// This method gets called to indicate that the agent request failed before a reply was returned.
    fn cancel(&self, _reason: CancellationReason) {}
}

struct AgentInterface<A> {
    agent: A,
    connection: Connection,
}

#[interface(name = "net.connman.iwd.Agent")]
impl<A: Agent> AgentInterface<A> {
    #[zbus(name = "Release")]
    fn release(&self) {
        self.agent.release();
    }

    #[zbus(name = "RequestPassphrase")]
    async fn request_passphrase(&self, network_path: OwnedObjectPath) -> zbus::fdo::Result<String> {
        let network = Network::new(self.connection.clone(), network_path).await?;

        Ok(self.agent.request_passphrase(&network).await?)
    }

    #[zbus(name = "RequestPrivateKeyPassphrase")]
    async fn request_private_key_passphrase(
        &self,
        network_path: OwnedObjectPath,
    ) -> zbus::fdo::Result<String> {
        let network = Network::new(self.connection.clone(), network_path).await?;
        Ok(self.agent.request_private_key_passphrase(&network).await?)
    }

    #[zbus(name = "RequestUserNameAndPassword")]
    async fn request_user_name_and_passphrase(
        &self,
        network_path: OwnedObjectPath,
    ) -> zbus::fdo::Result<(String, String)> {
        let network = Network::new(self.connection.clone(), network_path).await?;
        Ok(self
            .agent
            .request_user_name_and_passphrase(&network)
            .await?)
    }

    #[zbus(name = "RequestUserPassword")]
    async fn request_user_password(
        &self,
        network_path: OwnedObjectPath,
        user_name: zvariant::Optional<String>,
    ) -> zbus::fdo::Result<String> {
        let network = Network::new(self.connection.clone(), network_path).await?;
        let user_name = user_name.as_ref();
        Ok(self
            .agent
            .request_user_password(&network, user_name)
            .await?)
    }

    #[zbus(name = "Cancel")]
    fn cancel(&self, reason: String) {
        let reason = CancellationReason::from_str(&reason).unwrap();
        self.agent.cancel(reason);
    }
}
