use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use hyper::client::connect::dns::GaiResolver;
use hyper::client::HttpConnector;
use hyper::service::Service;
use hyper::Uri;
use tokio_rustls::rustls::ClientConfig;
use tokio_rustls::webpki::DNSNameRef;
use tokio_rustls::TlsConnector;

use crate::common::ApiConfig;
use crate::error::Error;
use crate::error::Result;

use super::TlsStream;

pub struct HttpsWssConnector {
    http: HttpConnector<GaiResolver>,
    tls_config: Arc<ClientConfig>,
}

impl HttpsWssConnector {
    pub fn new(api_config: &ApiConfig) -> Result<Self> {
        let mut http = HttpConnector::new();
        http.enforce_http(false);

        let tls_config = Arc::new(api_config.rustls_config()?);

        Ok(Self { http, tls_config })
    }
}

impl Clone for HttpsWssConnector {
    fn clone(&self) -> Self {
        Self {
            http: self.http.clone(),
            tls_config: Arc::clone(&self.tls_config),
        }
    }
}

fn connection_error<S: Into<String>>(reason: S) -> Error {
    Error::ConnectionError(reason.into())
}

impl Service<Uri> for HttpsWssConnector {
    type Response = TlsStream;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<()>> {
        match self.http.poll_ready(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(_)) => Poll::Ready(Err(connection_error("unknown"))),
            Poll::Pending => Poll::Pending,
        }
    }

    fn call(&mut self, dst: Uri) -> Self::Future {
        if !matches!(dst.scheme_str(), Some("https") | Some("wss")) {
            return Box::pin(async { Err(connection_error("Not an HTTPS/WSS Uri")) });
        }

        let cfg = self.tls_config.clone();
        let hostname = dst.host().unwrap_or_default().to_string();
        let connecting_future = self.http.call(dst);

        let f = async move {
            let tcp = connecting_future
                .await
                .map_err(|_| connection_error("unknown"))?;
            let connector = TlsConnector::from(cfg);
            let dnsname = DNSNameRef::try_from_ascii_str(&hostname)
                .map_err(|_| connection_error("invalid dnsname"))?;
            let tls = connector
                .connect(dnsname, tcp)
                .await
                .map_err(|e| connection_error(e.to_string()))?;
            Ok(tls.into())
        };
        Box::pin(f)
    }
}
