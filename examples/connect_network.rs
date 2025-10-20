use std::time::Duration;

use clap::Parser;
use iwdrs::{agent::Agent, network::Network};

#[derive(Debug, Parser)]
/// Connect to a Wifi Network given a SSID and optionally a password.
struct Args {
    ssid: String,
    password: Option<String>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let Args { ssid, password } = Args::parse();

    let session = iwdrs::session::Session::new().await.unwrap();

    let agent = PasswdAgent(password);
    let _agent_manager = session.register_agent(agent).await.unwrap();

    let station = session.stations().await.unwrap().pop().unwrap();
    let mut networks = station.discovered_networks().await.unwrap();

    let network = match find_network(&ssid, &networks).await {
        Some(network) => network,
        None => {
            station.scan().await.unwrap();
            while station.is_scanning().await.unwrap() {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            networks = station.discovered_networks().await.unwrap();
            find_network(&ssid, &networks)
                .await
                .expect("Could not find network")
        }
    };

    network.connect().await.unwrap()
}

async fn find_network<'a>(ssid: &str, networks: &'a [(Network, i16)]) -> Option<&'a Network> {
    for (network, _signal_strength) in networks {
        if network.name().await.unwrap() == ssid {
            return Some(network);
        }
    }
    None
}

struct PasswdAgent(Option<String>);

impl Agent for PasswdAgent {
    fn request_passphrase(
        &self,
        _network: &Network,
    ) -> impl Future<Output = Result<String, iwdrs::error::agent::Canceled>> + Send {
        std::future::ready(match self.0.as_ref() {
            Some(passwd) => Ok(passwd.clone()),
            None => Err(iwdrs::error::agent::Canceled {}),
        })
    }

    fn request_private_key_passphrase(
        &self,
        _network: &Network,
    ) -> impl Future<Output = Result<String, iwdrs::error::agent::Canceled>> + Send {
        std::future::ready(Err(iwdrs::error::agent::Canceled {}))
    }

    fn request_user_name_and_passphrase(
        &self,
        _network: &Network,
    ) -> impl Future<Output = Result<(String, String), iwdrs::error::agent::Canceled>> + Send {
        std::future::ready(Err(iwdrs::error::agent::Canceled {}))
    }

    fn request_user_password(
        &self,
        _network: &Network,
        _user_name: Option<&String>,
    ) -> impl Future<Output = Result<(String, String), iwdrs::error::agent::Canceled>> + Send {
        std::future::ready(Err(iwdrs::error::agent::Canceled {}))
    }
}
