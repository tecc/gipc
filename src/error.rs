//! A small module containing the [`Error`] and [`Result`] type.

/// Error type for this library. Any error this library produces uses this to represent it.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Indicates that there was some I/O error.
    #[error("io: {0}")]
    Io(
        #[doc = "The I/O error that occurred."]
        #[from]
        #[source]
        std::io::Error,
    ),
    /// Indicates that there was a serialisation error.
    #[error("serialisation: {0}")]
    Serialise(#[doc = "The message of the internal error."] String),
    /// Indicates that there was a deserialisation error.
    #[error("deserialisation: {0}")]
    Deserialise(#[doc = "The message of the internal error."] String),
    /// Indicates that tokio could not join a task.
    #[cfg(feature = "async-tokio")]
    #[error("tokio join failed: {0}")]
    TokioJoin(
        #[doc = "The join error that occurred."]
        #[from]
        tokio::task::JoinError,
    ),
    /// Indicates that something is closed.
    #[error("{}", if *.0 { "was closed by operation" } else { "already closed" })]
    Closed(
        #[doc = "Whether it was closed by the operation (`true`) or was already closed (`false`)"]
        bool,
    ),
}
/// Result type for this library. Shorthand for [`std::result::Result<T, Error>`].
pub type Result<T> = std::result::Result<T, Error>;
