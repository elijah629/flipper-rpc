//! # Flipper CLI
//!
//! A client for communicating with Flipper Zero devices over a serial port using Protobuf RPC.
//!
//! This crate provides a `Cli` struct which manages an RPC session, sending and receiving messages
//! defined in the `proto` module.
//!
//! ## Examples
//!
//! ```no_run
//! use flipper_cli::Cli;
//! use flipper_cli::error::Result;
//!
//! # fn main() -> Result<()> {
//! // List available Flipper devices
//! if let Some(ports) = Cli::flipper_ports() {
//!     for (port_name, product) in ports {
//!         println!("Found {} on {}", product, port_name);
//!     }
//! }
//!
//! // Connect to first device
//! let mut cli = Cli::new("/dev/ttyUSB0".to_string())?;
//!
//! // Send a ping RPC message
//! let ping = proto::Main::default();
//! cli.send_rpc_proto(ping)?;
//!
//! // Read response
//! let response = cli.read_rpc_proto()?;
//! println!("Received: {:?}", response);
//! # Ok(())
//! # }
//! ```

#![deny(missing_docs)]

use prost::{Message, bytes::Buf};
use serialport::SerialPort;
use std::time::Duration;

use crate::{error::Result, proto, reader_utils::drain_until};

const FLIPPER_BAUD: u32 = 115_200;

/// Flipper RPC communication class
#[derive(Debug)]
pub struct Cli {
    command_id: u32,
    port: Box<dyn SerialPort>,
}

impl Cli {
    /// Lists all connected Flipper devices by USB serial port and product name.
    ///
    /// # Returns
    ///
    /// - `Some(Vec<(port_name, product_name)>)` if querying ports succeeds.
    /// - `None` if listing ports fails, or no ports are available.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// if let Some(devices) = Cli::flipper_ports() {
    ///     for (port, name) in devices {
    ///         println!("{}: {}", port, name);
    ///     }
    /// }
    /// ```
    pub fn flipper_ports() -> Option<Vec<(String, String)>> {
        serialport::available_ports().ok().map(|ports| {
            ports
                .into_iter()
                .filter_map(|port| {
                    if let serialport::SerialPortType::UsbPort(usb_info) = port.port_type {
                        if usb_info.manufacturer.as_deref() == Some("Flipper Devices Inc.") {
                            if let Some(product) = usb_info.product {
                                return Some((port.port_name, product));
                            }
                        }
                    }
                    None
                })
                .collect()
        })
    }

    /// Opens a new CLI session on the given serial port path.
    ///
    /// Initialize the serial connection and start an RPC session.
    ///
    /// # Errors
    ///
    /// Returns an error if the port cannot be opened, initialization commands fail, or
    /// the RPC banner prompt is not received.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let mut cli = Cli::new("/dev/ttyACM0".to_string())?;
    /// ```
    pub fn new(port: String) -> Result<Self> {
        let mut port = serialport::new(port, FLIPPER_BAUD)
            .timeout(Duration::from_secs(2))
            .open()?;

        // INFO: Drains port until tty prompt
        drain_until(&mut port, ">: ", Duration::from_secs(2))?;

        port.write_all("start_rpc_session\r".as_bytes())?;
        port.flush()?;

        // INFO: Waits till start_rpc_session responds
        drain_until(&mut port, "\n", Duration::from_secs(2))?;

        Ok(Self {
            port,
            command_id: 0,
        })
    }

    /// Sends a length-delimited Protobuf RPC message to the Flipper.
    ///
    /// The internal command counter is used to set `command_id` on the message, whatever value is
    /// set in `rpc` will be overwritten
    ///
    /// # Errors
    ///
    /// Returns an error if the message cannot be encoded or written to the port.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let ping = proto::Main {
    /// command_id: 0,
    ///     command_status: proto::CommandStatus::Ok.into(),
    ///     has_next: false,
    ///     content: Some(proto::main::Content::SystemPingRequest(
    ///         proto::system::PingRequest {
    ///             data: vec![0xDE, 0xAD, 0xBE, 0xEF],
    ///         },
    ///     )),
    /// };
    /// cli.send_rpc_proto(ping)?;
    /// ```
    pub fn send_rpc_proto(&mut self, mut rpc: proto::Main) -> Result<()> {
        rpc.command_id = self.command_id;

        let encoded = rpc.encode_length_delimited_to_vec();
        self.port.write_all(&encoded)?;

        self.command_id = self.command_id.wrapping_add(1);

        Ok(())
    }

