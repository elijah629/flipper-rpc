# `flipper-rpc` – Serial RPC Control for the Flipper Zero

![Crates.io Version](https://img.shields.io/crates/v/flipper-rpc)
![Crates.io License](https://img.shields.io/crates/l/flipper-rpc)
![docs.rs](https://img.shields.io/docsrs/flipper-rpc)
![Crates.io MSRV](https://img.shields.io/crates/msrv/flipper-rpc)
![Crates.io License](https://img.shields.io/crates/l/flipper-rpc)
![Crates.io Downloads (recent)](https://img.shields.io/crates/dr/flipper-rpc)
![GitHub Repo stars](https://img.shields.io/github/stars/elijah629/flipper-rpc)

> _Finally!_ A Rust library to control a Flipper Zero through RPC commands.

`flipper-rpc` is a Rust library for sending and receiving RPC messages to and
from a Flipper Zero over a serial connection.

---

## ✨ Features

- `tokio-tracing` compatible
- Automatic Flipper detection
- Full
  [flipperzero-protobuf](https://github.com/flipperdevices/flipperzero-protobuf)
  support
- Serial-based RPC interface
- Full filesystem support
- Ergonomic, user-friendly API so you don't have to use raw protobuf messages.

### 🚧 Tentative

- Bluetooth support (maybe 🤞)

### 🧪 Planned

**Nothing for now...** Add an issue if you want something cool added!

---

## Features

| Feature                      | Description                                             |
| ---------------------------- | ------------------------------------------------------- |
| `default`                    | `minimal`                                               |
| `full`                       | Enables \*-all                                          |
| `minimal`                    | Just `proto`                                            |
| `proto`                      | Protobuf encoding/decoding                              |
| `easy-rpc`                   | High‑level RPC helper API                               |
| `fs-all`                     | Enables all filesystem operations                       |
| `fs-read`                    | Read files from Flipper Zero                            |
| `fs-write`                   | Write files to Flipper Zero                             |
| `fs-readdir`                 | List directory contents                                 |
| `fs-remove`                  | Remove files or directories                             |
| `fs-createdir`               | Create directories on device                            |
| `transport-all`              | All available transport mechanisms                      |
| `transport-any`              | Base transport support                                  |
| `transport-serial`           | Serial‑port communication                               |
| `transport-serial-optimized` | Optimized serial transport with a better varint decoder |
| `tracing`                    | Enable logging via `tokio-tracing`                      |

## 📦 Installation

> [!IMPORTANT]
> Please decide on features, and don't just use the `full` feature since you are
> lazy. Actually read the table above and enable what you need as it will
> significantly decrease compile time.

Run this command

```sh
cargo add flipper-rpc --features <FEATURES>
```

Or add this to your `Cargo.toml`:

```toml
[dependencies]
flipper-rpc = { features = [], version = "0.9.0" } # Replace with the latest version from crates.io
```

## 🚀 Usage

### Playing an alert

```rust
let ports = list_flipper_ports()?;
let port = &ports[0].port_name;

let mut cli = SerialRpcTransport::new(port.to_string())?;

let response = cli.send_and_receive(Request::SystemPlayAudiovisualAlert)?;
```

---

## 📚 Flipper Zero RPC Protocol Documentation

After far too much searching for usable documentation on this API (and finding
_nothing_), I decided to write my own. Enjoy!

> [!NOTE]
> Read the [source code](src/rpc) for information about actual communication,
> the following information is only for serial transport (**BLE COMING SOON**)

---

### 🔌 Connecting

1. **Make a serial connection.** Use your preferred serial library. This crate
   uses [`serialport`](https://docs.rs/serialport), which is simple and only
   requires the port path and a baud rate.

2. **Baud rate... Apparently doesn't matter** Serial over USB (`CDC-ACM`)
   abstracts baud rate away, it must be reasonable and capable by the hardware
   and software.

3. **Drain the buffer.** Keep reading until you see the shell prompt string
   `">: "`. This clears old buffer content, since reads begin from the buffer
   start — not your last write.

4. **Enter RPC mode.** Write the string: `start_rpc_session\r`
   > **Note:** `\r\n` does **not** work here. I do not know why.

5. **Drain again.** Read until you receive `\n`, which indicates that the
   Flipper has accepted the command.

---

### 📤 Sending RPC Commands

1. **Encode the request using protobuf.**
2. **Prefix the message with its length** (encoded as a **Varint**, not a
   regular byte).

> 🔢 Varint is a variable-length integer encoding. The Flipper expects this as
> the first byte(s) of your message.

#### Example: Sending `PingRequest` with `[1, 2, 3, 4]`

Raw request bytes:

```text
[8, 42, 6, 10, 4, 1, 2, 3, 4]
```

- `8` is the Varint-encoded length of the rest of the message.

---

### 📥 Receiving RPC Responses

1. **Read the length prefix** The length is a Varint and can be up to **10
   bytes**.

   - Slow way: Read byte-by-byte until the MSB is `1`.
   - Fast way: See my optimized logic in
     [`src/transport/serial/rpc.rs`](src/transport/serial/rpc.rs),
     `read_raw(...)` as it is too long to mention here.

2. **Read and decode** Once the length is known, read that many bytes, then
   decode with your protobuf deserializer.

#### Example response for the ping

```text
[8, 50, 6, 10, 4, 1, 2, 3, 4]
```

---

## 🤔 Why This Exists

This crate was built to support another project I'm working on: 👉
[`flippy`](https://github.com/elijah629/flippy) – a CLI tool for updating
firmware and databases on the Flipper Zero from remote sources.

Since there was no existing way to speak RPC to the Flipper in Rust, I split
that functionality into this standalone library.

> 💫 _Shameless plug:_ If you find this useful,
> [give it a star](https://github.com/elijah629/flippy) ⭐ — I'd appreciate it!

## Credits

This project would not be possible without two amazing repos:

- [flipwire](https://github.com/liamhays/flipwire). Gave amazing examples in
  source code that helped me implement the codec even better.
- [flipperzero_protobuf_py](https://github.com/flipperdevices/flipperzero_protobuf_py)
  More source-code examples, although in python, this library demonstrates the
  serial API very well and exactly how to use it and handle it properly.
