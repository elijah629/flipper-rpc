use std::sync::mpsc::channel;

use flipper_rpc::{
    error::Result,
    fs::{FsRemove, FsWrite},
    transport::serial::{list_flipper_ports, rpc::SerialRpcTransport},
};

fn main() -> Result<()> {
    let ports = list_flipper_ports()?;

    let port = &ports[0].port_name;

    let mut cli = SerialRpcTransport::new(port)?;

    let (tx, rx) = channel();

    let handle = std::thread::spawn(move || {
        use std::time::Instant;
        let start = Instant::now();
        for (sent, total) in rx {
            println!("[+{:.2?}] Progress: {}/{}", start.elapsed(), sent, total);
        }
    });

    cli.fs_write("/ext/file.txt", [65; 512 * 5], tx)?;

    handle.join().unwrap();

    cli.fs_remove("/ext/file.txt", false)?;

    Ok(())
}
