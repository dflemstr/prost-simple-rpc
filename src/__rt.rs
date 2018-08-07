//! Utility functions used by generated code; this is *not* part of the crate's public API!
use std::marker;
use std::mem;

use bytes;
use failure;
use futures;
use prost;

use descriptor;
use error;
use handler;

/// A future returned by a client call.
#[derive(Debug)]
pub enum ClientFuture<H, I, O>
where
    H: handler::Handler,
{
    /// The message has not yet been encoded.
    Encode(
        I,
        H,
        <H::Descriptor as descriptor::ServiceDescriptor>::Method,
    ),
    /// The message was sent over RPC but the call future is not yet done.
    Call(H::CallFuture),
    /// We have returned the response to the caller.
    Done(marker::PhantomData<O>),
}

impl<H, I, O> ClientFuture<H, I, O>
where
    H: handler::Handler,
    I: prost::Message,
    O: prost::Message + Default,
{
    pub fn new(
        handler: H,
        input: I,
        method: <H::Descriptor as descriptor::ServiceDescriptor>::Method,
    ) -> Self {
        ClientFuture::Encode(input, handler, method)
    }
}

impl<H, I, O> futures::Future for ClientFuture<H, I, O>
where
    H: handler::Handler,
    I: prost::Message,
    O: prost::Message + Default,
{
    type Item = O;
    type Error = error::Error<H::Error>;

    fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
        loop {
            match mem::replace(self, ClientFuture::Done(marker::PhantomData)) {
                ClientFuture::Encode(input, handler, method) => {
                    let input_bytes = encode(input)?;
                    *self = ClientFuture::Call(handler.call(method, input_bytes));
                }
                ClientFuture::Call(mut future) => match future.poll() {
                    Ok(futures::Async::Ready(bytes)) => {
                        return Ok(futures::Async::Ready(decode::<O, _>(bytes)?));
                    }
                    Ok(futures::Async::NotReady) => {
                        *self = ClientFuture::Call(future);
                        return Ok(futures::Async::NotReady);
                    }
                    Err(err) => return Err(error::Error::execution(err)),
                },
                ClientFuture::Done(_) => panic!("cannot poll a client future twice"),
            }
        }
    }
}

/// Efficiently decode a particular message type from a byte buffer.
pub fn decode<M, E>(buf: bytes::Bytes) -> error::Result<M, E>
where
    M: prost::Message + Default,
    E: failure::Fail,
{
    let message = prost::Message::decode(buf)?;
    Ok(message)
}

/// Efficiently encode a particular message into a byte buffer.
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
