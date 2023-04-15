#![warn(missing_docs)]
//! **General Interprocess Communication** (or *gipc*) is a library that abstracts away common things in
//! interprocess communication to speed up development and reduce errors.
//!
//! # General Interprocess Communication - gipc
//!
//! See the [`connection`] module for info on how to get started.
//!

pub mod message;
pub mod connection;
pub mod error;

pub use error::{Error, Result};