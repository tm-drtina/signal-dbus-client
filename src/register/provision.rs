use tungstenite::{Error as TungError, Message as TungMessage};

use signal_provisioning_api::{ApiConfig, ProvisionMessage, ProvisioningSocket, ProvisioningState};

use crate::error::{Error, Result};
use crate::utils::{connect_ws, qrcode_image};

pub(super) fn get_provision_message() -> Result<ProvisionMessage> {
    let api_config = ApiConfig::default();
    let mut socket = ProvisioningSocket::new();
    let mut ws = connect_ws(&api_config);
    let mut last_hb = std::time::Instant::now();
    loop {
        let now = std::time::Instant::now();
        if now.duration_since(last_hb).as_secs() > 15 {
            let hb = ProvisioningSocket::hb_request();
            let hb = ProvisioningSocket::serialize(hb);
            ws.write_message(TungMessage::Binary(hb)).unwrap();
            last_hb = now;
        }
        match ws.read_message() {
            Ok(msg) => match msg {
                TungMessage::Ping(data) => {
                    ws.write_message(TungMessage::Pong(data)).unwrap();
                }
                TungMessage::Pong(_) | TungMessage::Text(_) | TungMessage::Close(_) => {}
                TungMessage::Binary(data) => {
                    if let Some(request_id) = socket.process_message(data).unwrap() {
                        let ack = ProvisioningSocket::acknowledge(request_id);
                        let ack = ProvisioningSocket::serialize(ack);
                        ws.write_message(TungMessage::Binary(ack)).unwrap();
                        if let ProvisioningState::UuidReceived(uuid) = &socket.state {
                            let url = uuid.provisioning_url(socket.ephemeral_key_pair.public_key);
                            let image = qrcode_image(&url, true);

                            eprintln!("Scan the QR code with your app:\n{}\n{}", url, image);
                        } else if let ProvisioningState::Provisioned(_) = &socket.state {
                            ws.close(None).unwrap();
                        }
                    }
                }
            },
            Err(TungError::ConnectionClosed) => {
                break;
            }
            Err(TungError::Io(error)) if error.kind() == std::io::ErrorKind::ConnectionAborted => {
                // Signal servers doesn't close the connection properly, but terminate
                // Handle abort as close
                break;
            }
            Err(TungError::Io(error)) if error.kind() == std::io::ErrorKind::WouldBlock => {
                // eprintln!("spin...")
            }
            Err(err) => return Err(err.into()),
        }
    }

    if let ProvisioningState::Provisioned(msg) = socket.state {
        Ok(msg)
    } else {
        Err(Error::ProvisioningFailed)
    }
}
