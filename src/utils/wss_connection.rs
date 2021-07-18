use hyper::Uri;
use hyper::service::Service;
use tokio_tungstenite::WebSocketStream;

use crate::common::{ApiConfig, ApiPath};
use crate::error::Result;
use crate::utils::{HttpsWssConnector, TlsStream};

pub(crate) async fn connect_wss(api_config: &ApiConfig) -> Result<WebSocketStream<TlsStream>> {
    let mut connector = HttpsWssConnector::new(api_config)?;
    let uri = Uri::builder()
        .scheme("wss")
        .authority(api_config.authority.clone())
        .path_and_query(ApiPath::ProvisioningSocket.get_path())
        .build()?;

    let stream = connector.call(uri.clone()).await?;

    Ok(tokio_tungstenite::client_async(uri, stream).await?.0)
}
