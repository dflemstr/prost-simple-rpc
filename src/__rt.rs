//! Utility functions used by generated code; this is *not* part of the crate's public API!
use bytes;
use failure;
use prost;

use error;

/// Efficiently decode a particular message type from a byte buffer.
///
/// Mostly used from generated code.
pub fn decode<B, M, E>(buf: B) -> error::Result<M, E>
where
    B: bytes::IntoBuf,
    M: prost::Message + Default,
    E: failure::Fail,
{
    let message = prost::Message::decode(buf)?;
    Ok(message)
}

/// Efficiently encode a particular message into a byte buffer.
///
/// Mostly used from generated code.
pub fn encode<M, E>(message: M) -> error::Result<bytes::Bytes, E>
where
    M: prost::Message,
    E: failure::Fail,
{
    let len = prost::Message::encoded_len(&message);
    let mut buf = ::bytes::BytesMut::with_capacity(len);
    prost::Message::encode(&message, &mut buf)?;
    Ok(buf.freeze())
}
