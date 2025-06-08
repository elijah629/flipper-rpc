//! `flipper-rpc` is a Rust library for sending and receiving RPC messages to and
//! from a Flipper Zero over a serial connection.
//! ## Usage
//! ```rust
//! # fn main() -> std::io::Result<()> {
//!    let ports = list_flipper_ports()?;
//!
//!    let port = &ports[0].port_name;
//!
//!    let mut cli = SerialRpcTransport::new(port.to_string())?;
//!
//!    let response = cli.send_and_receive(RpcRequest::SystemPlayAudiovisualAlert)?;
//!
//!    assert!(response.is_none());
//!
//! # Ok(())
//! # }
//! ```

pub mod proto;

#[deny(missing_docs)]
pub mod rpc;
#[deny(missing_docs)]
pub mod transport;
