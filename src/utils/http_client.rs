use http::Uri;
use http::uri::{Authority, Scheme};
use reqwest::header::{HeaderMap, USER_AGENT, HeaderValue};
use reqwest::{Client, Response, Method};
use serde::Serialize;

use crate::common::{ApiConfig, ApiPath};
use crate::error::{Error, Result};

pub(crate) struct HttpClient {
    client: Client,
    username: String,
    password: String,
    authority: Authority,
}

pub(crate) enum Body<'a, T: Serialize> {
    Empty,
    Json(&'a T),
}

impl Body<'_, ()> {
    pub(crate) fn empty() -> Self {
        Self::Empty
    }
}

impl HttpClient {
    pub(crate) fn new(username: &str, password: &str, api_config: &ApiConfig) -> Result<Self> {
        let mut default_headers = HeaderMap::new();
        default_headers.insert(
            USER_AGENT,
            HeaderValue::from_str(&api_config.user_agent)
                .expect("User agent contains allowed charset."),
        );

        let client = Client::builder()
            .default_headers(default_headers)
            .trust_dns(true)
            .https_only(true)
            .tls_built_in_root_certs(false)
            .add_root_certificate(reqwest::Certificate::from_pem(&api_config.cert_bytes)?)
            .build()?;

        Ok(Self {
            client,
            username: username.to_string(),
            password: password.to_string(),
            authority: api_config.authority.clone(),
        })
    }

    pub async fn send<T: Serialize>(
        &self,
        method: Method,
        path: ApiPath<'_>,
        body: Body<'_, T>,
    ) -> Result<Response> {
        let uri = Uri::builder()
            .scheme(Scheme::HTTPS)
            .authority(self.authority.clone())
            .path_and_query(path.get_path())
            .build()
            .expect("URI should be valid.");

        let mut req_builder = self
            .client
            .request(method, "") // TODO
            .basic_auth(&self.username, Some(&self.password));

        if let Body::Json(data) = body {
            req_builder = req_builder.json(&data);
        }
    
        let resp = req_builder.send().await?;

        if resp.status().is_success() {
            Ok(resp)
        } else if resp.status().as_u16() == 499 {
            Err(Error::DeprecatedHttpError(
                resp.text().await?,
            ))
        } else {
            Err(Error::HttpError(
                resp.status(),
                resp.text().await?,
            ))
        }
    }
}
