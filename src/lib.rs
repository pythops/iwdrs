use futures_lite::{Stream, StreamExt};
use zbus::Proxy;
use zvariant::OwnedValue;

pub mod session;

pub mod station;

pub mod device;

pub mod adapter;

pub mod agent;

pub mod network;

pub mod known_network;

pub mod access_point;

pub mod modes;

pub mod error;

mod iwd_interface;

async fn property_stream<T: TryFrom<OwnedValue, Error = zvariant::Error> + Unpin>(
    proxy: Proxy<'static>,
    property_name: &'static str,
) -> zbus::Result<impl Stream<Item = zbus::Result<T>> + Unpin> {
    Ok(Box::pin(
        proxy
            .receive_property_changed(property_name)
            .await
            .then(|property_changed| async move { property_changed.get().await }),
    ))
}
