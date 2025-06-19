//! FsReadDir module

use std::path::Path;

use crate::transport::Transport;
use crate::{
    error::{Error, Result},
    proto::{
        self,
        storage::{File, ListRequest},
    },
    rpc::req::Request,
    transport::TransportRaw,
};

/// ReadDir traits for flipper filesystem
pub trait FsReadDir {
    /// Lists the files in a directory at path
    fn fs_read_dir(&mut self, path: impl AsRef<Path>) -> Result<Box<dyn Iterator<Item = File>>>;
}

impl<T> FsReadDir for T
where
    T: TransportRaw<proto::Main, proto::Main, Err = Error> + std::fmt::Debug,
{
    fn fs_read_dir(&mut self, path: impl AsRef<Path>) -> Result<Box<dyn Iterator<Item = File>>> {
        let path = path
            .as_ref()
            .to_str()
            .ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "Path is not UTF-8")
            })?
            .to_string();

        let req = Request::StorageList(ListRequest {
            path,
            // not including MD5 here, as it is not practical to check against it when we don't
            // have the file data
            include_md5: false,
            filter_max_size: 0,
        });

        let res = self.send_and_receive(req)?;
        let res: Vec<File> = res.try_into()?;

        Ok(Box::new(res.into_iter()))
    }
}
