use prost::{Message, bytes::Buf};
use serialport::SerialPort;
use std::time::Duration;

use crate::{error::Result, proto, reader_utils::drain_until};

const FLIPPER_BAUD: u32 = 115_200;

pub struct Cli {
    command_id: u32,
    port: Box<dyn SerialPort>,
}

impl Cli {
    /// (port_name, device_name)
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

    pub fn new(port: String) -> Result<Self> {
        let mut port = serialport::new(port, FLIPPER_BAUD)
            .timeout(Duration::from_secs(2))
            .open()?;

        drain_until(&mut port, ">: ", Duration::from_secs(2))?;

        port.write_all("start_rpc_session\r".as_bytes())?;
        port.flush()?;

        drain_until(&mut port, "\n", Duration::from_secs(2))?;

        Ok(Self {
            port,
            command_id: 0,
        })
    }

    /// Command ID is ignored, it gets reset to the internal counter's value
    pub fn send_rpc_proto(&mut self, rpc: proto::Main) -> Result<()> {
        let mut rpc = rpc;
        rpc.command_id = self.command_id;

        let encoded = rpc.encode_length_delimited_to_vec();
        let encoded = encoded.as_slice();

        self.port.write_all(encoded)?;

        self.command_id += 1;

        Ok(())
    }

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
                "Failed to read varint, no data read",
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

    /// Command ID is ignored, it gets reset to the internal counter's value
    pub fn send_read_rpc_proto(&mut self, rpc: proto::Main) -> Result<proto::Main> {
        self.send_rpc_proto(rpc)?;
        self.read_rpc_proto()
    }
}

/// Calculates the number of bytes required to encode a varint for the given value, without actually encoding it
fn varint_length(mut value: usize) -> usize {
    let mut len = 1;
    while value >= 0x80 {
        value >>= 7;
        len += 1;
    }
    len
}
