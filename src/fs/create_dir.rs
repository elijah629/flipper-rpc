//! FsCreateDir module

use std::path::Path;

use crate::transport::Transport;
use crate::{
    error::{Error, Result},
    proto::{self},
    rpc::req::Request,
    transport::TransportRaw,
};

/// CreateDir traits for flipper filesystem
pub trait FsCreateDir {
    /// Creates a directory at a path.
    fn fs_create_dir(&mut self, path: impl AsRef<Path>) -> Result<()>;
}

impl<T> FsCreateDir for T
where
    T: TransportRaw<proto::Main, proto::Main, Err = Error> + std::fmt::Debug,
{
    #[doc(alias = "fs_mkdir")]
    fn fs_create_dir(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let path = path
            .as_ref()
            .to_str()
            .ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "Path is not UTF-8")
            })?
            .to_string();

        self.send_and_receive(Request::StorageMkdir(path))?;

        Ok(())
    }
}
