use std::io::Read;
use std::ops::Deref;

use hyper::body::{Buf, HttpBody};
use hyper::header::{
    HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE, HOST, USER_AGENT,
};
use hyper::http::uri::{Authority, Scheme};
use hyper::{Body, Client, Method, Request, Response, Uri};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::common::{ApiConfig, ApiPath};
use crate::error::{Error, Result};
use crate::utils::HttpsWssConnector;

pub(crate) struct HttpClient {
    client: Client<HttpsWssConnector>,
    default_headers: HeaderMap,
    authority: Authority,
}

pub(crate) struct WrappedResponse(Response<Body>);

impl HttpClient {
    pub(crate) fn new(username: &str, password: &str, api_config: &ApiConfig) -> Result<Self> {
        let connector = HttpsWssConnector::new(api_config)?;
        let client = Client::builder().build(connector);
        let mut default_headers = HeaderMap::new();

        let creds = format!("{}:{}", username, password);
        let auth_value = format!("Basic {}", base64::encode(creds));
        let mut auth_value = HeaderValue::from_str(&auth_value).expect("Base64 chars are allowed.");
        auth_value.set_sensitive(true);
        default_headers.insert(AUTHORIZATION, auth_value);

        default_headers.insert(
            USER_AGENT,
            HeaderValue::from_str(&api_config.user_agent)
                .expect("User agent contains allowed charset."),
        );
        default_headers.insert(
            HOST,
            HeaderValue::from_str(&api_config.authority.host())
                .expect("Host from Authority is valid."),
        );

        Ok(Self {
            client,
            default_headers,
            authority: api_config.authority.clone(),
        })
    }

    async fn send_inner(
        &self,
        method: Method,
        path: ApiPath<'_>,
        body: Body,
    ) -> Result<WrappedResponse> {
        let uri = Uri::builder()
            .scheme(Scheme::HTTPS)
            .authority(self.authority.clone())
            .path_and_query(path.get_path())
            .build()
            .expect("URI should be valid.");

        let mut builder = Request::builder().method(method).uri(uri);
        for (name, value) in &self.default_headers {
            builder = builder.header(name.clone(), value);
        }

        if let Some(size) = body.size_hint().exact() {
            builder = builder.header(
                CONTENT_LENGTH,
                HeaderValue::from_str(&format!("{}", size)).expect("Numbers are always valid"),
            );
            builder = builder.header(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        }

        let req = builder.body(body)?;
        eprintln!("{:?}", req);

        let resp = self.client.request(req).await?;
        if resp.status().is_success() {
            Ok(resp.into())
        } else {
            Err(Error::HttpError(
                resp.status(),
                WrappedResponse(resp).text().await?,
            ))
        }
    }

    pub(crate) async fn send(&self, method: Method, path: ApiPath<'_>) -> Result<WrappedResponse> {
        self.send_inner(method, path, Body::empty()).await
    }

    pub(crate) async fn send_json<S: Serialize>(
        &self,
        method: Method,
        path: ApiPath<'_>,
        body: &S,
    ) -> Result<WrappedResponse> {
        let serialized_body = serde_json::to_vec(body)?;
        self.send_inner(method, path, serialized_body.into()).await
    }
}

impl WrappedResponse {
    pub(crate) async fn bytes(self) -> Result<impl Buf> {
        hyper::body::aggregate(self.0.into_body())
            .await
            .map_err(Into::into)
    }

    pub(crate) async fn json<T: DeserializeOwned>(self) -> Result<T> {
        let bytes = self.bytes().await?;
        serde_json::from_reader(bytes.reader()).map_err(Into::into)
    }

    pub(crate) async fn text(self) -> Result<String> {
        let bytes = self.bytes().await?;
        let mut buf = String::new();
        bytes.reader().read_to_string(&mut buf)?;
        Ok(buf)
    }
}

impl Deref for WrappedResponse {
    type Target = Response<Body>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<WrappedResponse> for Response<Body> {
    fn from(res: WrappedResponse) -> Self {
        res.0
    }
}

impl From<Response<Body>> for WrappedResponse {
    fn from(res: Response<Body>) -> Self {
        Self(res)
    }
}
