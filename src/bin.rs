use flipper_rpc::{
    cli::{self, Cli},
    proto,
};

fn main() {
    let ports = Cli::flipper_ports().unwrap();

    let port = &ports[0].0;

    let mut cli = Cli::new(port.to_string()).unwrap();

    let ping = proto::Main {
        command_id: 0,
        command_status: proto::CommandStatus::Ok.into(),
        has_next: false,
        content: Some(proto::main::Content::SystemPingRequest(
            proto::system::PingRequest {
                data: vec![0xDE, 0xAD, 0xBE, 0xEF],
            },
        )),
    };

    let response = cli.send_read_rpc_proto(ping).unwrap();

    println!("{response:?}");
}
