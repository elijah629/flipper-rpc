//! Helpers for working with the flipper's filesystem through RPC.

#[cfg(feature = "fs-createdir")]
pub mod create_dir;
#[cfg(feature = "fs-createdir")]
pub use create_dir::FsCreateDir;

#[cfg(feature = "fs-read")]
pub mod read;
#[cfg(feature = "fs-read")]
pub use read::FsRead;

#[cfg(feature = "fs-readdir")]
pub mod read_dir;
#[cfg(feature = "fs-readdir")]
pub use read_dir::FsReadDir;

#[cfg(feature = "fs-remove")]
pub mod remove;

#[cfg(feature = "fs-remove")]
pub use remove::FsRemove;

#[cfg(feature = "fs-write")]
pub mod write;
#[cfg(feature = "fs-write")]
pub use write::FsWrite;

#[cfg(feature = "fs-metadata")]
pub mod metadata;
#[cfg(feature = "fs-metadata")]
pub use metadata::FsMetadata;

#[cfg(feature = "fs-md5")]
pub mod md5;
#[cfg(feature = "fs-md5")]
pub use md5::FsMd5;

pub mod helpers;

pub(crate) const CHUNK_SIZE: usize = 512;
