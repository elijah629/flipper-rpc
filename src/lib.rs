//! `flipper-rpc` is a Rust library for sending and receiving RPC messages to and
//! from a Flipper Zero over a serial connection.
//! ## Usage
//! ```rust
//! let mut cli = Cli::new("/dev/ttyACM0".to_string());
//! // or use Cli::flipper_ports() to find the port dynamically
//!  let ping = proto::Main {
//!     command_id: 0,
//!     command_status: proto::CommandStatus::Ok.into(),
//!     has_next: false,
//!     content: Some(proto::main::Content::SystemPingRequest(
//!         proto::system::PingRequest {
//!             data: vec![0xDE, 0xAD, 0xBE, 0xEF],
//!         },
//!     )),
//! };
//!
//! let response = cli.send_read_rpc_proto(ping)?;
//! println!("{response:?}");
//! ```

pub mod cli;
pub mod error;
pub mod proto;
pub mod reader_utils;
