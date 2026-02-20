use crate::error::monitor::MonitorError;
use futures_lite::StreamExt;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use zbus::{
    fdo::{InterfacesAddedStream, InterfacesRemovedStream, ManagedObjects, ObjectManagerProxy},
    names::OwnedInterfaceName,
};
use zvariant::{OwnedObjectPath, OwnedValue};

/// Sets up the D-Bus signal subscriptions and initial object snapshot for
/// monitoring iwd's managed objects.
///
/// Returns the shared object cache and an async future that keeps it up to
/// date by processing `InterfacesAdded` and `InterfacesRemoved` signals.
pub(crate) async fn setup_monitored_objects(
    object_manager: ObjectManagerProxy<'static>,
) -> zbus::Result<(
    Arc<RwLock<ManagedObjects>>,
    impl std::future::Future<Output = Result<(), MonitorError>> + Send,
)> {
    // Subscribe to signals *before* querying GetManagedObjects to avoid
    // a race where an object is added between the query and the subscription.
    let mut added_stream = object_manager.receive_interfaces_added().await?;
    let mut removed_stream = object_manager.receive_interfaces_removed().await?;

    let objects = object_manager.get_managed_objects().await?;
    let objects = Arc::new(RwLock::new(objects));
    let monitor_objects = Arc::clone(&objects);

    let monitor = async move {
        // Keep the object_manager alive for the lifetime of the monitor
        // so the signal streams remain valid.
        let _object_manager = object_manager;

        loop {
            futures_lite::future::race(
                handle_interfaces_added(&mut added_stream, &monitor_objects),
                handle_interfaces_removed(&mut removed_stream, &monitor_objects),
            )
            .await?;
        }
    };

    Ok((objects, monitor))
}

async fn handle_interfaces_added(
    stream: &mut InterfacesAddedStream,
    objects: &Arc<RwLock<ManagedObjects>>,
) -> Result<(), MonitorError> {
    let signal = stream
        .next()
        .await
        .ok_or(MonitorError::InterfacesAddedStreamEnded)?;
    let args = signal.args()?;
    let path = args.object_path.to_owned().into();
    let interfaces: HashMap<OwnedInterfaceName, HashMap<String, OwnedValue>> = args
        .interfaces_and_properties
        .iter()
        .map(|(iface, props)| {
            let owned_props: HashMap<String, OwnedValue> = props
                .iter()
                .filter_map(|(k, v)| Some((k.to_string(), v.try_to_owned().ok()?)))
                .collect();
            (iface.to_owned().into(), owned_props)
        })
        .collect();
    objects
        .write()
        .expect("monitor objects lock poisoned")
        .entry(path)
        .or_default()
        .extend(interfaces);
    Ok(())
}

async fn handle_interfaces_removed(
    stream: &mut InterfacesRemovedStream,
    objects: &Arc<RwLock<ManagedObjects>>,
) -> Result<(), MonitorError> {
    let signal = stream
        .next()
        .await
        .ok_or(MonitorError::InterfacesRemovedStreamEnded)?;
    let args = signal.args()?;
    let path: OwnedObjectPath = args.object_path.to_owned().into();
    let mut objects = objects.write().expect("monitor objects lock poisoned");
    if let Some(ifaces) = objects.get_mut(&path) {
        for iface_name in args.interfaces.iter() {
            ifaces.remove(iface_name.as_str());
        }
        if ifaces.is_empty() {
            objects.remove(&path);
        }
    }
    Ok(())
}
