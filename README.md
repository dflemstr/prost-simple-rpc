# `prost-simple-rpc`

Do you want to use type-safe Protobuf-based RPC without having to use something heavy-weight like
gRPC?

This library lets you generate traits for implementing a generic RPC mechanism using Protobuf as
the schema language.  You have to supply your own underlying transport mechanism, for example
WebSockets, UNIX pipes, HTTP, etc.

## TODO

This library is quite complete but there are still a few things I would like to fix before a "1.0":

  - [ ] Use unboxed futures in generated client code.
  - [ ] Use unboxed futures in generated server code.
  - [ ] Try to support execution errors that don't implement `failure::Fail`.

## Usage

For the complete example, see the [example](./example) directory.

Start by defining a schema for your service in e.g. `src/schema/echo/service.proto`:

```proto
syntax = "proto3";

package echo;

// The Echo service. This service returns back the same data that it is given.
service Echo {
  // Echoes back the data sent, unmodified.
  rpc Echo (EchoRequest) returns (EchoResponse);
}

// The request for an `Echo.Echo` call.
message EchoRequest {
  // The data to be echoed back.
  bytes data = 1;
}

// The response for an `Echo.Echo` call.
message EchoResponse {
  // The echoed back data from `EchoRequest.data`.
  bytes data = 1;
}
```

Use `prost`, `prost-build` and `prost-simple-rpc-build` to generate Rust code for this service, by
putting this in your `build.rs`:

```rust
extern crate prost_build;
extern crate prost_simple_rpc_build;

fn main() {
    prost_build::Config::new()
        .service_generator(Box::new(prost_simple_rpc_build::ServiceGenerator))
        .compile_protos(
            &["src/schema/echo/service.proto"],
            &["src/schema"],
        )
        .unwrap();
}
```

Then, include the generated code in your Rust build, for example in `main.rs`.  There are a bunch of
extra crate dependencies for the generated code:

```rust
extern crate bytes;
extern crate failure;
extern crate futures;
extern crate prost;
#[macro_use]
extern crate prost_derive;
extern crate prost_simple_rpc;
extern crate tokio;

mod schema {
    pub mod echo {
        include!(concat!(env!("OUT_DIR"), "/echo.rs"));
    }
}

fn main() {
    // ...
}
```

### Client

Let's say you want to create a client for your service.  You need to implement a `Handler` that
handles the transport for your client calls.  Let's imagine you have some form of `WebSockets`
transport:

```rust
struct WebSocketTransport { /* ... */ }

impl prost_simple_rpc::handler::Handler for WebSocketTransport {
    // From our imaginary websocket library:
    type Error = websocket::Error;
    // This type is generated by prost-simple-rpc:
    type Descriptor = schema::echo::EchoDescriptor;
    // From our imaginary websocket library:
    type CallFuture = websocket::Future;

    /// Perform a raw call to the specified service and method.
    fn call(
        &mut self,
        method: <Self::Descriptor as descriptor::ServiceDescriptor>::Method,
        input: bytes::Bytes,
    ) -> Self::CallFuture {
        // You can use information from the descriptors to include in the request:
        self.websocket.call(Self::Descriptor::name(), method.name(), input)
    }
}
```

You can now use this handler with the client generated by `prost-simple-rpc`:

```rust
fn main() {
    let websocket = WebSocketTransport::connect("...");
    let client = schema::echo::EchoClient::new(websocket);
    let future = client.echo(schema::echo::EchoRequest { /* ... */ });
    // ... use the future to wait for a response.
}
```

### Server

To create a server for your service, start by implementing the generated service trait for the
service:

```rust
struct EchoService;

#[derive(Debug, Eq, Fail, PartialEq)]
#[fail(display = "Error!")]
struct Error;

impl schema::echo::Echo for EchoService {
    // You can supply an error type here if your service can fail.
    type Error = Error;
    // The future type used in the `echo()` method; you can of course use Box<Future<...>> here
    // but this library assumes unboxed futures by default.
    type EchoFuture = futures::future::FutureResult<schema::echo::EchoResponse, Self::Error>;

    fn echo(&self, input: schema::echo::EchoRequest) -> Self::EchoFuture {
        futures::future::ok(schema::echo::EchoResponse { data: input.data })
    }
}
```

You can now wrap this service with the generated server implementation to get something that can be
plugged into your preferred routing system:

```rust
fn main() {
    let server = schema::echo::EchoServer::new(EchoService);
    
    websocket::spawn_server(move |request| {
        // You would probably normally look up the right method descriptor via some kind of routing
        // information; here's a hard-coded example:
        let method = schema::echo::EchoMethodDescriptor::Echo;

        server.call(method, request.data);
    });
}
```
