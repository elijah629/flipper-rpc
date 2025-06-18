//! A transport that sends RPC messages on a port
//!
//! # Examples
//!
//! ```no_run
//! use flipper_rpc::{rpc::{res::Response, req::Request}, error::Result, transport::serial::rpc::SerialRpcTransport};
//! use flipper_rpc::transport::Transport;
//!
//! # fn main() -> Result<()> {
//! let mut cli = SerialRpcTransport::new("/dev/ttyACM0".to_string())?;
//!
//! let resp = cli.send_and_receive(Request::Ping(vec![1, 2, 3, 4]))?; // or send_raw for raw proto messages!
//!
//! assert_eq!(resp, Response::Ping(vec![1, 2, 3, 4]));
//! # Ok(())
//! # }
//! ```
use crate::error::{Error, Result};
use crate::logging::{debug, trace};
use crate::{
    proto,
    transport::{
        TransportRaw,
        serial::{
            FLIPPER_BAUD,
            helpers::{drain_until, drain_until_str},
        },
    },
};
use prost::Message;
use serialport::SerialPort;
use std::time::Duration;

/// A transport that sends RPC messages on a port
///
/// # Examples
///
/// ```no_run
/// use flipper_rpc::transport::serial::rpc::SerialRpcTransport;
/// use flipper_rpc::rpc::{req::Request, res::Response};
/// use flipper_rpc::error::Result;
/// use flipper_rpc::transport::Transport;
///
/// # fn main() -> Result<()> {
/// let mut cli = SerialRpcTransport::new("/dev/ttyACM0".to_string())?;
///
/// let resp = cli.send_and_receive(Request::Ping(vec![1, 2, 3, 4]))?; // or send_raw for raw proto messages!
///
/// assert_eq!(resp, Response::Ping(vec![1, 2, 3, 4]));
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct SerialRpcTransport {
    command_index: u32,
    port: Box<dyn SerialPort>,
}

impl SerialRpcTransport {
    /// Opens a new RPC session on the given serial port path.
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
    /// use flipper_rpc::{error::Result, transport::serial::rpc::SerialRpcTransport};
    ///
    /// # fn main() -> Result<()> {
    ///
    /// let mut cli = SerialRpcTransport::new("/dev/ttyACM0".to_string())?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn new<S: AsRef<str> + std::fmt::Debug>(port: S) -> Result<Self> {
        let mut port = serialport::new(port.as_ref(), FLIPPER_BAUD)
            .timeout(Duration::from_secs(2))
            .open()?;

        debug!("Draining port until prompt");
        drain_until_str(&mut port, ">: ", Duration::from_secs(2))?;

        debug!("Calling start_rpc_session");
        port.write_all("start_rpc_session\r".as_bytes())?;
        port.flush()?;

        debug!("Draining until start_rpc_session has a \\n");
        drain_until(&mut port, b'\n', Duration::from_secs(2))?;

        Ok(Self {
            command_index: 0,
            port,
        })
    }

    /// Wraps a SerialPort with a SerialRpcTransport
    /// WARN: Does not reconfigure the port, just passes it into the internal holder, you must make
    /// sure that the port is in an RPC session. To convert a SerialCliTransport into
    /// a SerialRpcTransport, use SerialCliTransport::into_rpc(self) instead.
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    pub fn from_port(port: Box<dyn SerialPort>) -> Result<Self> {
        Ok(Self {
            command_index: 0,
            port,
        })
    }
}

impl TransportRaw<proto::Main> for SerialRpcTransport {
    type Err = Error;

    /// Sends a length-delimited Protobuf RPC message to the Flipper.
    ///
    /// The internal command counter is used to set `command_id` on the message, whatever value is
    /// set in `value` will be overwritten
    ///
    /// # Errors
    ///
    /// Returns an error if the message cannot be encoded or written to the port.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use flipper_rpc::proto;
    /// use flipper_rpc::proto::CommandStatus;
    /// use flipper_rpc::proto::main::Content;
    /// use flipper_rpc::proto::system;
    /// use flipper_rpc::transport::serial::rpc::SerialRpcTransport;
    /// use flipper_rpc::transport::TransportRaw;
    /// use flipper_rpc::error::Result;
    ///
    /// # fn main() -> Result<()> {
    ///
    /// let mut cli = SerialRpcTransport::new("/dev/ttyACM0")?;
    ///
    /// let ping = proto::Main {
    /// command_id: 0,
    ///     command_status: proto::CommandStatus::Ok.into(),
    ///     has_next: false,
    ///     content: Some(proto::main::Content::SystemPingRequest(
    ///         system::PingRequest {
    ///             data: vec![0xDE, 0xAD, 0xBE, 0xEF],
    ///         },
    ///     )),
    /// };
    /// cli.send_raw(ping)?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    fn send_raw(&mut self, mut value: proto::Main) -> std::result::Result<(), Self::Err> {
        trace!("send_rpc_proto");
        debug!("command index: {}", self.command_index);

