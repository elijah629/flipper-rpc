use std::sync::mpsc::channel;

use flipper_rpc::{
    error::Result,
    fs::{FsMetadata, FsRead, FsReadDir, FsRemove, FsWrite},
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

    let data = (0..512 * 10).map(|i| (i / 512) as u8).collect::<Vec<_>>();
    cli.fs_write("/ext/file2.txt", data, tx)?;

    handle.join().unwrap();

    println!("{:?}", cli.fs_metadata("/ext/file2.txt")?);
    println!("{:?}", cli.fs_read("/ext/file2.txt")?.len());
    println!("{:?}", cli.fs_read_dir("/ext/subghz")?.collect::<Vec<_>>());

    cli.fs_remove("/ext/file2.txt", false)?;

    Ok(())
}
