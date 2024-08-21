//#[cfg(feature = "data-server")]
//mod data_events;
//#[cfg(feature = "data-server")]
//mod data_service;
//#[cfg(feature = "data-server")]
//mod handle;
//#[cfg(not(feature = "data-server"))]
//mod noop;
//#[cfg(feature = "data-server")]
//mod request_handlers;
//#[cfg(feature = "data-server")]
//mod server;

//#[cfg(feature = "data-server")]
//pub use handle::DataServerHandle;
//#[cfg(not(feature = "data-server"))]
//pub use noop::init_data_server;
//#[cfg(not(feature = "data-server"))]
//pub use noop::DataServerHandle;
//#[cfg(feature = "data-server")]
//pub use server::init_data_server;

#[cfg(not(feature = "data-server"))]
pub mod noop;
#[cfg(not(feature = "data-server"))]
pub use noop::noop::init_data_server;
#[cfg(not(feature = "data-server"))]
pub use noop::noop::DataServerHandle;
#[cfg(feature = "data-server")]
pub mod server;
#[cfg(feature = "data-server")]
pub use server::handle::DataServerHandle;
#[cfg(feature = "data-server")]
pub use server::server::init_data_server;
