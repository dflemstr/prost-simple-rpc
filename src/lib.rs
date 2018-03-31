//! Common RPC definitions for various communication protocols.
//!
//! These RPC definitions can be used for a wide variety of transport protocols as long as they can
//! agree on using protobuf-derived message schemata.

#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(missing_copy_implementations)]
#![deny(trivial_casts)]
#![deny(trivial_numeric_casts)]
#![deny(unsafe_code)]
#![deny(unstable_features)]
#![deny(unused_import_braces)]
#![deny(unused_qualifications)]
#![cfg_attr(feature = "dev", allow(unstable_features))]
#![cfg_attr(feature = "dev", feature(plugin))]
#![cfg_attr(feature = "dev", plugin(clippy))]

extern crate bytes;
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate futures;
extern crate prost;

pub mod descriptor;
pub mod error;
pub mod handler;
#[doc(hidden)]
pub mod __rt;
