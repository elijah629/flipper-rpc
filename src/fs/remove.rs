//! FsRemove module

use std::path::Path;

use crate::proto::storage::DeleteRequest;
use crate::transport::Transport;
use crate::{
    error::{Error, Result},
    proto,
    rpc::req::Request,
    transport::TransportRaw,
};

/// ReadDir traits for flipper filesystem
pub trait FsRemove {
    /// Removes a file or directory at path
    fn fs_remove(&mut self, path: impl AsRef<Path>, recursive: bool) -> Result<()>;
}

impl<T> FsRemove for T
where
    T: TransportRaw<proto::Main, proto::Main, Err = Error> + std::fmt::Debug,
{
    fn fs_remove(&mut self, path: impl AsRef<Path>, recursive: bool) -> Result<()> {
        let path = path
            .as_ref()
            .to_str()
            .ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "Path is not UTF-8")
            })?
            .to_string();

        let rm_req = Request::StorageDelete(DeleteRequest { path, recursive });

        self.send_and_receive(rm_req)?;

        Ok(())
    }
}
