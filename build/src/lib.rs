extern crate heck;
extern crate prost_build;

use std::fmt;

pub struct ServiceGenerator;

impl prost_build::ServiceGenerator for ServiceGenerator {
    fn generate(&mut self, service: prost_build::Service, mut buf: &mut String) {
        use std::fmt::Write;
        use heck::CamelCase;

        let mut trait_types = String::new();
        let mut trait_methods = String::new();
        let mut enum_methods = String::new();
        let mut list_enum_methods = String::new();
        let mut client_types = String::new();
        let mut client_methods = String::new();
        let mut client_own_methods = String::new();
        let mut match_name_methods = String::new();
        let mut match_proto_name_methods = String::new();
        let mut match_input_type_methods = String::new();
        let mut match_input_proto_type_methods = String::new();
        let mut match_output_type_methods = String::new();
        let mut match_output_proto_type_methods = String::new();
        let mut match_handle_methods = String::new();

        for method in service.methods {
            assert!(
                !method.client_streaming,
                "Client streaming not yet supported for method {}",
                method.proto_name
            );
            assert!(
                !method.server_streaming,
                "Server streaming not yet supported for method {}",
                method.proto_name
            );

            writeln!(
                trait_types,
                "    type {camel_case_name}Future: ::futures::Future<Item = {output_type}, Error = Self::Error> + Send;",
                camel_case_name = method.name.to_camel_case(),
                output_type = method.output_type
            ).unwrap();

            ServiceGenerator::write_comments(&mut trait_methods, 4, &method.comments).unwrap();
            writeln!(
                trait_methods,
                r#"    fn {name}(&self, input: {input_type}) -> Self::{camel_case_name}Future;"#,
                name = method.name,
                camel_case_name = method.name.to_camel_case(),
                input_type = method.input_type
            ).unwrap();

            ServiceGenerator::write_comments(&mut enum_methods, 4, &method.comments).unwrap();
            writeln!(enum_methods, "    {name},", name = method.proto_name).unwrap();
            writeln!(
                list_enum_methods,
                "            {service_name}MethodDescriptor::{name},",
                service_name = service.name,
                name = method.proto_name
            ).unwrap();

            writeln!(
                client_types,
                "    type {camel_case_name}Future = Box<::futures::Future<Item = {output_type}, Error = Self::Error> + Send>;",
                camel_case_name = method.name.to_camel_case(),
                output_type = method.output_type
            ).unwrap();

            writeln!(
                client_methods,
                r#"    fn {name}(&self, input: {input_type})
        -> Self::{camel_case_name}Future
    {{
        {client_name}::{name}_inner(self.0.clone(), input)
    }}
"#,
                name = method.name,
                camel_case_name = method.name.to_camel_case(),
                input_type = method.input_type,
                client_name = format!("{}Client", service.name)
            ).unwrap();

            writeln!(
                client_own_methods,
                r#"    fn {name}_inner(mut handler: H, input: {input_type})
        -> <Self as {trait_name}>::{camel_case_name}Future
    {{
        use futures::Future;
        Box::new(::futures::future::result(::prost_simple_rpc::__rt::encode(input))
            .and_then(move |i| {{
                handler.call({service_name}MethodDescriptor::{proto_name}, i)
                    .map_err(|e| ::prost_simple_rpc::error::Error::execution(e))
            }})
            .and_then(::prost_simple_rpc::__rt::decode))
    }}
"#,
                trait_name = service.name,
                name = method.name,
                camel_case_name = method.name.to_camel_case(),
                input_type = method.input_type,
                service_name = service.name,
                proto_name = method.proto_name
            ).unwrap();

            let case = format!(
                "            {service_name}MethodDescriptor::{proto_name} => ",
                service_name = service.name,
                proto_name = method.proto_name
            );

            writeln!(match_name_methods, "{}{:?},", case, method.name).unwrap();
            writeln!(match_proto_name_methods, "{}{:?},", case, method.proto_name).unwrap();
            writeln!(
                match_input_type_methods,
                "{}::std::any::TypeId::of::<{}>(),",
                case, method.input_type
            ).unwrap();
            writeln!(
                match_input_proto_type_methods,
                "{}{:?},",
                case, method.input_proto_type
            ).unwrap();
            writeln!(
                match_output_type_methods,
                "{}::std::any::TypeId::of::<{}>(),",
                case, method.output_type
            ).unwrap();
            writeln!(
                match_output_proto_type_methods,
                "{}{:?},",
                case, method.output_proto_type
            ).unwrap();
            write!(
                match_handle_methods,
                r#"{}
                Box::new(
                    ::futures::future::result(::prost_simple_rpc::__rt::decode(input))
                        .and_then(move |i| {{
                            service.{name}(i).map_err(|e| ::prost_simple_rpc::error::Error::execution(e))
                        }})
                        .and_then(::prost_simple_rpc::__rt::encode)),
"#,
                case,
                name = method.name
            ).unwrap();
        }

        ServiceGenerator::write_comments(&mut buf, 0, &service.comments).unwrap();
        write!(
            buf,
            r#"pub trait {name} {{
    type Error: ::failure::Fail;
{trait_types}
{trait_methods}}}
/// A service descriptor for a `{name}`.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct {descriptor_name};
/// A server for a `{name}`.
///
/// This implements the `Server` trait by handling requests and dispatch them to methods on the
/// supplied `{name}`.
#[derive(Clone, Debug)]
pub struct {server_name}<A>(A) where A: {name} + Clone + Send + 'static;
/// A client for a `{name}`.
///
/// This implements the `{name}` trait by dispatching all method calls to the supplied `Handler`.
#[derive(Clone, Debug)]
pub struct {client_name}<H>(H) where H: ::prost_simple_rpc::handler::Handler;
/// A method available on a `{name}`.
///
/// This can be used as a key when routing requests for servers/clients of a `{name}`.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum {method_descriptor_name} {{
{enum_methods}}}
impl<A> {server_name}<A> where A: {name} + Clone + Send + 'static {{
    pub fn new(service: A) -> {server_name}<A> {{
        {server_name}(service)
    }}

    fn call_inner(
        service: A,
        method: {method_descriptor_name},
        input: ::bytes::Bytes)
        -> <Self as ::prost_simple_rpc::handler::Handler>::CallFuture
    {{
        use futures::Future;

        match method {{
{match_handle_methods}        }}
    }}
}}
impl<H> {client_name}<H> where H: ::prost_simple_rpc::handler::Handler<Descriptor = {descriptor_name}> {{
    pub fn new(handler: H) -> {client_name}<H> {{
        {client_name}(handler)
    }}
}}
impl ::prost_simple_rpc::descriptor::ServiceDescriptor for {descriptor_name} {{
    type Method = {method_descriptor_name};
    fn name() -> &'static str {{ {name:?} }}
    fn proto_name() -> &'static str {{ {proto_name:?} }}
    fn methods() -> &'static [Self::Method] {{
        &[
{list_enum_methods}        ]
    }}
}}
impl<A> ::prost_simple_rpc::handler::Handler for {server_name}<A> where A: {name} + Clone + Send + 'static {{
    type Error = ::prost_simple_rpc::error::Error<<A as {name}>::Error>;
    type Descriptor = {descriptor_name};
    type CallFuture = Box<::futures::Future<Item = ::bytes::Bytes, Error = Self::Error> + Send>;

    fn call(
        &mut self,
        method: {method_descriptor_name},
        input: ::bytes::Bytes)
        -> Self::CallFuture
    {{
        {server_name}::call_inner(self.0.clone(), method, input)
    }}
}}
impl<H> {client_name}<H> where H: ::prost_simple_rpc::handler::Handler<Descriptor = {descriptor_name}> {{
{client_own_methods}}}
impl<H> {name} for {client_name}<H> where H: ::prost_simple_rpc::handler::Handler<Descriptor = {descriptor_name}> {{
    type Error = ::prost_simple_rpc::error::Error<H::Error>;
{client_types}
{client_methods}}}
impl ::prost_simple_rpc::descriptor::MethodDescriptor for {method_descriptor_name} {{
    fn name(&self) -> &'static str {{
        match *self {{
{match_name_methods}        }}
    }}
    fn proto_name(&self) -> &'static str {{
        match *self {{
{match_proto_name_methods}        }}
    }}
    fn input_type(&self) -> ::std::any::TypeId {{
        match *self {{
{match_input_type_methods}        }}
    }}
    fn input_proto_type(&self) -> &'static str {{
        match *self {{
{match_input_proto_type_methods}        }}
    }}
    fn output_type(&self) -> ::std::any::TypeId {{
        match *self {{
{match_output_type_methods}        }}
    }}
    fn output_proto_type(&self) -> &'static str {{
        match *self {{
{match_output_proto_type_methods}        }}
    }}
}}
"#,
            name = service.name,
            descriptor_name = format!("{}Descriptor", service.name),
            server_name = format!("{}Server", service.name),
            client_name = format!("{}Client", service.name),
            method_descriptor_name = format!("{}MethodDescriptor", service.name),
            proto_name = service.proto_name,
            trait_types = trait_types,
            trait_methods = trait_methods,
            enum_methods = enum_methods,
            list_enum_methods = list_enum_methods,
            client_own_methods = client_own_methods,
            client_types = client_types,
            client_methods = client_methods,
            match_name_methods = match_name_methods,
            match_proto_name_methods = match_proto_name_methods,
            match_input_type_methods = match_input_type_methods,
            match_input_proto_type_methods = match_input_proto_type_methods,
            match_output_type_methods = match_output_type_methods,
            match_output_proto_type_methods = match_output_proto_type_methods,
            match_handle_methods = match_handle_methods
        ).unwrap();
    }
}

impl ServiceGenerator {
    fn write_comments<W>(
        mut write: W,
        indent: usize,
        comments: &prost_build::Comments,
    ) -> fmt::Result
    where
        W: fmt::Write,
    {
        for comment in &comments.leading {
            for line in comment.lines().filter(|s| !s.is_empty()) {
                writeln!(write, "{}///{}", " ".repeat(indent), line)?;
            }
        }
        Ok(())
    }
}
