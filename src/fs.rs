//! Helpers for working with the flipper's filesystem through RPC.

/// Path to the external storage root (e.g., SD card).
pub const EXTERNAL_STORAGE: &str = "/ext";

/// Path to the internal flash storage.
pub const INTERNAL_FLASH: &str = "/int";

/// Path to the infrared database stored on external storage.
pub const DB_INFARED: &str = "/ext/infared";

/// Path to the iButton database stored on external storage.
pub const DB_IBUTTON: &str = "/ext/ibutton";

/// Path to the LF RFID database stored on external storage.
pub const DB_LFRFID: &str = "/ext/lfrfid";

/// Path to the BadUSB database stored on external storage.
pub const DB_BADUSB: &str = "/ext/basusb";

/// Path to the Sub-GHz database stored on external storage.
pub const DB_SUBGHZ: &str = "/ext/subghz";

/// Path to the NFC database stored on external storage.
pub const DB_NFC: &str = "/ext/nfc";

/// Path to the update directory on external storage.
pub const UPDATE_DIR: &str = "/ext/update";

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

#[cfg(feature = "fs-tar-extract")]
pub mod tar;
#[cfg(feature = "fs-tar-extract")]
pub use tar::FsTarExtract;

pub mod helpers;

// 512 WORKS! See https://github.com/liamhays/flipwire/blob/a6b2aee31aad39a0322cfbda3d19fa43851189e0/src/protobuf_codec.rs#L21 if further correction is needed
pub(crate) const CHUNK_SIZE: usize = 512;
