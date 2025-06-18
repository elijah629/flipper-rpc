#![deny(missing_docs)]
#![deny(unused_must_use)]
#![deny(clippy::all)]

//! `flipper-rpc` is a Rust library for sending and receiving RPC messages to and
//! from a Flipper Zero over a serial connection.
//! ## Usage
//! ```no_run
//! use flipper_rpc::{
//!    error::Result,
//!    rpc::req::Request,
//!    transport::{Transport, serial::{list_flipper_ports, rpc::SerialRpcTransport}},
//! };
//!
//! # fn main() -> Result<()> {
//!    let ports = list_flipper_ports()?;
//!
//!    let port = &ports[0].port_name;
//!
//!    let mut cli = SerialRpcTransport::new(port.to_string())?;
//!
//!    let _ = cli.send_and_receive(Request::SystemPlayAudiovisualAlert)?;
//!
//! # Ok(())
//! # }
//! ```

// I don't have the time to write docs for auto-generated things
#[cfg(feature = "proto")]
#[allow(missing_docs)]
pub mod proto;

pub mod error;
pub mod logging;

#[cfg(feature = "easy-rpc")]
pub mod rpc;

#[cfg(feature = "storage")]
pub mod storage;

#[cfg(feature = "transport-any")]
pub mod transport;
