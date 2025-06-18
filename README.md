# `flipper-rpc` – Serial RPC Control for the Flipper Zero

[![crates.io](https://img.shields.io/crates/v/flipper-rpc.svg)](https://crates.io/crates/flipper-rpc)

> _Finally!_ A Rust library to control a Flipper Zero through ProtoBuf RPC
> commands.

`flipper-rpc` is a Rust library for sending and receiving RPC messages to and
from a Flipper Zero over a serial connection.

---

## ✨ Features

- Tracing compatible
- 🔍 Automatic Flipper detection
- 🧠 Full
  [flipperzero-protobuf](https://github.com/flipperdevices/flipperzero-protobuf)
  support
- 🔌 Serial-based RPC interface

### 🚧 Tentative

- Ergonomic, user-friendly API (see early draft in [`src/rpc.rs`](src/rpc.rs))
- Built-in file transfer support

### 🧪 Planned

- Bluetooth support (maybe 🤞)

---

## 📦 Installation

Run this command

```sh
cargo add flipper-rpc
```

Or add this to your `Cargo.toml`:

```toml
[dependencies]
flipper-rpc = "0.3.0"  # Replace with the latest version from crates.io
```

## 🚀 Usage

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
   - Fast way: See the optimized logic in [`src/cli.rs`](src/cli.rs),
     `read_rpc_proto()`.

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

- [flipwire](https://github.com/liamhays/flipwire)
- [flipperzero_protobuf_py](https://github.com/flipperdevices/flipperzero_protobuf_py)
