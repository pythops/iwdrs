use std::time::Duration;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let session = iwdrs::session::Session::new().await.unwrap();

    let station_diagnostics = session.stations_diagnostics().await.unwrap().pop().unwrap();

    let mut ticker = tokio::time::interval(Duration::from_secs(1));
    loop {
        println!("{:?}", station_diagnostics.get().await);
        ticker.tick().await;
    }
}
