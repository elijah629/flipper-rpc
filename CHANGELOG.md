# Changelog

## 0.6.2

- **logging** Improve logging across the entire application

## 0.6.1

### Fixed

- oopsy i put a println in the code

## 0.6.0

### BREAKING CHANGES

- **fs_read_dir** now asks if it should include the MD5 hash

### Fixed

- **serial** timeout error when reading large rpc messages. Increased timeout to
  10 seconds.

### Added

- **serial** global serial timeout unified and increased to 10 seconds.
- **fs** New md5 function

## 0.5.2

- **rpc** fix the bytewise method of reading. Improper imports

## 0.5.1

- **fs** fix `fs_read_dir`. ReadDir now reads the full chain, it now exhibits
  expected behavior.

## 0.5.0

### BREAKING CHANGES

- **fs**: Remove `fs-read-verify` feature and MD5 verification on read
  operations
- **fs**: Rename `fs-progress-mpsc` to `fs-write-progress-mpsc` and
  `fs-read-progress-mpsc`
- **fs**: Change filesystem read API to handle chunked responses differently
- **rpc**: Modify request API signatures - several methods now take simplified
  parameters
- **transport**: Add `CommandIndex` trait requirement for filesystem operations

### Added

- **fs**: Add `fs-metadata` feature and `FsMetadata` trait for file metadata
  operations
- **fs**: Add `fs-read-metadata` feature for size-aware file reading with
  pre-allocation
- **fs**: Add `fs-read-progress-mpsc` feature for read operation progress
  tracking
- **fs**: Add `fs-progress-mpsc` meta-feature combining read and write progress
  features
- **fs**: Add `helpers` module with `os_str_to_str` utility function
- **transport**: Add `CommandIndex` trait for managing RPC command indices
- **proto**: Add `with_command_id()` and `with_has_next()` builder methods to
  `proto::Main`

### Changed

- **fs**: Rewrite file reading to handle chunked responses properly using
  command index system
- **fs**: Simplify `FsWrite::fs_write()` progress parameter with feature flag
- **fs**: Update all filesystem traits to require `CommandIndex` bound
- **rpc**: Simplify request constructors - `StorageMd5sum`, `StorageRename`,
  `StorageBackupCreate`, `StorageBackupRestore`, `StorageTarExtract` now take
  direct parameters
- **rpc**: Rename `StorageStat` to `StorageMetadata` in request enum
- **rpc**: Change `into_rpc()` to take `command_id` parameter instead of
  `has_next`
- **rpc**: Update response types - `StorageRead` now returns
  `Option<Cow<'static, [u8]>>`, `StorageStat` returns `Option<u32>`
- **rpc**: Modify `ReadDirItem` enum variants to include filenames
- **transport**: Move command index auto-increment logic to easy API layer
- **transport**: Update `Transport` implementation to require `CommandIndex`
  trait

### Removed

- **fs**: Remove `fs-read-verify` feature and associated MD5 hash verification
- **rpc**: Remove `ReadFile` enum (replaced with direct `Cow<[u8]>`)

### Fixed

- **fs**: Fix chunked file reading by properly implementing command index
  tracking
- **transport**: Fix command ID management in multi-part RPC messages
- **transport**: Improve error handling in bytewise reading method

### Refactor

- **fs**: Extract path conversion logic to shared `helpers::os_str_to_str`
  function
- **fs**: Consolidate filesystem operation error handling
- **examples**: Update file example to demonstrate new metadata and read APIs
- **rpc**: Simplify request building by removing nested struct requirements

### Documentation

- **fs**: Add comprehensive comments explaining chunked read/write behavior
- **fs**: Add doc aliases for common filesystem operation names (`fs_rm`,
  `fs_mkdir`)
- **transport**: Document command index behavior and usage patterns
