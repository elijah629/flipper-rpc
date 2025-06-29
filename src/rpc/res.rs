//! Response type. Maps all Content's ending with "Response"

use std::borrow::Cow;

use crate::proto::app::{AppStateResponse, GetErrorResponse, LockStatusResponse};
use crate::proto::desktop::Status;
use crate::proto::gpio::{GetOtgModeResponse, GetPinModeResponse, ReadPinResponse};
use crate::proto::gui::ScreenFrame;
use crate::proto::main::Content;
use crate::proto::property::GetResponse;
use crate::proto::storage::file::FileType;
use crate::proto::storage::{InfoResponse, TimestampResponse};
use crate::proto::system::{
    DeviceInfoResponse, PowerInfoResponse, ProtobufVersionResponse, UpdateResponse,
};
use crate::proto::{self, system::DateTime};

macro_rules! define_into_impl {
    ($enum_name:ident $variant:ident $typ:ty) => {
        impl std::convert::TryInto<$typ> for $enum_name {
            type Error = crate::error::Error;

            fn try_into(self) -> Result<$typ, Self::Error> {
                match self {
                    $enum_name::$variant(x) => Ok(x),
                    x => Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!(
                            "Cannot convert {x:?} into a {}",
                            stringify!($enum_name::$variant)
                        ),
                    )
                    .into()),
                }
            }
        }
    };
    ($enum_name:ident $variant:ident) => {};
}

macro_rules! define_into_enum {
     (
        $(#[$enum_meta:meta])*
        $vis:vis enum $enum_name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident $( ( $typ:ty ) )?
            ),* $(,)?
        }
    ) => {
        $(#[$enum_meta])*
        $vis enum $enum_name {
            $(
                $(#[$variant_meta])*
                #[doc = stringify!($enum_name::$variant)]
                $variant $( ( $typ ) )?,
            )*
        }

        $(
            define_into_impl!($enum_name $variant $( $typ)?);
        )*
    };
}

// bootleg proc-macros but i dont wanna make any
define_into_enum! {
    /// Wrapper around proto::Main tailored for responses. Can be made from a proto::Main by
    /// Into/From
#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum Response {
    Empty,
    Ping(Vec<u8>),
    SystemDeviceInfo(DeviceInfoResponse),
    SystemGetDatetime(Option<DateTime>),
    SystemProtobufVersion(ProtobufVersionResponse),
    SystemUpdate(UpdateResponse),
    SystemPowerInfo(PowerInfoResponse),
    StorageInfo(InfoResponse),
    StorageTimestamp(TimestampResponse),
    StorageStat(Option<u32>),
    StorageList(Vec<ReadDirItem>),
    StorageRead(Option<Cow<'static, [u8]>>),
    StorageMd5sum(String),
    AppLockStatus(LockStatusResponse),
    AppGetError(GetErrorResponse),
    GuiScreenFrame(ScreenFrame),
    GpioGetPinMode(GetPinModeResponse),
    GpioReadPin(ReadPinResponse),
    GpioGetOtgMode(GetOtgModeResponse),
    AppState(AppStateResponse),
    PropertyGet(GetResponse),
    DesktopStatus(Status),
}
}

/// Item read using fs_read_dir / Request::StorageList
#[derive(Debug, PartialEq)]
pub enum ReadDirItem {
    /// Directory + Name
    Dir(String),
    /// Name, File size, MD5 Hash
    File(String, u32, Option<String>),
}

// Only extracts raw content, ignores errors
impl From<proto::Main> for Response {
    fn from(val: proto::Main) -> Self {
        use Response::*;
        let content = val.content;
        match content {
            None | Some(Content::Empty(_)) => Empty,
            Some(x) => match x {
                Content::SystemPingResponse(r) => Ping(r.data),
                Content::SystemDeviceInfoResponse(r) => SystemDeviceInfo(r),
                Content::SystemGetDatetimeResponse(r) => SystemGetDatetime(r.datetime),
                Content::SystemProtobufVersionResponse(r) => SystemProtobufVersion(r),
                Content::SystemUpdateResponse(r) => SystemUpdate(r),
                Content::SystemPowerInfoResponse(r) => SystemPowerInfo(r),
                Content::StorageInfoResponse(r) => StorageInfo(r),
                Content::StorageTimestampResponse(r) => StorageTimestamp(r),
                Content::StorageStatResponse(r) => StorageStat(r.file.map(|x| x.size)),
                Content::StorageListResponse(r) => StorageList(
                    r.file
                        .into_iter()
                        .map(|file| match FileType::try_from(file.r#type).unwrap() {
                            FileType::File => ReadDirItem::File(
                                file.name,
                                file.size,
                                if file.md5sum.is_empty() {
                                    None
                                } else {
                                    Some(file.md5sum)
                                },
                            ),
                            FileType::Dir => ReadDirItem::Dir(file.name),
                        })
                        .collect::<Vec<_>>(),
                ),
                Content::StorageReadResponse(r) => {
                    // Response would have returned an error if the requested path was a dir
                    // As of now, reading does not return any data about the file besides the data.
                    // No name/hash/size etc.
                    StorageRead(r.file.map(|x| match FileType::try_from(x.r#type).unwrap() {
                        FileType::File => x.data.into(),
                        FileType::Dir => unreachable!(),
                    }))
                }
                Content::StorageMd5sumResponse(r) => StorageMd5sum(r.md5sum),
                Content::AppLockStatusResponse(r) => AppLockStatus(r),
                Content::AppGetErrorResponse(r) => AppGetError(r),
                Content::GuiScreenFrame(r) => GuiScreenFrame(r),
                Content::GpioGetPinModeResponse(r) => GpioGetPinMode(r),
                Content::GpioReadPinResponse(r) => GpioReadPin(r),
                Content::GpioGetOtgModeResponse(r) => GpioGetOtgMode(r),
                Content::AppStateResponse(r) => AppState(r),
                Content::PropertyGetResponse(r) => PropertyGet(r),
                Content::DesktopStatus(r) => DesktopStatus(r),

                _ => panic!("Cannot convert {:?} into RpcResponse", x),
            },
        }
    }
}
