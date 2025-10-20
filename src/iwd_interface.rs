use zbus::{Connection, Proxy};
use zvariant::OwnedObjectPath;

const DESTINATION: &str = "net.connman.iwd";

pub trait IwdInterface: Sized {
    const INTERFACE: &str;

    async fn new(connection: Connection, dbus_path: OwnedObjectPath) -> zbus::Result<Self>;

    async fn proxy(
        connection: Connection,
        dbus_path: OwnedObjectPath,
    ) -> zbus::Result<Proxy<'static>> {
        Proxy::new_owned(connection, DESTINATION, dbus_path, Self::INTERFACE).await
    }
}

macro_rules! iwd_interface_impl {
    ($interface_ty:ident, $interface_name:expr) => {
        #[derive(Clone, Debug)]
        pub struct $interface_ty {
            proxy: Proxy<'static>,
        }

        impl crate::iwd_interface::IwdInterface for $interface_ty {
            const INTERFACE: &str = $interface_name;

            async fn new(connection: Connection, dbus_path: OwnedObjectPath) -> zbus::Result<Self> {
                Ok(Self {
                    proxy: Self::proxy(connection, dbus_path).await?,
                })
            }
        }
    };
}

pub(crate) use iwd_interface_impl;
