//! Test helpers. A small HTTP stub that stands in for the Slack API, so that
//! tests never call the real Slack and need no extra dependencies.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::client::{Client, ClientOptions};

pub const TEST_CLIENT_ID: &str = "test-client-id";
pub const TEST_CLIENT_SECRET: &str = "test-client-secret";

#[derive(Debug, Clone)]
pub struct RecordedRequest {
    pub method: String,
    /// Path and query, for example `/users.list?limit=200`.
    pub target: String,
    headers: Vec<(String, String)>,
    pub body: String,
}

impl RecordedRequest {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }

    /// Decodes an `application/x-www-form-urlencoded` body.
    pub fn form(&self) -> HashMap<String, String> {
        self.body
            .split('&')
            .filter(|pair| !pair.is_empty())
            .map(|pair| {
                let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
                (url_decode(key), url_decode(value))
            })
            .collect()
    }
}

fn url_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        match bytes[index] {
            b'+' => decoded.push(b' '),
            b'%' if index + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[index + 1..index + 3]).unwrap();
                decoded.push(u8::from_str_radix(hex, 16).unwrap());
                index += 2;
            }
            byte => decoded.push(byte),
        }
        index += 1;
    }

    String::from_utf8(decoded).unwrap()
}

/// Answers every request with the same status and body, and records the
/// requests it got.
pub struct StubServer {
    pub url: String,
    requests: Arc<Mutex<Vec<RecordedRequest>>>,
}

impl StubServer {
    pub fn json(body: &str) -> Self {
        Self::start(200, "application/json", body)
    }

    pub fn start(status: u16, content_type: &str, body: &str) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind stub server");
        let url = format!("http://{}", listener.local_addr().unwrap());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let recorded = Arc::clone(&requests);
        let response = format!(
            "HTTP/1.1 {status} STUB\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );

        thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };

                if let Some(request) = read_request(&stream) {
                    recorded.lock().unwrap().push(request);
                }

                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
            }
        });

        Self { url, requests }
    }

    pub fn requests(&self) -> Vec<RecordedRequest> {
        self.requests.lock().unwrap().clone()
    }

    pub fn single_request(&self) -> RecordedRequest {
        let requests = self.requests();
        assert_eq!(requests.len(), 1, "expected exactly one request");
        requests.into_iter().next().unwrap()
    }

    pub fn client(&self) -> Client {
        client_for(&self.url)
    }
}

pub fn client_for(url: &str) -> Client {
    Client::new(ClientOptions {
        url: url.to_owned(),
        client_id: TEST_CLIENT_ID.to_owned(),
        client_secret: TEST_CLIENT_SECRET.to_owned(),
    })
}

/// A URL where nothing listens, to simulate a Slack outage.
pub fn unreachable_url() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    drop(listener);
    url
}

fn read_request(stream: &TcpStream) -> Option<RecordedRequest> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;

    let mut parts = line.split_whitespace();
    let method = parts.next()?.to_owned();
    let target = parts.next()?.to_owned();
    let mut headers = Vec::new();

    loop {
        let mut header = String::new();
        reader.read_line(&mut header).ok()?;
        let header = header.trim_end();

        if header.is_empty() {
            break;
        }

        if let Some((key, value)) = header.split_once(':') {
            headers.push((key.trim().to_owned(), value.trim().to_owned()));
        }
    }

    let length = headers
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case("content-length"))
        .and_then(|(_, value)| value.parse::<usize>().ok())
        .unwrap_or(0);
    let mut body = vec![0; length];
    reader.read_exact(&mut body).ok()?;

    Some(RecordedRequest {
        method,
        target,
        headers,
        body: String::from_utf8(body).ok()?,
    })
}
