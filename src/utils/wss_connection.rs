use std::sync::Arc;

use http::Uri;
use tokio::net::TcpStream;
use tokio_tungstenite::{WebSocketStream, MaybeTlsStream};

use crate::common::{ApiConfig, ApiPath};
use crate::error::Result;

pub(crate) async fn connect_wss(api_config: &ApiConfig) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>> {
    let uri = Uri::builder()
        .scheme("wss")
        .authority(api_config.authority.clone())
        .path_and_query(ApiPath::ProvisioningSocket.get_path())
        .build()?;

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
