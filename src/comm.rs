use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use std::time::{Duration, Instant};
use serde_json::Value;

use crate::data::Method;

pub enum UserAction {
    SendRequest {
        url: String,
        method: Method,
        headers: Vec<(String, String)>,
        body: Option<String>,
    }
}

pub enum UiMessage {
    Response {
        status_line: String,
        raw: String,
        parsed: Option<Value>,
        elapsed_ms: u128,
    }
}

pub fn spawn_comm() -> (Sender<UserAction>, Receiver<UiMessage>) {
    let (tx, rx_bg) = channel::<UserAction>();
    let (tx_ui, rx_ui) = channel::<UiMessage>();

    thread::spawn(move || {
        while let Ok(msg) = rx_bg.recv() {
            match msg {
                UserAction::SendRequest { url, method, headers, body } => {
                    let start = Instant::now();
                    let client = reqwest::blocking::Client::builder()
                        .timeout(Duration::from_secs(60))
                        .danger_accept_invalid_certs(true)
                        .build();

                    let response_msg = if let Ok(client) = client {
                        let mut req = match method {
                            Method::GET => client.get(&url),
                            Method::POST => client.post(&url),
                            Method::PUT => client.put(&url),
                            Method::PATCH => client.patch(&url),
                            Method::DELETE => client.delete(&url),
                        };
                        for (k, v) in headers {
                            req = req.header(k, v);
                        }
                        if let Some(b) = body {
                            req = req.body(b);
                        }
                        match req.send() {
                            Ok(resp) => {
                                let status = resp.status();
                                let version = resp.version();
                                let status_line = format!("{:?} {}", version, status);
                                match resp.text() {
                                    Ok(text) => {
                                        let parsed = serde_json::from_str::<Value>(&text).ok();
                                        UiMessage::Response {
                                            status_line,
                                            raw: text,
                                            parsed,
                                            elapsed_ms: start.elapsed().as_millis(),
                                        }
                                    }
                                    Err(e) => UiMessage::Response {
                                        status_line,
                                        raw: format!("<read error: {}>", e),
                                        parsed: None,
                                        elapsed_ms: start.elapsed().as_millis(),
                                    },
                                }
                            }
                            Err(e) => UiMessage::Response {
                                status_line: "<request error>".to_string(),
                                raw: format!("{}", e),
                                parsed: None,
                                elapsed_ms: start.elapsed().as_millis(),
                            },
                        }
                    } else {
                        UiMessage::Response {
                            status_line: "<client build error>".to_string(),
                            raw: "".into(),
                            parsed: None,
                            elapsed_ms: start.elapsed().as_millis(),
                        }
                    };
                    let _ = tx_ui.send(response_msg);
                }
            }
        }
    });

    (tx, rx_ui)
}
