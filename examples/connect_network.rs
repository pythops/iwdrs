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

    let agent = Agent {
        request_passphrase_fn: Box::new(move || {
            let password = password.clone();
            Box::pin(async {
                match password {
                    Some(password) => Ok(password.clone()),
                    None => Err("No Password Provided".into()),
                }
            })
        }),
    };
    let _agent_manager = session.register_agent(agent).await.unwrap();

    let station = session.stations().pop().unwrap();
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
