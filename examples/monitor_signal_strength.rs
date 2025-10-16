use clap::Parser;
use iwdrs::station::signal_level_agent::SignalLevelAgent;

#[derive(Debug, Parser)]
/// Connect to a Wifi Network given a SSID and optionally a password.
struct Args {
    #[clap(allow_hyphen_values = true)]
    levels: Vec<i16>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let Args { levels } = Args::parse();

    let session = iwdrs::session::Session::new().await.unwrap();

    let station = session.stations().pop().unwrap();

    station
        .register_signal_level_agent(levels, Agent {})
        .await
        .unwrap();

    std::future::pending::<()>().await;
}

struct Agent {}

impl SignalLevelAgent for Agent {
    fn changed(
        &self,
        _station: &iwdrs::station::Station,
        signal_level: impl std::ops::RangeBounds<i16>,
    ) {
        println!(
            "Wifi Signal Strength Min {:?} Max {:?}",
            signal_level.start_bound(),
            signal_level.end_bound()
        )
    }
}
