use std::net::TcpStream;
use std::sync::Arc;
use std::time::Duration;

use rustls::{ClientConfig, ClientSession, StreamOwned};
use tungstenite::{client, WebSocket};
use webpki::DNSNameRef;

use signal_provisioning_api::ApiConfig;

pub(crate) type TlsStream = StreamOwned<ClientSession, TcpStream>;

pub(crate) fn connect(api_config: &ApiConfig, read_timeout: Option<Duration>) -> TlsStream {
    let host_port = format!("{}:{}", api_config.host, api_config.port);

    let socket = TcpStream::connect(host_port).unwrap();
    socket
        .set_read_timeout(read_timeout)
        .unwrap();

    let mut config = ClientConfig::new();
    config
        .root_store
        .add_pem_file(&mut &api_config.cert_bytes[..])
        .unwrap();

    let config = Arc::new(config);
    let dns_name = DNSNameRef::try_from_ascii_str(&api_config.host).unwrap();
    let session = ClientSession::new(&config, dns_name);
    StreamOwned::new(session, socket)
}

pub(crate) fn connect_ws(api_config: &ApiConfig) -> WebSocket<TlsStream> {
    let ws_addr = format!(
        "{}{}:{}{}",
        api_config.socket_protocol,
        api_config.host,
        api_config.port,
        api_config.provisioning_socket_path
    );
    let stream = connect(api_config, Some(Duration::from_secs(2)));
    client(ws_addr, stream).unwrap().0
}
