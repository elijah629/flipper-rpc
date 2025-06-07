use core::str;
use std::io::Read;
use std::time::{Duration, Instant};

use crate::error::{Error, Result};

/// Drain a stream until a specific byte is seen or a timeout elapses.
///
/// Returns `Ok(())` if the byte is found, or an errorif timed out or another I/O issue occurs.
pub fn drain_until<R: Read>(reader: &mut R, until_str: &str, timeout: Duration) -> Result<usize> {
    const CHUNK_SIZE: usize = 1024;

    let until_bytes = until_str.as_bytes();

    let mut total: usize = 0;
    let mut buffer = Vec::with_capacity(CHUNK_SIZE * 2);
    let mut buf = [0u8; CHUNK_SIZE];

    let deadline = Instant::now() + timeout;

    while Instant::now() < deadline {
        match reader.read(&mut buf) {
            Ok(0) => continue, // no data available
            Ok(n) => {
                total += n;

                buffer.extend_from_slice(&buf[..n]);

                if buffer.windows(until_bytes.len()).any(|w| w == until_bytes) {
                    return Ok(total);
                }

                // prevent buffer from growing indefinitely
                if buffer.len() > CHUNK_SIZE * 4 {
                    buffer.drain(..CHUNK_SIZE * 2);
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => continue,
            Err(e) => return Err(e.into()),
        }
    }

    Err(Error::DrainTimeout(until_str.to_string()))
}

pub fn drain<R: Read>(reader: &mut R) -> Result<usize> {
    let mut total: usize = 0;
    let mut buf = [0u8; 1024];

    loop {
        match reader.read(&mut buf) {
            Ok(0) => break,
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => break,

            Ok(n) => total += n,
            Err(e) => return Err(e.into()),
        }
    }

    Ok(total)
}
