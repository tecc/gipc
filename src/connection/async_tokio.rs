//! Asynchronous connections using [`tokio`](tokio).
//! Since the asynchronous code is fundamentally different from the synchronous code (see [`sync`](super::sync)),
//! the asynchronous code is in its own module.
//!
//! # Asynchronous connections
//!
//! This module provides an asynchronous implementation of [`Listener`] and [`Connection`], powered by [`tokio`].
//! It is specifically for Tokio as [`interprocess`] has built-in support for asynchronous connections using it.
//!
//! ## Examples
//!
//! See the [`async-tokio` example directory](https://github.com/tecc/gipc/tree/dev/examples/async-tokio) for both an example client and listener.

use super::interprocess::name_onto;
use crate::message::Message;
use crate::{Error, Result};
use async_trait::async_trait;
use futures_io::{AsyncRead, AsyncWrite};
use interprocess::local_socket::tokio::{LocalSocketListener, LocalSocketStream};
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::sync::Mutex;

/// Listeners allow you to wait until new [`Connection`s](Connection) can be established.
pub struct Listener {
    inner: Mutex<ListenerInner>
}
struct ListenerInner {
    internal: Box<dyn ListenerImpl>,
    closed: bool,
}

impl Listener {
    /// Creates a new listener based on a specified [`ListenerImpl`].
    /// Generally, you won't call this directly unless you're extending gipc.
    pub fn new(internal: Box<dyn ListenerImpl>) -> Self {
        Self {
            inner: Mutex::new(ListenerInner {
                internal,
                closed: false,
            })
        }
    }
    /// Listens to a socket on the local machine with a name based on `name`.
    /// The actual name used is generated internally.
    pub fn listen_as_socket<S>(name: S, global: bool) -> Result<Self>
    where
        S: AsRef<str>,
    {
        let bound = name_onto!(LocalSocketListener::bind; name, global)?;
        Ok(Self::new(Box::new(bound)))
    }

    /// Accept a new connection.
    pub async fn accept(&self) -> Result<Connection> {
        let mut lock = self.inner.lock().await;
        if lock.closed {
            return Err(Error::Closed(false));
        }
        lock.internal.accept().await
    }

    /// Closes this listener, returning any error that occurred whilst closing it.
    /// After calling this function, all other methods will immediately return [`Error::Closed(false)`](Error::Closed).
    pub async fn close(&self) -> Result<()> {
        let mut lock = self.inner.lock().await;
        if lock.closed {
            return Err(Error::Closed(false));
        }
        lock.closed = true; // we set it to closed either way
        lock.internal.close().await
    }

    /// Check if this listener is closed.
    pub fn is_closed(&self) -> bool {
        tokio::task::block_in_place(|| {
            self.inner.blocking_lock().closed
        })
    }
}

/// Connections represent a two-way bidirectional stream that you can send and receive messages through.
pub struct Connection {
    inner: Mutex<ConnectionInner>
}
struct ConnectionInner {
    internal: Box<dyn ConnectionImpl>,
    closed: bool,
}
impl ConnectionInner {
    async fn _send<T>(&mut self, message: Message<T>) -> Result<()>
        where
            T: Serialize,
    {
        message.write_to_async(&mut self.internal).await
    }
    async fn _receive<T>(&mut self) -> Result<Message<T>>
        where
            T: DeserializeOwned,
    {
        Message::<T>::read_from_async(&mut self.internal).await
    }

    async fn _close(&mut self) {
        self.internal.close().await;
        self.closed = true;
    }
}

impl Connection {
    /// Creates a new connection based on a specified [`ConnectionImpl`].
    /// Generally, you won't call this directly unless you're extending gipc.
    pub fn new(internal: Box<dyn ConnectionImpl>) -> Self {
        Self {
            inner: Mutex::new(ConnectionInner{
                internal,
                closed: false,
            })
        }
    }
    /// Connects to a socket using a name based on `name`.
    /// The actual name used is generated internally.
    pub async fn connect_to_socket<S>(name: S, global: bool) -> Result<Self>
    where
        S: AsRef<str>,
    {
        let bound = name_onto!(await LocalSocketStream::connect; name, global)?;
        Ok(Self::new(Box::new(bound)))
    }

    /// Send a message through this connection.
    /// Will immediately fail with [`Error::Closed(false)`](Error::Closed) if this connection is already closed.
    pub async fn send<T>(&self, message_data: T) -> Result<()>
    where
        T: Serialize,
    {
        let mut lock = self.inner.lock().await;
        if lock.closed {
            return Err(Error::Closed(false));
        }
        let message = Message::Data(message_data);
        lock._send(message).await
    }
    /// Receive a message from this connection.
    /// Will immediately fail with [`Error::Closed(false)`](Error::Closed) if this connection is already closed,
    /// or fail with [`Error::Closed(true)`](Error::Closed) if this connection was closed whilst trying to read the message.
    pub async fn receive<T>(&self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let mut lock = self.inner.lock().await;
        if lock.closed {
            return Err(Error::Closed(false));
        }
        let message = lock._receive().await?;
        match message {
            Message::ClosingConnection => {
                lock._close().await;
                Err(Error::Closed(true))
            }
            Message::Data(data) => Ok(data),
        }
    }

    /// Shorthand for calling [`send`] and [`receive`] after one another.
    pub async fn send_and_receive<A, B>(&self, data: &A) -> Result<B>
    where
        A: Serialize,
        B: DeserializeOwned,
    {
        self.send(data).await?;
        self.receive().await
    }

    /// Closes this connection if it isn't already closed.
    /// This operation can never fail.
    pub async fn close(&self) {
        let mut lock = self.inner.lock().await;
        if lock.closed {
            return;
        }
        // ignore the results of this - it doesn't matter since we're closing it either way
        let _ = lock._send::<()>(Message::ClosingConnection);
        lock._close().await;
    }

    /// Check if this connection is closed.
    pub fn is_closed(&self) -> bool {
        tokio::task::block_in_place(|| {
            self.inner.blocking_lock().closed
        })
    }
}

/// Listener implementation.
#[async_trait]
pub trait ListenerImpl: Send + Unpin {
    /// Accept a new connection.
    /// This function should return when a connection can be established.
    async fn accept(&mut self) -> Result<Connection>;
    /// Closes this listener implementation.
    /// After this function is called, no more functions will be called from the implementation.
    async fn close(&mut self) -> Result<()>;
}

#[async_trait]
impl ListenerImpl for LocalSocketListener {
    async fn accept(&mut self) -> Result<Connection> {
        Ok(Connection::from(LocalSocketListener::accept(self).await?))
    }
    async fn close(&mut self) -> Result<()> {
        Ok(())
    }
}

/// Internal implementation for a [`Connection`].
#[async_trait]
pub trait ConnectionImpl: AsyncRead + AsyncWrite + Send + Unpin {
    /// Closes this connection implementation.
    /// After this function is called, no more functions will be called from the implementation.
    async fn close(&mut self);
}

#[async_trait]
impl ConnectionImpl for LocalSocketStream {
    async fn close(&mut self) {
        // Once again, do nothing
    }
}

impl From<LocalSocketStream> for Connection {
    fn from(value: LocalSocketStream) -> Self {
        Connection::new(Box::new(value))
    }
}
