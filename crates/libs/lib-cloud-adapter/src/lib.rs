// region:  --- Modules

use std::future::Future;

use cloud_adapter_core::{
    CloudAdapterTrait, ConnectionError, ConnectionLost, TokenConnection, TokenDelivery,
};
#[cfg(feature = "dev")]
use connector_dev::Dev;
use data_source_core::MsgBusData;
#[cfg(feature = "special-hivemq")]
use special_hivemq::SpecialHiveMQ;
#[cfg(feature = "special-iothub")]
use special_iothub::{DeliverContext, SpecialIoTHub};
use tokio::sync::watch;

// endregion

// region:  --- Types

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    /// * Adapter chosen
    AdapterTypeNotFound(String),
    Initialization(String),
}

// TODO -- a macro could be written to apply to this enum.  All the code below this enum could then be auto generated
// just like enum_dispatch. except the trait in a different library is still an issue
// This will eventually be helpful because to support multiple different adapter types, an enum needs to be generated
// for each token type as well
//enum PublishReturnTypes {
//Dev(cloud_adapter_core::TokenDelivery)
//HiveMQ(special_hivemq::DeliveryToken),
//IoTHub(special_iothub::DeliverContext),
//}
enum CloudAdapter {
    #[cfg(feature = "dev")]
    DevCloud(connector_dev::Dev),
    #[cfg(feature = "special-hivemq")]
    HiveMQ(special_hivemq::SpecialHiveMQ),
    #[cfg(feature = "special-iothub")]
    IoTHub(special_iothub::SpecialIoTHub),
}

// endregion

// region:  --- Public Functions

pub fn new(adapter_type: &str, credentials: &str) -> Result<impl CloudAdapterTrait> {
    match adapter_type {
        #[cfg(feature = "dev")]
        "dev" => Ok(CloudAdapter::from(Dev::new())),
        #[cfg(feature = "special-hivemq")]
        "special-hivemq" => Ok(CloudAdapter::from(CloudAdapter::HiveMQ(
            SpecialHiveMQ::new(credentials)
                .map_err(|err| Error::Initialization(err.to_string()))?,
        ))),
        #[cfg(feature = "special-iothub")]
        "special-iothub" => Ok(CloudAdapter::from(SpecialIoTHub::new())),
        _ => Err(Error::AdapterTypeNotFound(adapter_type.to_string())),
    }
}

// endregion

// region: --- Boilerplate

// This is the manual way of being able to use a static dispatch in this way and the compiler will bring this down to a 0 cost abstraction.
// It could be annoying to maintain, the `enum_dispatch` crate handles this generation for us. However I can't figure out the
// cross library use of it, yet. It may be best to write my own
impl CloudAdapterTrait for CloudAdapter {
    fn publish(&mut self, msg: MsgBusData) -> impl TokenDelivery + 'static {
        match self {
            #[cfg(feature = "dev")]
            CloudAdapter::DevCloud(inner) => inner.publish(msg),
            #[cfg(feature = "special-hivemq")]
            CloudAdapter::HiveMQ(inner) => inner.publish(msg),
            #[cfg(feature = "special-iothub")]
            CloudAdapter::IoTHub(inner) => inner.publish(msg),
        }
    }

    async fn connect(
        &mut self,
    ) -> std::result::Result<
        TokenConnection<
            impl Future<
                    Output = std::result::Result<watch::Receiver<ConnectionLost>, ConnectionError>,
                > + 'static,
        >,
        ConnectionError,
    > {
        match self {
            #[cfg(feature = "dev")]
            CloudAdapter::DevCloud(inner) => inner.connect().await,
            #[cfg(feature = "special-hivemq")]
            CloudAdapter::HiveMQ(inner) => inner.connect().await,
            #[cfg(feature = "special-iothub")]
            CloudAdapter::IoTHub(inner) => inner.connect(),
        }
    }

    fn disconnect(
        &mut self,
    ) -> std::result::Result<
        impl Future<Output = std::result::Result<(), ConnectionError>> + 'static,
        ConnectionError,
    > {
        match self {
            #[cfg(feature = "dev")]
            CloudAdapter::DevCloud(inner) => inner.disconnect(),
            #[cfg(feature = "special-hivemq")]
            CloudAdapter::HiveMQ(inner) => inner.disconnect(),
            #[cfg(feature = "special-iothub")]
            CloudAdapter::IoTHub(inner) => inner.disconnect(),
        }
    }
}

#[cfg(feature = "dev")]
impl From<Dev> for CloudAdapter {
    fn from(value: Dev) -> Self {
        CloudAdapter::DevCloud(value)
    }
}

#[cfg(feature = "special-hivemq")]
impl From<SpecialHiveMQ> for CloudAdapter {
    fn from(value: SpecialHiveMQ) -> Self {
        CloudAdapter::HiveMQ(value)
    }
}

#[cfg(feature = "special-iothub")]
impl From<SpecialHiveMQ> for CloudAdapter {
    fn from(value: SpecialIoTHub) -> Self {
        CloudAdapter::IoTHub(value)
    }
}

// endregion

// This is the idea of enum_dispatch
// Any cloud adapter needs to be added to this enum
// The impl of the enum can be completely negated in this case
//#[enum_dispatch(CloudAdapter)]
//enum CloudAdapterManifest {
//SpecialHiveMQ,
//SpecialIoTHub,
//}

//#[enum_dispatch]
//pub trait CloudAdapter {
//fn publish(&self) -> impl Future;
//}
