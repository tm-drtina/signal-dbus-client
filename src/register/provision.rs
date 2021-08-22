use std::sync::Arc;
use std::time::Duration;

use futures_util::{Sink, SinkExt, Stream, StreamExt};
use tokio::sync::Mutex;
use tungstenite::{Error as TungError, Message as TungMessage};

use signal_provisioning_api::{ProvisionMessage, ProvisioningSocket, ProvisioningState};

use crate::common::ApiConfig;
use crate::error::{Error, Result};
use crate::utils::{connect_wss, qrcode_image};

async fn process_stream<Si, St>(sink: Arc<Mutex<Si>>, mut stream: St) -> Result<ProvisionMessage>
where
    Si: Sink<TungMessage, Error = TungError> + Unpin,
    St: Stream<Item = std::result::Result<TungMessage, TungError>> + Unpin,
{
    let mut socket = ProvisioningSocket::new();
    loop {
        match stream.next().await {
            Some(Ok(msg)) => match msg {
                TungMessage::Ping(data) => {
                    sink.lock()
                        .await
                        .start_send_unpin(TungMessage::Pong(data))?;
                }
                TungMessage::Pong(_) | TungMessage::Text(_) | TungMessage::Close(_) => {}
                TungMessage::Binary(data) => {
                    if let Some(request_id) = socket.process_message(data).unwrap() {
                        let ack = ProvisioningSocket::acknowledge(request_id);
                        let ack = ProvisioningSocket::serialize(ack);
                        sink.lock()
                            .await
                            .start_send_unpin(TungMessage::Binary(ack))?;
                        if let ProvisioningState::UuidReceived(uuid) = &socket.state {
                            let url = uuid.provisioning_url(socket.ephemeral_key_pair.public_key);
                            let image = qrcode_image(&url, true);

                            eprintln!("Scan the QR code with your app:\n{}\n{}", url, image);
                        } else if let ProvisioningState::Provisioned(_) = &socket.state {
                            sink.lock().await.close().await?;
                        }
                    }
                }
            },
            Some(Err(err)) => {
                match err {
                    TungError::ConnectionClosed => break,
                    TungError::Io(error)
                        if error.kind() == std::io::ErrorKind::ConnectionAborted =>
                    {
                        // Signal servers doesn't close the connection properly, but terminate
                        // Handle abort as close
                        break;
                    }
                    _ => return Err(err.into()),
                }
            }
            None => break,
        }
    }

    if let ProvisioningState::Provisioned(msg) = socket.state {
        Ok(msg)
    } else {
        Err(Error::ProvisioningFailed)
    }
}

pub(super) async fn get_provision_message(api_config: &ApiConfig) -> Result<ProvisionMessage> {
    let (sink, stream) = connect_wss(api_config).await?.split();
    let sink = Arc::new(Mutex::new(sink));
    let clone = Arc::clone(&sink);
    let jh = tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(15)).await;
            let hb = ProvisioningSocket::hb_request();
            let hb = ProvisioningSocket::serialize(hb);
            clone
                .lock()
                .await
                .start_send_unpin(TungMessage::Binary(hb))
                .unwrap();
        }
    });
    let result = process_stream(sink, stream).await;

    jh.abort();
    jh.await
        .expect_err("We are cancelling the job, so error is expected");

    result
}
