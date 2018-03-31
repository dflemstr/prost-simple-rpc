//! Traits for generic service descriptor definitions.
//!
//! These traits are built on the assumption that some form of code generation is being used (e.g.
//! using only `&'static str`s) but it's of course possible to implement these traits manually.
use std::any;

/// A descriptor for an available RPC service.
pub trait ServiceDescriptor {
    /// The associated type of method descriptors.
    type Method: MethodDescriptor;

    /// The name of the service, used in Rust code and perhaps for human readability.
    fn name() -> &'static str;

    /// The raw protobuf name of the service.
    fn proto_name() -> &'static str;

    /// All of the available methods on the service.
    fn methods() -> &'static [Self::Method];
}

/// A descriptor for a method available on an RPC service.
pub trait MethodDescriptor: Copy {
    /// The name of the service, used in Rust code and perhaps for human readability.
    fn name(&self) -> &'static str;

    /// The raw protobuf name of the service.
    fn proto_name(&self) -> &'static str;

    /// The Rust `TypeId` for the input that this method accepts.
    fn input_type(&self) -> any::TypeId;

    /// The raw protobuf name for the input type that this method accepts.
    fn input_proto_type(&self) -> &'static str;

    /// The Rust `TypeId` for the output that this method produces.
    fn output_type(&self) -> any::TypeId;

    /// The raw protobuf name for the output type that this method produces.
    fn output_proto_type(&self) -> &'static str;
}