        value.command_id = self.command_index;

        let encoded = value.encode_length_delimited_to_vec();
        self.port.write_all(&encoded)?;

        self.port.flush()?;

        // Command streams of has_next for chunked data MUST share the same command ID. The entire
        // chain must have it. This will inc after data is sent and the chain will have the same id
        // for all
        if !value.has_next {
            self.command_index = self.command_index.wrapping_add(1);
        }

        Ok(())
    }

    /// Reads a length-delimited Protobuf RPC message from the flipper. This must be called
    /// directly after data is sent, and cannot be called after a message is sent before (will
    /// panic)
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
    /// use flipper_rpc::transport::serial::rpc::SerialRpcTransport;
    /// use flipper_rpc::error::Result;
    /// use flipper_rpc::transport::TransportRaw;
    ///
    /// # fn main() -> Result<()> {
    /// let mut cli = SerialRpcTransport::new("/dev/ttyACM0".to_string())?;
    /// let response = cli.receive_raw()?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    #[cfg(feature = "serial-optimized-varint-reading")]
    fn receive_raw(&mut self) -> std::result::Result<proto::Main, Self::Err> {
        use prost::bytes::Buf;

        use crate::transport::serial::helpers::varint_length;

        self.port.flush()?;

        trace!("read_rpc_proto");

        trace!("optimized-2shot-read");
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
        // ```no_run
        // let read = self.port.read(&mut buf)?;
        // ```
        //
        // Error-proof code!

        let mut read = 0;
        let mut available_bytes = buf.len();

        debug!("Reading varint");
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

        debug!("Decoding data length");
        let total_data_length = prost::decode_length_delimiter(&buf[..read])?;

        // We have the length of the data, however some or all of the actual data is inside of buf,
        // after the varint, it just continues to RPC data.

        // How many bytes does the varint take up?
        debug!("Calculating varint length");
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

            debug!("FASTEST: Decoding response");
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
                debug!("FAST: Decoding response");
                // Free speed for small messages!
                let mut stack_buf = [0u8; STACK_LIMIT];
                self.port.read_exact(&mut stack_buf[..remaining_length])?;

                let chained = partial_data.chain(&stack_buf[..remaining_length]);

                proto::Main::decode(chained)?
            } else {
                debug!("SLOW: Decoding response");
                // Uses a slower heap (vec) based decoding for larger messages.
                let mut remaining_data = vec![0u8; remaining_length];
                self.port.read_exact(&mut remaining_data)?;

                let chained = partial_data.chain(remaining_data.as_slice());

                proto::Main::decode(chained)?
            }
        };

        Ok(main)
    }

    /// Reads a length-delimited Protobuf RPC message from the flipper. This must be called
    /// directly after data is sent, and cannot be called after a message is sent before (will
    /// panic)
    ///
    /// Reads the next RPC message from the Flipper using a byte-wise varint decoder.
    ///
    /// This method issues up to 11 syscalls but and relies on only heap buffers.
    /// Opt to use read_rpc_proto when possible
    /// NOTE: Optimized method disabled
    #[cfg(not(feature = "serial-optimized-varint-reading"))]
    #[cfg_attr(feature = "tracing", tracing::instrument)]
    fn receive_raw(&mut self) -> Result<proto::Main, Self::Err> {
        trace!("bytewise-read");
        // NOTE: Comapred to the one above, this looks stupid and shitty. It makes a maximum of 11
        // syscalls, with a minimum of 2. 11 for large messages and 2 for messages < 127 bytes.
        // It also only relies on the heap
        //
        // Useful for less-complex and very small transfers. Otherwise use the other version

        self.port.flush()?;

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
}
