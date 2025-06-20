//! FsReadDir module

use std::path::Path;

use crate::fs::helpers::os_str_to_str;
use crate::rpc::res::ReadDirItem;
use crate::transport::Transport;
use crate::transport::serial::rpc::CommandIndex;
use crate::{
    error::{Error, Result},
    proto::{self, storage::ListRequest},
    rpc::req::Request,
    transport::TransportRaw,
};

/// ReadDir traits for flipper filesystem
pub trait FsReadDir {
    /// Lists the files in a directory at path
    fn fs_read_dir(&mut self, path: impl AsRef<Path>) -> Result<impl Iterator<Item = ReadDirItem>>;
}

impl<T> FsReadDir for T
where
    T: TransportRaw<proto::Main, proto::Main, Err = Error> + CommandIndex + std::fmt::Debug,
{
    fn fs_read_dir(&mut self, path: impl AsRef<Path>) -> Result<impl Iterator<Item = ReadDirItem>> {
        let path = os_str_to_str(path.as_ref().as_os_str())?.to_string();

        let req = Request::StorageList(ListRequest {
            path,
            include_md5: true, // useful to have
            filter_max_size: 0,
        });

        let res = self.send_and_receive(req)?;
        let res: Vec<ReadDirItem> = res.try_into()?;

        Ok(res.into_iter())
    }
}
