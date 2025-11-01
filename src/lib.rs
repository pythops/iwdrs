use futures_lite::{Stream, StreamExt, stream};
use zbus::Proxy;
use zvariant::OwnedValue;

pub mod access_point;
pub mod adapter;
pub mod agent;
pub mod daemon;
pub mod device;
pub mod error;
mod iwd_interface;
pub mod known_network;
pub mod modes;
pub mod network;
pub mod session;
pub mod station;

async fn property_stream<T: TryFrom<OwnedValue, Error = zvariant::Error> + Unpin>(
    proxy: Proxy<'static>,
    initial_value: zbus::Result<T>,
    property_name: &'static str,
) -> zbus::Result<impl Stream<Item = zbus::Result<T>> + Unpin> {
    // `receive_property_changed` does not yield the initial value if another stream on the same property has already
    // been created and yielded a value. Therefore we manually add an initial value to the beginning of the stream
    // as most consumers will expect it to be there. Note that the first stream created for each property (per proxy)
    // will initially yield two elements, but I think that is a worthwhile tradeoff.
    Ok(Box::pin(
        stream::iter([initial_value]).chain(
            proxy
                .receive_property_changed(property_name)
                .await
                .then(|property_changed| async move { property_changed.get().await }),
        ),
    ))
}
