//! Error type definitions for errors that can occur during RPC interactions.
use std::result;

use futures;
use prost;

/// A convenience type alias for creating a `Result` with the error being of type `Error`.
pub type Result<A, E> = result::Result<A, Error<E>>;

/// An error has occurred.
#[derive(Clone, Debug, Eq, Fail, PartialEq)]
pub enum Error<E>
where
    E: Send,
{
    /// An error occurred during the execution of a (server) RPC endpoint or a (client) RPC transfer
    /// mechanism.
    #[fail(display = "Execution error: {}", error)]
    Execution {
        /// The underlying execution error.
        #[cause]
        error: E,
    },
    /// An error occurred during input decoding.
    #[fail(display = "Decode error: {}", error)]
    Decode {
        /// The underlying decode error.
        #[cause]
        error: prost::DecodeError,
    },
    /// An error occurred during output encoding.
    #[fail(display = "Encode error: {}", error)]
    Encode {
        /// The underlying encode error.
        #[cause]
        error: prost::EncodeError,
    },
    /// An async cancellation occurred.
    #[fail(display = "Canceled error: {}", error)]
    Canceled {
        /// The underlying canceled error.
        #[cause]
        error: futures::Canceled,
    },
}

impl<E> Error<E>
where
    E: Send,
{
    /// Constructs a new execution error.
    pub fn execution(error: E) -> Self {
        Error::Execution { error }
    }
}

impl<E> From<prost::DecodeError> for Error<E>
where
    E: Send,
{
    fn from(error: prost::DecodeError) -> Self {
        Error::Decode { error }
    }
}

impl<E> From<prost::EncodeError> for Error<E>
where
    E: Send,
{
    fn from(error: prost::EncodeError) -> Self {
        Error::Encode { error }
    }
}

impl<E> From<futures::Canceled> for Error<E>
where
    E: Send,
{
    fn from(error: futures::Canceled) -> Self {
        Error::Canceled { error }
    }
}
