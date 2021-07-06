use std::io::{Read, Write};
use std::time::Duration;

use serde::de::DeserializeOwned;
use serde::Serialize;
use tungstenite::http::{header::HeaderName, HeaderValue, Request, Response, StatusCode, Version};

use signal_provisioning_api::ApiConfig;

use crate::error::{Error, Result};

use super::connect;

/// We know that \r cannot be used alone and must be used as \r\n as delimiter
/// Using \r in status line is prohibited
/// Status line: https://www.w3.org/Protocols/rfc2616/rfc2616-sec6.html#sec6.1
/// Newlines are allowed in header for compatibility reasons. We will ignore them
fn read_until_rn<'a>(buf: &mut &'a [u8]) -> &'a [u8] {
    let index = buf.iter().position(|byte| *byte == b'\r').unwrap();
    assert_eq!(buf[index + 1], b'\n');

    let ret = &buf[0..index];
    *buf = &buf[index + 2..];

    ret
}

fn parse_status_line(status_line: &[u8]) -> (Version, StatusCode) {
    let mut chunks = status_line.split(|b| *b == b' ');
    let version = chunks.next().unwrap();
    let status = chunks.next().unwrap();
    let _reason = chunks.next().unwrap();

    assert_eq!(std::str::from_utf8(version).unwrap().trim(), "HTTP/1.1");

    let version = Version::HTTP_11;
    let status = StatusCode::from_bytes(status).unwrap();

    (version, status)
}

fn parse_header_line(header_line: &[u8]) -> (HeaderName, HeaderValue) {
    let index = header_line.iter().position(|b| *b == b':').unwrap();
    (
        HeaderName::from_bytes(&header_line[0..index]).unwrap(),
        HeaderValue::from_bytes(&header_line[index + 1..]).unwrap(),
    )
}

pub(crate) fn send_json<T: Serialize, U: DeserializeOwned>(
    api_config: &ApiConfig,
    mut req: Request<T>,
) -> Result<Response<U>> {
    let mut stream = connect(api_config, Some(Duration::from_secs(30)));
    let body = serde_json::to_vec(req.body()).unwrap();

    req.headers_mut()
        .entry("Content-Type")
        .or_insert(HeaderValue::from_static("application/json"));
    req.headers_mut()
        .entry("Content-Length")
        .or_insert(HeaderValue::from_str(&body.len().to_string()).unwrap());

    write!(
        stream,
        "\
        {method} {path} {version:?}\r\n\
        Host: {host}\r\n",
        method = req.method(),
        path = req.uri().path(),
        version = req.version(),
        host = req.uri().host().unwrap(),
    )
    .unwrap();

    req.headers().iter().for_each(|(name, value)| {
        write!(stream, "{}: {}\r\n", name, value.to_str().unwrap()).unwrap();
    });

    write!(stream, "\r\n").unwrap();
    stream.write_all(&body[..]).unwrap();
    write!(stream, "\r\n").unwrap();

    let mut buf = Vec::<u8>::with_capacity(0);
    buf.resize(1024, 0u8);
    let mut total_read = 0usize;
    loop {
        let read = stream.read(&mut buf[total_read..]).unwrap();
        total_read += read;
        if total_read < buf.capacity() {
            break;
        }
        buf.resize(buf.len() * 2, 0u8);
    }

    let buf = &mut &buf[..total_read];
    let mut resp_builder = Response::builder();

    let status_line = loop {
        let status_line = read_until_rn(buf);
        if !status_line.is_empty() {
            break status_line;
        }
    };
    let (version, status) = parse_status_line(status_line);
    resp_builder = resp_builder.version(version).status(status);
    loop {
        let header_line = read_until_rn(buf);
        if header_line.is_empty() {
            break;
        }
        let (name, value) = parse_header_line(header_line);
        resp_builder = resp_builder.header(name, value);
    }

    if status.is_success() {
        let parsed_resp_body = serde_json::from_reader(buf).unwrap();
        let response = resp_builder.body(parsed_resp_body).unwrap();
        Ok(response)
    } else {
        let body = std::str::from_utf8(buf).unwrap().to_string();
        let response = resp_builder.body(body).unwrap();
        Err(Error::HttpError(response))
    }
}
