//! Synchronous connections using [`interprocess`].
//! For listeners, the asynchronous [`async_tokio`](super::async_tokio) module is recommended.
//!
//! # Synchronous connection module
//!
//! This module provides a synchronous implementation of [`Listener`] and [`Connection`].
//! These are well-suited for clients that don't have a need to be asynchronous.
//!
//! ## Examples
//!
//! See the [sync example directory](https://github.com/tecc/gipc/tree/dev/examples/sync) for both an example client and listener.

use std::io::{Read, Write};
use interprocess::local_socket::{LocalSocketListener, LocalSocketStream};
use serde::{Deserialize, Serialize};
use crate::{Error, Result};
use super::interprocess::name_onto;
use crate::message::Message;

/// Listeners allow you to wait until new [`Connection`s](Connection) can be established.
pub struct Listener {
    internal: Box<dyn ListenerImpl>,
    closed: bool,
}

impl Listener {
    /// Creates a new listener based on a specified [`ListenerImpl`].
    /// Generally, you won't call this directly unless you're extending gipc.
    pub const fn new(internal: Box<dyn ListenerImpl>) -> Self {
        Self {
            internal,
            closed: false,
        }
    }

    /// Listens to a socket on the local machine with a name based on `name`.
    /// The actual name used is generated internally.
    pub fn listen_as_socket<'a, S>(name: S, global: bool) -> Result<Self> where S: AsRef<str> {
        let bound = name_onto!(LocalSocketListener::bind; name, global)?;
        Ok(Self::new(Box::new(bound)))
    }

    /// Accept a new connection.
    pub fn accept(&mut self) -> Result<Connection> {
        if self.closed {
            return Err(Error::Closed(false));
        }
        self.internal.accept()
    }
    /// Closes this listener, returning any error that occurred whilst closing it.
    /// After calling this function, all other methods will immediately return [`Error::Closed(false)`](Error::Closed) if called.
    pub fn close(&mut self) -> Result<()> {
        if self.closed {
            return Err(Error::Closed(false));
        }
        self.closed = true; // we set it to closed either way
        self.internal.close()
    }
}
impl Drop for Listener {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

/// Connections represent a two-way bidirectional stream that you can send and receive messages through.
pub struct Connection {
    internal: Box<dyn ConnectionImpl>,
    closed: bool,
}

impl Connection {
    /// Creates a new connection based on a specified [`ConnectionImpl`].
    /// Generally, you won't call this directly unless you're extending gipc.
    pub const fn new(internal: Box<dyn ConnectionImpl>) -> Self {
        Self {
            internal,
            closed: false,
        }
    }
    /// Connects to a socket using a name based on `name`.
    /// The actual name used is generated internally.
    pub fn connect_to_socket<S>(name: S, global: bool) -> Result<Self> where S: AsRef<str> {
        let bound = name_onto!(LocalSocketStream::connect; name, global)?;
        Ok(Self::new(Box::new(bound)))
    }

    fn _send<T>(&mut self, message: Message<T>) -> Result<()> where T: Serialize {
        message.write_to(&mut self.internal)
    }
    fn _receive<'de, T>(&mut self) -> Result<Message<T>> where T: Deserialize<'de> {
        Message::<T>::read_from(&mut self.internal)
    }

    /// Send a message through this connection.
    /// Will immediately fail with [`Error::Closed(false)`] if this connection is already closed.
    pub fn send<T>(&mut self, message_data: &T) -> Result<()> where T: Serialize {
        if self.closed {
            return Err(Error::Closed(false));
        }
        let message = Message::Data(message_data);
        self._send(message)
    }
    /// Receive a message from this connection.
    /// Will immediately fail with [`Error::Closed(false)`] if this connection is already closed,
    /// or fail with [`Error::Closed(true)`] if this connection was closed whilst trying to read the message.
    pub fn receive<'de, T>(&mut self) -> Result<T> where T: Deserialize<'de> {
        if self.closed {
            return Err(Error::Closed(false));
        }
        let message = self._receive()?;
        match message {
            Message::ClosingConnection => {
                self._close();
                Err(Error::Closed(true))
            }
            Message::Data(data) => Ok(data)
        }
    }
    /// Shorthand for calling [`send`](Self::send) and [`receive`](Self::receive) after one another.
    pub fn send_and_receive<'de, A, B>(&mut self, data: &A) -> Result<Message<B>> where A: Serialize, B: Deserialize<'de> {
        self.send(data)?;
        self.receive()
    }

    fn _close(&mut self) {
        self.internal.close();
        self.closed = true;
    }

    /// Closes this connection if it isn't already closed.
    /// This operation can never fail.
    pub fn close(&mut self) {
        if self.closed {
            return;
        }
        // ignore the results of this - it doesn't matter since we're closing it either way
        let _ = self._send::<()>(Message::ClosingConnection);
        self._close();
    }
}
impl Drop for Connection {
    fn drop(&mut self) {
        self.close();
    }
}

/// Internal implementation for a [`Listener`].
pub trait ListenerImpl {
    /// Accept a new connection.
    /// This function should _block_ until a connection can be established.
    fn accept(&mut self) -> Result<Connection>;

    /// Closes this listener implementation.
    /// After this function is called, no more functions will be called from the implementation.
    fn close(&mut self) -> Result<()>;
}

impl ListenerImpl for LocalSocketListener {
    fn accept(&mut self) -> Result<Connection> {
        Ok(Connection::from(LocalSocketListener::accept(self)?))
    }

    fn close(&mut self) -> Result<()> {
        // LocalSocketListener doesn't need to do anything when closing
        Ok(())
    }
}
impl From<LocalSocketListener> for Listener {
    fn from(value: LocalSocketListener) -> Self {
        Self::new(Box::new(value))
    }
}


/// Internal implementation for a [`Connection`].
pub trait ConnectionImpl: Read + Write {
    /// Closes this connection implementation.
    /// After this function is called, no more functions will be called from the implementation.
    fn close(&mut self);
}

impl ConnectionImpl for LocalSocketStream {
    fn close(&mut self) {
        let _ = self.flush();
    }
}
impl From<LocalSocketStream> for Connection {
    fn from(value: LocalSocketStream) -> Self {
        Connection::new(Box::new(value))
    }
}