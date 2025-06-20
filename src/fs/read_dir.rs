//! FsReadDir module

use std::path::Path;

use crate::rpc::res::ReadDirItem;
use crate::transport::Transport;
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
    T: TransportRaw<proto::Main, proto::Main, Err = Error> + std::fmt::Debug,
{
    fn fs_read_dir(&mut self, path: impl AsRef<Path>) -> Result<impl Iterator<Item = ReadDirItem>> {
        let path = path
            .as_ref()
            .to_str()
            .ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "Path is not UTF-8")
            })?
            .to_string();

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
