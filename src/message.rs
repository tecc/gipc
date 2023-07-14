//! Communication structures for the protocol. This is generally for internal use by gipc.
//! It is exposed as any change to the protocol is useful for the general consumer, as well as allowing for custom implementations of connections should that be required.

#[cfg(feature = "async-tokio")]
use futures_io::{AsyncRead, AsyncWrite};
use serde::{de::DeserializeOwned, Deserialize, Serialize}; // NOTE(tecc): Keeping Deserialize allows compatibility with older versions of Ciborium
use std::io::{Read, Write};

use crate::{Error, Result};

type Endian = byteorder::BigEndian;

/// Module for the raw reading and writing of messages.
/// This is the true core of how gipc works - any change to this module is dangerous.
/// It is also not exposed as this module does not have any relation to any users of gipc.
mod raw {
    use super::Endian;
    use crate::Result;
    #[cfg(feature = "async-tokio")]
    use futures_io::{AsyncRead, AsyncWrite};
    #[cfg(feature = "sync")]
    use std::io::{Read, Write};
    use std::mem::size_of;
    #[cfg(feature = "async-tokio")]
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    #[cfg(feature = "async-tokio")]
    use tokio_util::compat::Compat;

    fn serialised_vec(data: &Vec<u8>) -> Result<Vec<u8>> {
        use byteorder::WriteBytesExt;
        #[cfg(not(feature = "sync"))]
        use std::io::Write;
        let mut complete = Vec::new();
        complete.reserve_exact(data.len() + size_of::<u64>());
        WriteBytesExt::write_u64::<Endian>(&mut complete, data.len() as u64)?;
        Write::write_all(&mut complete, data.as_slice())?;
        Ok(complete)
    }

    #[cfg(feature = "sync")]
    /// Reads a message from `reader`.
    pub fn read_from<R>(reader: &mut R) -> Result<Vec<u8>>
    where
        R: Read,
    {
        use byteorder::ReadBytesExt;
        let size = reader.read_u64::<Endian>()? as usize;
        let mut vector = vec![0u8; size];
        reader.read_exact(vector.as_mut_slice())?;
        Ok(vector)
    }
    /// Writes bytes to `writer` asynchronously.
    #[cfg(feature = "sync")]
    pub fn write_to<W>(writer: &mut W, data: &Vec<u8>) -> Result<()>
    where
        W: Write,
    {
        let complete = serialised_vec(data)?;
        writer.write_all(complete.as_slice())?;
        Ok(())
    }

    /// Reads bytes from `reader` asynchronously.
    #[cfg(feature = "async-tokio")]
    pub async fn read_from_async<R>(reader: &mut Compat<R>) -> Result<Vec<u8>>
    where
        R: AsyncRead + Unpin,
    {
        let size = reader.read_u64().await? as usize;
        let mut vector = vec![0u8; size];
        reader.read_exact(vector.as_mut_slice()).await?;
        Ok(vector)
    }
    /// Write `data` to `writer` asynchronously.
    #[cfg(feature = "async-tokio")]
    pub async fn write_to_async<W>(writer: &mut Compat<W>, data: &Vec<u8>) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        let vec = serialised_vec(data)?;
        writer.write_all(vec.as_slice()).await?;
        Ok(())
    }
}

/// The core of gipc's protocol.
/// This is primarily for internal use.
#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub enum Message<T> {
    /// Indicates that the connection is about to be closed.
    ClosingConnection,
    /// Container for user-defined data.
    /// This is the main variant used for communication using [`Connection`s and `Listener`s](crate::connection).
    Data(T),
}

impl<T> Message<T> {
    /// Reads a [`Message`] from `reader`.
    #[cfg(feature = "sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "sync")))]
    pub fn read_from<R>(reader: &mut R) -> Result<Self>
    where
        T: DeserializeOwned,
        R: Read,
    {
        let raw = raw::read_from(reader)?;
        let deserialised: Self = ciborium::de::from_reader(raw.as_slice())
            .map_err(|v| Error::Deserialise(v.to_string()))?;

        Ok(deserialised)
    }
    /// Writes this [`Message`] to `writer`.
    #[cfg(feature = "sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "sync")))]
    pub fn write_to<W>(&self, writer: &mut W) -> Result<()>
    where
        T: Serialize,
        W: Write,
    {
        let mut serialised = Vec::new();
        ciborium::ser::into_writer(self, &mut serialised)
            .map_err(|v| Error::Serialise(v.to_string()))?;
        raw::write_to(writer, &serialised)?;
        writer.flush()?;
        Ok(())
    }

    /// Reads a [`Message`] from `reader` asynchronously.
    #[cfg(feature = "async-tokio")]
    #[cfg_attr(docsrs, doc(cfg(feature = "async-tokio")))]
    pub async fn read_from_async<R>(reader: R) -> Result<Self>
    where
        T: DeserializeOwned,
        R: AsyncRead + Unpin,
    {
        use tokio_util::compat::FuturesAsyncReadCompatExt;
        let mut reader = reader.compat();
        let raw = raw::read_from_async(&mut reader).await?;
        let deserialised: Self = ciborium::de::from_reader(raw.as_slice())
            .map_err(|v| Error::Deserialise(v.to_string()))?;

        Ok(deserialised)
    }

    /// Writes this [`Message`] to `writer` asynchronously.
    #[cfg(feature = "async-tokio")]
    #[cfg_attr(docsrs, doc(cfg(feature = "async-tokio")))]
    pub async fn write_to_async<W>(&self, writer: W) -> Result<()>
    where
        T: Serialize,
        W: AsyncWrite + Unpin + Send,
    {
        use tokio::io::AsyncWriteExt;
        use tokio_util::compat::FuturesAsyncWriteCompatExt;
        let mut serialised = Vec::new();
        let mut writer = writer.compat_write();
        ciborium::ser::into_writer(self, &mut serialised)
            .map_err(|v| Error::Serialise(v.to_string()))?;
        raw::write_to_async(&mut writer, &serialised).await?;
        writer.flush().await?;
        Ok(())
    }
}
