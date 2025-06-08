use std::io;

use flipper_rpc::{
    rpc::RpcRequest,
    transport::{
        Transport,
        serial::{list_flipper_ports, rpc::SerialRpcTransport},
    },
};

fn main() -> std::io::Result<()> {
    let ports = list_flipper_ports()?;

    let port = &ports[0].port_name;

    let mut cli = SerialRpcTransport::new(port.to_string())?;

    let response = cli.send_and_receive(RpcRequest::SystemPlayAudiovisualAlert)?;

    assert!(response.is_none());

    Ok(())
}
