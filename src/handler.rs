//! Traits for defining generic RPC handlers.
use bytes;
use futures;

use descriptor;

/// An implementation of a specific RPC handler.
///
/// This can be an actual implementation of a service, or something that will send a request over
/// a network to fulfill a request.
pub trait Handler: Clone + Send + 'static {
    /// The type of errors that this handler might generate, beyond the default RPC error type.
    type Error: Send;
    /// The service descriptor for the service whose requests this handler can handle.
    type Descriptor: descriptor::ServiceDescriptor;
    /// The future that results from a call to the `call` method of this trait.
    type CallFuture: futures::Future<Item = bytes::Bytes, Error = Self::Error> + Send;

    /// Perform a raw call to the specified service and method.
    fn call(
        &mut self,
        method: <Self::Descriptor as descriptor::ServiceDescriptor>::Method,
        input: bytes::Bytes,
    ) -> Self::CallFuture;
}
