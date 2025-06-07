# `flipper-rpc` - Serial RPC control for the flipper zero

> Finally! A rust library to control a flipper zero through ProtoBuf RPC
> commands!

`flipper-rpc` is a library for sending and receiving RPC messages to and from a
flipper zero.

## Features

- Automatic flipper detection
- Full flipper-zero/protobuf support
- Serial RPC

### Tentative

- Better user experience when using the API (initial planned version is in
  `src/rpc.rs`, it is not imported or used)
- Built-in file transfer code

### Planned

- Bluetooth, maybe?

## Usage

## RPC Documentation

After god knows how long of searching for some damn documentation on this shitty
API, and finding nothing. I decided to make my own!

### Connecting

First, you need to make a serial connection. You can use any of your favorite
libraries to do this, for this project I chose `serialport`. It's a very simple
API and just requires the port of the device and a baud rate to connect.

Speaking of baud rate, I don't think it matters. I have tested many different
baud rates when connecting to the flipper, and nothing changes, it works each
time. This could be because the serial connection is a `CDC-ACM`, which is a
"fake" serial connection over USB.

After connecting, you have to "drain" the buffer until you reach the string
`>:`, otherwise known as the shell prompt. You must do this as when you read, it
starts from the start of the buffer, not where you last wrote. Draining is very
simple, you just read a bunch of chunks and check if the string is inside the
concatenation of them all. (source code is in src/reader\_utils.rs)

Then, after draining you must start an RPC session.

The serial connection starts in an ASCII-based text command line. To enter RPC,
write the data: `start_rpc_session\r` (note: writing `\r\n` does not work for
some reason)

Then you must drain until `\n` since the flipper will send `\n` when it has
processed that command.

Then, you can finally start sending RPC commands.

### Sending

Sending a command is quite simple, you first encode the command you want to
send, and then you send it (haha).

The flipper expects all commands to be length-delimeted (fancy way of saying the
first byte must be the length of the data after it):

The length is **NOT** a normal byte representing the length, it **MUST** be
`Varint` encoded.

Example raw request data for sending a `PingRequest` with the data:
`[1, 2, 3, 4]`:

`[8, 42, 6, 10, 4, 1, 2, 3, 4]`

The first byte is the length, in this example 8 encoded into Varint is just 8.

### Receiving

To read a response, you must first read the length of the response.

The length is varint encoded, which has a maximum length of 10 bytes. The slow
way to do this is to read byte by byte until the MSB (most significant bit) of a
byte is `1`, varints end when the MSB is 1

The fast way I developed is too long to explain here, but you can read the many
comments about it in `src/cli.rs` in the `read_rpc_proto` function. In short, it
uses exactly 2 syscalls and stack buffers instead of a max of 11 and heap
buffers.

After you read the varint, you then decode it. After decoding, you read N bytes
and decode that with the protobuf decoder.

And the byte response from the flipper for the previously mentioned ping request
is:

`[8, 50, 6, 10, 4, 1, 2, 3, 4]`

## Why?

This project was made for another project I am making:
[`flippy`](https://github.com/elijah629/flippy), a command line tool to update
firmware and databases from remote sources on the flipper zero. Since there was
no existing way to communicate with it, I split it's RPC code into this library.

(shameless promo: please star :star: it, I would greatly appreciate it)
