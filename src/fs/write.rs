//! FsWrite module

use std::path::Path;
#[cfg(feature = "fs-write-progress-mpsc")]
use std::sync::mpsc::Sender;

use tracing::debug;

use crate::{
    error::{Error, Result},
    fs::{CHUNK_SIZE, helpers::os_str_to_str},
    proto::{
        self,
        storage::{File, WriteRequest, file::FileType},
    },
    rpc::req::Request,
    transport::{TransportRaw, serial::rpc::CommandIndex},
};

/// Write traits for flipper filesystem
pub trait FsWrite {
    /// Writes a &[u8] to a file on the flipper zero to dst
    fn fs_write(
        &mut self,
        path: impl AsRef<Path>,
        data: impl AsRef<[u8]>,
        #[cfg(feature = "fs-write-progress-mpsc")] tx: Sender<(usize, usize)>,
    ) -> Result<()>;
}

impl<T> FsWrite for T
where
    T: TransportRaw<proto::Main, proto::Main, Err = Error> + CommandIndex + std::fmt::Debug,
{
    fn fs_write(
        &mut self,
        path: impl AsRef<Path>,
        data: impl AsRef<[u8]>,
        #[cfg(feature = "fs-write-progress-mpsc")] tx: Sender<(usize, usize)>,
    ) -> Result<()> {
        let path = path.as_ref();

        let path_str = os_str_to_str(path.as_os_str())?;

        let file = path.file_name().ok_or_else(||
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Path is only a directory, not a file. Use fs_mkdir instead if you intend to create a directory")
            )?.to_str().unwrap(); // SAFE: We just verified that the entire path was UTF-8 above in
        // [`path_str`]

        let data = data.as_ref();

        let chunks = chunks_or_empty(data, CHUNK_SIZE);
        let total_chunks = chunks.len();

        #[cfg(feature = "fs-write-progress-mpsc")]
        let total_data = data.len();

        #[cfg(feature = "fs-write-progress-mpsc")]
        let mut sent = 0;

        #[cfg(feature = "fs-write-progress-mpsc")]
        tx.send((sent, total_data))?;

        let command_id = self.command_index();

        debug!("writing {} bytes to {path:?}", data.len());

        for (i, chunk) in chunks.enumerate() {
            let has_next = i != total_chunks - 1; // If this is not the last chunk, it has another.

            let write_req = Request::StorageWrite(WriteRequest {
                path: path_str.to_string(),
                file: Some(File {
                    r#type: FileType::File.into(),
                    name: file.to_string(),
                    data: chunk.to_vec(),
                    size: chunk.len() as u32,
                    md5sum: format!("{:x}", md5::compute(chunk)),
                }),
            })
            .into_rpc(command_id)
            .with_has_next(has_next);

            if has_next {
                self.send_raw(write_req)?;
            } else {
                self.send_and_receive_raw(write_req)?;
            }

            #[cfg(feature = "fs-write-progress-mpsc")]
            {
                sent += chunk.len();
                tx.send((sent, total_data))?;
            }
        }

        self.increment_command_index(1);

        Ok(())
    }
}

#[inline(always)]
fn chunks_or_empty<'a>(
    data: &'a [u8],
    chunk_size: usize,
) -> Box<dyn ExactSizeIterator<Item = &'a [u8]> + 'a> {
    if data.is_empty() {
        Box::new(std::iter::once(&[][..]))
    } else {
        Box::new(data.chunks(chunk_size))
    }
}
