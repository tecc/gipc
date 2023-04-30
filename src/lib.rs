#![warn(missing_docs)]
//! **General Interprocess Communication** (or *gipc*) is a library that abstracts away common things in
//! interprocess communication to speed up development and reduce errors.
//!
//! See the [`connection`] module for info on how to get started.
//!
//! ## Crate structure
//!
//! The [`connection`] module handles all things related to receiving and sending data between your programs.
//! This is aided by the [`message`] module, which describes the communication protocol that it uses.
//!
//! Any errors the crate can return are in the [`error`] module.

pub mod message;
pub mod connection;
pub mod error;

pub use error::{Error, Result};