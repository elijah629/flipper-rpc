//! Helpers for working with the flipper's filesystem through RPC.

use std::{io, path::Path};

use crate::{
    error::{Error, Result},
    proto::{
        self,
        storage::{DeleteRequest, File, ListRequest, WriteRequest, file::FileType},
    },
    rpc::{
        req::Request,
        res::{ReadFile, Response},
    },
    transport::{Transport, TransportRaw},
};

const CHUNK_SIZE: usize = 512;

/// Filesystem interaction helper for RPC. Internally uses the easy-rpc proto communication.
pub trait FlipperFs {
    /// Lists the files in a directory at path
    fn fs_readdir<P: AsRef<Path>>(&mut self, path: P) -> Result<Vec<File>>;

    /// Creates a directory at path
    fn fs_mkdir<P: AsRef<Path>>(&mut self, path: P) -> Result<()>;

    /// Removes a file or directory at path
    fn fs_rm<P: AsRef<Path>>(&mut self, path: P) -> Result<()>;

    /// Reads a file on the flipper zero from src
    fn fs_read<P: AsRef<Path>>(&mut self, path: P) -> Result<ReadFile>;

    /// Writes a &[u8] to a file on the flipper zero to dst
    fn fs_write<P: AsRef<Path>, D: AsRef<[u8]>>(&mut self, dst: P, data: D) -> Result<()>;
}

impl<T> FlipperFs for T
where
    T: TransportRaw<proto::Main, proto::Main, Err = Error> + std::fmt::Debug,
{
    fn fs_rm<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path
            .as_ref()
            .to_str()
            .ok_or(io::Error::new(
                io::ErrorKind::InvalidData,
                "Path is not UTF-8",
            ))?
            .to_string();

        let rm_req = Request::StorageDelete(DeleteRequest {
            path,
            recursive: true,
        });

        self.send_and_receive(rm_req)?;

        Ok(())
    }
    fn fs_read<P: AsRef<Path>>(&mut self, path: P) -> Result<ReadFile> {
        let path = path
            .as_ref()
            .to_str()
            .ok_or(io::Error::new(
                io::ErrorKind::InvalidData,
                "Path is not UTF-8",
            ))?
            .to_string();

        let response = self.send_and_receive(Request::StorageRead(path))?;

        match response {
            Response::Empty => Ok(ReadFile::Dir),
            Response::StorageRead(file) => {
                file.ok_or(io::Error::new(io::ErrorKind::Other, "Failed to read file").into())
            }
            _ => Err(io::Error::new(io::ErrorKind::Other, "Invalid response").into()),
        }
    }
    fn fs_mkdir<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path
            .as_ref()
            .to_str()
            .ok_or(io::Error::new(
                io::ErrorKind::InvalidData,
                "Path is not UTF-8",
            ))?
            .to_string();

        self.send_and_receive(Request::StorageMkdir(path))?;

        Ok(())
    }
    fn fs_write<P: AsRef<Path>, D: AsRef<[u8]>>(&mut self, dst: P, data: D) -> Result<()> {
        let dst = dst.as_ref();

        let path = dst.to_str().ok_or(io::Error::new(
            io::ErrorKind::InvalidData,
            "Path is not UTF-8",
        ))?;

        let file = dst.file_name().ok_or(
            io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Path is only a directory, not a file. Use fs_mkdir instead if you intend to create a directory")
            )?.
            to_str()
            .ok_or(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Path is not UTF-8",
            ))?
            .to_string();

        let data = data.as_ref();

        if data.is_empty() {
            let write_req = Request::StorageWrite(WriteRequest {
                path: path.to_string(),
                file: Some(File {
                    r#type: FileType::File.into(),
                    name: file.clone(),
                    data: vec![],
                    size: 0,
                    md5sum: "d41d8cd98f00b204e9800998ecf8427e".to_string(), // Empty MD5
                }),
            });

            self.send_and_receive(write_req)?;

            return Ok(());
        }

        let chunks = data.chunks(CHUNK_SIZE);
        let length = chunks.len();

        for (i, chunk) in chunks.enumerate() {
            let last_chunk = i == length - 1;

            let write_req = Request::StorageWrite(WriteRequest {
                path: path.to_string(),
                file: Some(File {
                    r#type: FileType::File.into(),
                    name: file.clone(),
                    data: data.to_vec(),
                    size: chunk.len() as u32,
                    md5sum: format!("{:x}", md5::compute(data)),
                }),
            })
            .into_rpc(!last_chunk);

            self.send_and_receive_raw(write_req)?;
        }

        Ok(())
    }
    fn fs_readdir<P: AsRef<Path>>(&mut self, path: P) -> Result<Vec<File>> {
        let path = path
            .as_ref()
            .to_str()
            .ok_or(io::Error::new(
                io::ErrorKind::InvalidData,
                "Path is not UTF-8",
            ))?
            .to_string();

        let req = Request::StorageList(ListRequest {
            path,
            // not including MD5 here, as it is not practical to check against it when we don't
            // have the file data
            ..Default::default()
        });

        let res = self.send_and_receive(req)?.try_into()?;

        Ok(res)
    }
}
