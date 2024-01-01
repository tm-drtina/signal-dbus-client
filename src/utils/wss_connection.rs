use std::sync::Arc;

use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use crate::common::{ApiConfig, ApiPath};
use crate::error::Result;

pub(crate) async fn connect_wss(
    api_config: &ApiConfig,
) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>> {
    let uri = format!(
        "wss://{}{}",
        api_config.authority,
        ApiPath::ProvisioningSocket.get_path()
    );

    Ok(tokio_tungstenite::connect_async_tls_with_config(
        uri,
        None,
        false,
        Some(tokio_tungstenite::Connector::Rustls(Arc::new(
            api_config.rustls_config()?,
        ))),
    )
    .await?
    .0)
}
