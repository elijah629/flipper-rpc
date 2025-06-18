//! CommandStatus mapped into a user-friendly error type. This excludes CommandStatus::Ok as these
//! Error types are meant to be used with results, and an Ok is either Ok(()) or Ok(some data to
//! return)

use thiserror::Error;

#[derive(Error, Debug)]
/// Generic error type for all RPC errors
pub enum Error {
    #[error("command: {0}")]
    /// Command error
    CommandError(#[from] CommandError),
    #[error("storage: {0}")]
    /// Storage error
    StorageError(#[from] StorageError),
    #[error("application: {0}")]
    /// Application error
    ApplicationError(#[from] ApplicationError),
    #[error("virtual display: {0}")]
    /// Virtual Display error
    VirtualDisplayError(#[from] VirtualDisplayError),
    #[error("gpio: {0}")]
    /// GPIO error
    GPIOError(#[from] GPIOError),
}

#[derive(Error, Debug)]
/// Command errors
pub enum CommandError {
    /// Unknown error
    #[error("unknown error (ERROR)")]
    Unknown,
    /// Command can't be decoded successfully - command_id in response may be wrong!
    #[error("decode error (ERROR_DECODE)")]
    Decode,
    /// Command successfully decoded, but not implemented (deprecated or not yet implemented)
    #[error("not implemented (ERROR_NOT_IMPLEMENTED)")]
    NotImplemented,
    /// Somebody took global lock, so not all commands are available
    #[error("busy (ERROR_BUSY)")]
    Busy,
    /// Not received has_next == false
    #[error("continuous command interrupted (ERROR_CONTINUOUS_COMMAND_INTERRUPTED)")]
    ContinuousCommandInterrupted,
    /// Not provided (or provided invalid) crucial parameters to perform RPC
    #[error("invalid RPC parameters (ERROR_INVALID_PARAMETERS)")]
    InvalidParameters,
}

#[derive(Error, Debug)]
/// Storage errors
pub enum StorageError {
    /// FS not ready
    #[error("filesystem is not ready for use (ERROR_STORAGE_NOT_READY)")]
    NotReady,
    /// File/Dir already exist
    #[error("file/dir already exists (ERROR_STORAGE_EXIST)")]
    AlreadyExists,
    /// File/Dir does not exist
    #[error("file/dir not found (ERROR_STORAGE_NOT_EXIST)")]
    NotFound,
    /// Invalid API parameter
    #[error("invalid storage parameter (ERROR_STORAGE_INVALID_PARAMETER)")]
    InvalidParameter,
    /// Access denied
    #[error("permission denied (ERROR_STORAGE_DENIED)")]
    PermissionDenied,
    /// Invalid name/path
    #[error("invalid name/path (ERROR_STORAGE_INVALID_NAME)")]
    InvalidName,
    /// Internal error
    #[error("internal error (ERROR_STORAGE_INTERNAL)")]
    Internal,
    /// Function is not implemented
    #[error("not implemented (ERROR_STORAGE_NOT_IMPLEMENTED)")]
    NotImplemented,
    /// File/Dir already opened
    #[error("already open (ERROR_STORAGE_ALREADY_OPEN)")]
    AlreadyOpen,
    /// Directory, you're going to remove is not empty
    #[error("directory not empty (ERROR_STORAGE_DIR_NOT_EMPTY)")]
    DirectoryNotEmpty,
}

#[derive(Error, Debug)]
/// Application errors
pub enum ApplicationError {
    /// Can't start app - internal error
    #[error("unable to start app - internal (ERROR_APP_CANT_START)")]
    CannotStart,

    /// Another app is already running
    #[error("system locked, another app is already running (ERROR_APP_SYSTEM_LOCKED)")]
    SystemLocked,

    /// App is not running or doesn't support RPC commands
    #[error("rpc is unavailable for app (ERROR_APP_NOT_RUNNING)")]
    RpcUnavailable,

    /// Command execution error
    #[error("command execution error (ERROR_APP_CMD_ERROR)")]
    CommandExecution,
}

#[derive(Error, Debug)]
/// Virtual Display errors
pub enum VirtualDisplayError {
    /// Virtual Display session can't be started twice
    #[error("session already started (ERROR_VIRTUAL_DISPLAY_ALREADY_STARTED)")]
    AreadyStarted,
    /// Virtual Display session can't be stopped when it's not started
    #[error("session not started (ERROR_VIRTUAL_DISPLAY_NOT_STARTED)")]
    NotStarted,
}

#[derive(Error, Debug)]
/// GPIO errors
pub enum GPIOError {
    /// Incorrect pin mode
    #[error("incorrect pin mode (ERROR_GPIO_MODE_INCORRECT)")]
    IncorrectMode,
    /// Unknown pin mode
    #[error("unknown pin mode (ERROR_GPIO_UNKNOWN_PIN_MODE)")]
    UnknownMode,
}
