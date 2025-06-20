//! FsCreateDir module

use std::path::Path;

use crate::fs::helpers::os_str_to_str;
use crate::transport::Transport;
use crate::transport::serial::rpc::CommandIndex;
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
    T: TransportRaw<proto::Main, proto::Main, Err = Error> + CommandIndex + std::fmt::Debug,
{
    #[doc(alias = "fs_mkdir")]
    fn fs_create_dir(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let path = os_str_to_str(path.as_ref().as_os_str())?.to_string();

        self.send_and_receive(Request::StorageMkdir(path))?;

        Ok(())
    }
}
