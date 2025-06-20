//! FsRead module

use std::borrow::Cow;
use std::path::Path;

use crate::rpc::res::{ReadFile, Response};
use crate::transport::Transport;
use crate::{
    error::{Error, Result},
    proto,
    rpc::req::Request,
    transport::TransportRaw,
};

/// Read traits for flipper filesystem
pub trait FsRead {
    /// Reads a file on the flipper zero from src
    fn fs_read(&mut self, path: impl AsRef<Path>) -> Result<Cow<'static, [u8]>>;

    /// Reads to a string
    fn fs_read_to_string(&mut self, path: impl AsRef<Path>) -> Result<Cow<'static, str>> {
        let bytes = self.fs_read(path)?;

        match bytes {
            Cow::Borrowed(bytes) => std::str::from_utf8(bytes)
                .map(Cow::Borrowed)
                .map_err(|e| e.to_string()),

            Cow::Owned(bytes) => String::from_utf8(bytes)
                .map(Cow::Owned)
                .map_err(|e| e.to_string()),
        }
        .map_err(|error_text| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, error_text).into()
        })
    }

    /// Like [`fs_read_to_string`] but replaces non-utf8 chars with a replacement character
    fn fs_read_to_string_lossy(&mut self, path: impl AsRef<Path>) -> Result<Cow<'static, str>> {
        // Like String::from_utf8_lossy but operates on owned values
        #[inline(always)]
        fn string_from_utf8_lossy(buf: Vec<u8>) -> String {
            match String::from_utf8_lossy(&buf) {
                // buf contained non-utf8 chars than have been patched
                Cow::Owned(s) => s,
                // SAFETY: if Borrowed then the buf only contains utf8 chars,
                // we do this instead of .into_owned() to avoid copying the input buf
                Cow::Borrowed(_) => unsafe { String::from_utf8_unchecked(buf) },
            }
        }

        let bytes = self.fs_read(path)?;
        match bytes {
            Cow::Borrowed(bytes) => Ok(String::from_utf8_lossy(bytes)),
            Cow::Owned(bytes) => Ok(Cow::Owned(string_from_utf8_lossy(bytes))),
        }
    }
}

impl<T> FsRead for T
where
    T: TransportRaw<proto::Main, proto::Main, Err = Error> + std::fmt::Debug,
{
    fn fs_read(&mut self, path: impl AsRef<Path>) -> Result<Cow<'static, [u8]>> {
        let path = path
            .as_ref()
            .to_str()
            .ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "Path is not UTF-8")
            })?
            .to_string();

        let response = self.send_and_receive(Request::StorageRead(path))?;

        match response {
            Response::Empty | Response::StorageRead(Some(ReadFile::Dir)) => {
                Err(std::io::Error::new(
                    std::io::ErrorKind::IsADirectory,
                    "Cannot read a directory, use fs_read_dir instead.",
                )
                .into())
            }
            Response::StorageRead(None) => {
                Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to read file").into())
            }
            Response::StorageRead(Some(ReadFile::File(data, size, actual))) => {
                #[cfg(feature = "fs-read-verify")]
                {
                    let length = data.len() as u32;

                    if length != size {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("File size mismatch: expected {size}, only got {length}"),
                        )
                        .into());
                    }

                    let expected = format!("{:x}", md5::compute(&data));

                    if actual != expected {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!(
                                "MD5 hash mismatch: expected {:02x?}, got {:02x?}",
                                expected, actual
                            ),
                        )
                        .into());
                    }
                }

                Ok(data)
            }
            _ => Err(std::io::Error::new(std::io::ErrorKind::Other, "Invalid response").into()),
        }
    }
}
