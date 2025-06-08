use core::str;
use std::io::{ErrorKind, Read};
use std::time::{Duration, Instant};

use crate::error::Result;

/// Drain a stream until a str + padding chunk
///
/// Returns `Ok(())` if the byte is found, or an error if timed out or another I/O issue occurs.
pub fn drain_until<R: Read>(reader: &mut R, until_str: &str, timeout: Duration) -> Result<()> {
    assert!(!until_str.is_empty(), "until_str must not be empty");

    const CHUNK_SIZE: usize = 256;

    // INFO: In the worst case scenario, where the string starts @ the last byte in a chunk, it
    // must fit within the second chunk, otherwise it will not be found.
    assert!(until_str.len() <= CHUNK_SIZE + 1);

    let until_bytes = until_str.as_bytes();

    const BUF_LEN: usize = CHUNK_SIZE * 2;

    let mut buf = [0u8; BUF_LEN]; // Two chunk juggle

    let deadline = Instant::now() + timeout;

    let finder = memchr::memmem::Finder::new(until_bytes);

    let mut filled = 0;

    loop {
        if filled > CHUNK_SIZE {
            buf.copy_within(CHUNK_SIZE.., 0);
            filled -= CHUNK_SIZE;
        }

        let now = Instant::now();
        if now >= deadline + timeout {
            break;
        }

        match reader.read(&mut buf[filled..filled + CHUNK_SIZE.min(BUF_LEN - filled)]) {
            Ok(0) => {
                std::thread::sleep(Duration::from_millis(10));
                continue;
            }
            Ok(n) => {
                filled += n;

                if finder.find(&buf).is_some() {
                    return Ok(());
                }
            }
            Err(ref e) if e.kind() == ErrorKind::TimedOut => continue,
            Err(e) => return Err(e.into()),
        }
    }

    Err(std::io::Error::new(
        ErrorKind::TimedOut,
        format!("Timeout searching for '{}'", until_str),
    )
    .into())
}
