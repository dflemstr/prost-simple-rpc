extern crate bytes;
extern crate futures;
extern crate prost;
#[macro_use]
extern crate prost_derive;
extern crate prost_simple_rpc;
extern crate tokio;

mod schema;

fn main() {
    run_echo_roundtrip();
    run_greeting_roundtrip();
}

fn run_echo_roundtrip() {
    use schema::echo::Echo;
    use futures::Future;

    let server = schema::echo::EchoServer::new(EchoService { fail: false });
    let client = schema::echo::EchoClient::new(server);
    let data = vec![1, 2, 3];
    let future = client
        .echo(schema::echo::EchoRequest { data })
        .map(|r| {
            eprintln!("Response: {:?}", r);
        })
        .map_err(|e| {
            eprintln!("Error: {:?}", e);
        });
    tokio::run(future)
}

fn run_greeting_roundtrip() {
    use schema::greeting::Greeting;
    use futures::Future;

    let server = schema::greeting::GreetingServer::new(GreetingService {
        fail_hello: false,
        fail_goodbye: false,
    });
    let client = schema::greeting::GreetingClient::new(server);
    let name = "dflemstr".to_owned();

    let future = client
        .say_hello(schema::greeting::SayHelloRequest { name })
        .map(|r| {
            eprintln!("Response: {:?}", r);
        })
        .map_err(|e| {
            eprintln!("Error: {:?}", e);
        });
    tokio::run(future)
}

#[derive(Clone, Debug)]
struct EchoService {
    fail: bool,
}

impl schema::echo::Echo for EchoService {
    type Error = &'static str;
    type EchoFuture = futures::future::FutureResult<schema::echo::EchoResponse, Self::Error>;

    fn echo(&self, input: schema::echo::EchoRequest) -> Self::EchoFuture {
        if self.fail {
            futures::future::err("OMG FAILURE!!!")
        } else {
            futures::future::ok(schema::echo::EchoResponse { data: input.data })
        }
    }
}

#[derive(Clone, Debug)]
struct GreetingService {
    fail_hello: bool,
    fail_goodbye: bool,
}

impl schema::greeting::Greeting for GreetingService {
    type Error = &'static str;
    type SayHelloFuture =
        futures::future::FutureResult<schema::greeting::SayHelloResponse, Self::Error>;
    type SayGoodbyeFuture =
        futures::future::FutureResult<schema::greeting::SayGoodbyeResponse, Self::Error>;

    fn say_hello(&self, input: schema::greeting::SayHelloRequest) -> Self::SayHelloFuture {
        if self.fail_hello {
            futures::future::err("OMG FAILED HELLO!!!")
        } else {
            futures::future::ok(schema::greeting::SayHelloResponse {
                greeting: format!("Hello, {}!", input.name),
            })
        }
    }

    fn say_goodbye(&self, input: schema::greeting::SayGoodbyeRequest) -> Self::SayGoodbyeFuture {
        if self.fail_hello {
            futures::future::err("OMG FAILED GOODBYE!!!")
        } else {
            futures::future::ok(schema::greeting::SayGoodbyeResponse {
                greeting: format!("Goodbye, {}!", input.name),
            })
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::sync;

    #[test]
    fn echo_success() {
        use schema::echo::Echo;
        use futures::Future;

        let server = schema::echo::EchoServer::new(EchoService { fail: false });
        let client = schema::echo::EchoClient::new(server);
        let data = vec![1, 2, 3];

        let response = sync::Arc::new(sync::Mutex::new(None));
        let response_clone = response.clone();
        let error = sync::Arc::new(sync::Mutex::new(None));
        let error_clone = error.clone();
        let future = client
            .echo(schema::echo::EchoRequest { data })
            .map(move |r| {
                *response_clone.lock().unwrap() = Some(r);
            })
            .map_err(move |e| {
                *error_clone.lock().unwrap() = Some(e);
            });

        tokio::run(future);

        assert_eq!(
            *response.lock().unwrap(),
            Some(schema::echo::EchoResponse {
                data: vec![1, 2, 3],
            })
        );
        assert_eq!(*error.lock().unwrap(), None);
    }

    #[test]
    fn echo_fail() {
        use schema::echo::Echo;
        use futures::Future;

        let server = schema::echo::EchoServer::new(EchoService { fail: true });
        let client = schema::echo::EchoClient::new(server);
        let data = vec![1, 2, 3];

        let response = sync::Arc::new(sync::Mutex::new(None));
        let response_clone = response.clone();
        let error = sync::Arc::new(sync::Mutex::new(None));
        let error_clone = error.clone();
        let future = client
            .echo(schema::echo::EchoRequest { data })
            .map(move |r| {
                *response_clone.lock().unwrap() = Some(r);
            })
            .map_err(move |e| {
                *error_clone.lock().unwrap() = Some(e);
            });

        tokio::run(future);

        assert_eq!(*response.lock().unwrap(), None);
        // We expect two layers of execution errors; one from the server and one from the client.
        assert_eq!(
            *error.lock().unwrap(),
            Some(prost_simple_rpc::error::Error::execution(
                prost_simple_rpc::error::Error::execution("OMG FAILURE!!!")
            ))
        );
    }
}
