#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![deny(missing_docs)]
#![deny(unused_must_use)]
#![deny(clippy::all)]

//! `flipper-rpc` is a Rust library for sending and receiving RPC messages to and
//! from a Flipper Zero over a serial connection.
//!
//! ## Usage
//!
//! Cannot be described here, please see each submodule for examples as they all depend on features not available here.

// I don't have the time to write docs for auto-generated things
#[cfg(feature = "proto")]
#[allow(missing_docs)]
pub mod proto;

pub mod error;
pub mod logging;

#[cfg(feature = "easy-rpc")]
pub mod rpc;

#[cfg(feature = "fs-any")]
pub mod fs;

#[cfg(feature = "transport-any")]
pub mod transport;