    /// Reads the next RPC message from the Flipper using a two-stage buffered read.
    ///
    /// Uses a two-shot method of reading: first to get varint length + partial data, then to
    /// fetch remaining bytes if the message exceeds the initial buffer.
    ///
    /// # Errors
    ///
    /// Returns an error if no data is received, decoding fails, or IO operations fail.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let response = cli.read_rpc_proto()?;
    /// ```
    pub fn read_rpc_proto(&mut self) -> Result<proto::Main> {
        // INFO: Super-overcomplicated but fast and efficent way of reading any length varint + data in exactly two
        // syscalls
        // Tries to use a stack-based approach when possible and does it efficently

        // Hard limit for all stack-based buffers
        // NOTE: Adding 10 as Varint max length is 10
        const STACK_LIMIT: usize = 10 + 128;

        let mut buf = [0u8; STACK_LIMIT];

        // Yeah that first comment was somewhat of a lie, it should be a MINIMUM of two reads.
        // If the first read fails, we wouldn't know and it would return incomplete data.
        // So we have to have a fail-safe loop. This actually does not cost much, as it will
        // still read only once if it doesn't fail

        // Error-prone code
        // ```rs
        // let read = self.port.read(&mut buf)?;
        // ```
        //
        // Error-proof code!

        let mut read = 0;
        let mut available_bytes = buf.len();

        while read < available_bytes {
            match self.port.read(&mut buf[read..available_bytes]) {
                Ok(0) => break, // No more data
                Ok(n) => {
                    available_bytes = self.port.bytes_to_read()? as usize;
                    read += n
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => break,
                Err(e) => return Err(e.into()),
            }
        }

        if read == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "no data read, failed to parse varint",
            )
            .into());
        }

        let total_data_length = prost::decode_length_delimiter(&buf[..read])?;

        // We have the length of the data, however some or all of the actual data is inside of buf,
        // after the varint, it just continues to RPC data.

        // How many bytes does the varint take up?
        let varint_length = varint_length(total_data_length);

        // PERF: All the data that is not varint data, this is another main optimization,
        // we skip another read, as we have already read the data.
        // We get all data after the varint until we stopped reading
        let partial_data = &buf[varint_length..read];

        // NOTE: We can skip the math since varint_length is always `1` for numbers 0-9 (varints go
        // above 1 byte when value > 127, since they only use 7 bits, and the MSB is an indicator
        // of weather the varint is done). If we read
        // more data, we would have to do:
        // total_data_length <= 10 - varint_length or total_data_length <= partial_data.len()
        let read_all_data = total_data_length <= partial_data.len();

        // PERF: If all of the data was read, the entire message is contained within the 10 byte buffer,
        // so we do not need to perfom another read operation
        let main = if read_all_data {
            // INFO: partial_data is all of the data in the buffer besides the varint and
            // trailing zeros if we read less than the buf's size

            proto::Main::decode(partial_data)?
        } else {
            // WARN: Data did NOT fit inside of the buffer, this means that some of the data is
            // missing from the buffer

            // `partial_data` is the only data that was in the buffer

            // Now we need to get the remaining bytes and join them together with partial_data to
            // get the full data, then we should decode it

            // PERF: Optimization alert! As no one expected, stack buffers are waaay faster than vecs.
            // Capiitalizing on this, all messages with < STACK_LIMIT bytes will be put (partially)
            // into a stack buffer. Then, we can chain them with bytes::buf::Buf and pass that
            // directly to the decoder for a zero-overhead decoding
            //
            // PERF: For messages larger than STACK_LIMIT, we do a vec based processing. Since stack
            // sizes must be known at compile time, this is the only way to do a stack-processing
            // method with data that could be infinitely large.

            // How much data is left
            let remaining_length = total_data_length - partial_data.len();

            if remaining_length <= STACK_LIMIT {
                // Free speed for small messages!
                let mut stack_buf = [0u8; STACK_LIMIT];
                self.port.read_exact(&mut stack_buf[..remaining_length])?;

                let chained = partial_data.chain(&stack_buf[..remaining_length]);

                proto::Main::decode(chained)?
            } else {
                // Uses a slower heap (vec) based decoding for larger messages.
                let mut remaining_data = vec![0u8; remaining_length];
                self.port.read_exact(&mut remaining_data)?;

                let data = [partial_data.to_vec(), remaining_data].concat();

                proto::Main::decode(data.as_slice())?
            }
        };

        Ok(main)
    }

    /// Reads the next RPC message from the Flipper using a byte-wise varint decoder.
    ///
    /// This method issues up to 11 syscalls but and relies on only heap buffers.
    /// Opt to use read_rpc_proto when possible
    ///
    /// # Errors
    ///
    /// Returns an error if IO operations fail or decoding fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let response = cli.read_rpc_proto_bytewise()?;
    /// ```
    pub fn read_rpc_proto_bytewise(&mut self) -> Result<proto::Main> {
        // NOTE: Comapred to the one above, this looks stupid and shitty. It makes a maximum of 11
        // syscalls, with a minimum of 2. 11 for large messages and 2 for messages < 127 bytes.
        // It also only relies on the heap

        let mut buf = Vec::with_capacity(10);
        let mut byte = [0u8; 1];

        loop {
            self.port.read_exact(&mut byte)?;
            buf.push(byte[0]);
            if byte[0] & 0x80 == 0 {
                break;
            }
        }

        let len = prost::decode_length_delimiter(buf.as_slice())?;
        let mut msg_buf = vec![0u8; len];
        self.port.read_exact(&mut msg_buf)?;

        let main = proto::Main::decode(msg_buf.as_slice())?;

        Ok(main)
    }

    /// Sends an RPC request and immediately reads the response.
    ///
    /// # Errors
    ///
    /// Propagates errors from `send_rpc_proto` or `read_rpc_proto`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let response = cli.send_read_rpc_proto(request)?;
    /// ```
    pub fn send_read_rpc_proto(&mut self, rpc: proto::Main) -> Result<proto::Main> {
        self.send_rpc_proto(rpc)?;
        self.read_rpc_proto()
    }
}

/// Returns the number of bytes used by the varint encoding of `value`.
///
/// This does not allocate or write the varint, only computes its length.
///
/// # Examples
///
/// ```
/// let len = varint_length(300);
/// assert_eq!(len, 2);
/// ```
fn varint_length(mut value: usize) -> usize {
    let mut len = 1;
    while value >= 0x80 {
        value >>= 7;
        len += 1;
    }
    len
}
