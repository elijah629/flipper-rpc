use flipper_rpc::{
    error::Result,
    storage::FlipperFs,
    transport::serial::{list_flipper_ports, rpc::SerialRpcTransport},
};

fn main() -> Result<()> {
    let ports = list_flipper_ports()?;

    let port = &ports[0].port_name;

    let mut cli = SerialRpcTransport::new(port)?;

    cli.fs_mkdir("/ext/path")?;
    cli.fs_write("/ext/path/file.txt", "Hello, what is this?")?;

    println!("{:?}", cli.fs_readdir("/ext/path")?);

    let out = cli.fs_read("/ext/path")?;
    println!("{out:?}");
    cli.fs_rm("/ext/path")?;

    Ok(())
}
