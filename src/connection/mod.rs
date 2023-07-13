//! Connection and listener implementations.
//!
//! ## Connections
//! Connections represent bidirectional streams that gipc can communicate through.
//!
//! ## Listeners
//! Listeners allow programs to accept [connections](#connections) from other programs.

pub(crate) mod interprocess;

#[cfg(feature = "sync")]
pub mod sync;
#[cfg(feature = "sync")]
pub use sync::{Connection, Listener};

#[cfg(feature = "async-tokio")]
pub mod async_tokio;
#[cfg(feature = "async-tokio")]
pub use async_tokio::{Connection as AsyncConnection, Listener as AsyncListener};
