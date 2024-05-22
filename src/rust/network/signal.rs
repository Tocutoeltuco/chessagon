use serde::{de::DeserializeOwned, Deserialize, Serialize};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::{window, Request, RequestInit, RequestMode, Response};
use web_time::{Duration, Instant, SystemTime};

use crate::utils::wait_until;

use super::p2p::IceCandidate;

#[derive(Serialize, Deserialize)]
pub enum Signal {
    // Sets the peer's SDP
    SetSDP(String),
    // Adds an ICE candidate, as offered by the peer
    AddCandidate(IceCandidate),
    // Joins a room
    JoinRoom(String),
    // When to attempt peer connection
    ConnectAt(SystemTime),
    // When to poll next
    NextPoll(SystemTime),
}

#[derive(Serialize, Deserialize)]
struct IdentResponse {
    token: String,
}

async fn exec<T>(url: &str, auth: Option<&str>, signals: Option<Vec<Signal>>) -> Result<T, JsValue>
where
    T: DeserializeOwned,
{
    let mut opts = RequestInit::new();
    opts.method("POST");
    opts.mode(RequestMode::Cors);

    if let Some(ref signals) = signals {
        opts.body(Some(&JsValue::from_str(
            &serde_json::ser::to_string(signals).unwrap(),
        )));
    }

    let req = Request::new_with_str_and_init(url, &opts)?;
    if let Some(token) = auth {
        req.headers().set("Authorization", token)?;
    }
    if signals.is_some() {
        req.headers().set("Content-Type", "application/json")?;
    }

    let window = window().unwrap();
    let promise = window.fetch_with_request(&req);
    let resp = JsFuture::from(promise).await?;
    let resp: Response = resp.into();
    let body = JsFuture::from(resp.text()?).await?;
    let body = body.as_string().unwrap();

    Ok(serde_json::de::from_str(&body).unwrap())
}

async fn ident(url: &str) -> Result<String, JsValue> {
    // Generates a token
    let resp: IdentResponse = exec(&format!("{}/ident", url), None, None).await?;
    Ok(resp.token)
}

async fn poll(url: &str, auth: &str, updates: Vec<Signal>) -> Result<Vec<Signal>, JsValue> {
    // Sends & receives updates
    exec(&format!("{}/poll", url), Some(auth), Some(updates)).await
}

pub struct SignalClient {
    url: String,
    token: Option<String>,
    pub room: String,
    pub peer_sdp: Option<String>,
    pub peer_ice: Vec<IceCandidate>,
    pub connect_at: Option<Instant>,
    signal_queue: Vec<Signal>,
    next_poll: Instant,
}

impl SignalClient {
    pub fn new(url: &str) -> Self {
        SignalClient {
            url: url.to_string(),
            token: None,
            room: "".to_string(),
            peer_sdp: None,
            peer_ice: vec![],
            connect_at: None,
            signal_queue: vec![],
            next_poll: Instant::now(),
        }
    }

    fn handle_signals(&mut self, signals: Vec<Signal>) {
        for signal in signals {
            match signal {
                Signal::JoinRoom(r) => {
                    self.room = r;
                }
                Signal::SetSDP(s) => {
                    self.peer_sdp = Some(s.to_string());
                }
                Signal::AddCandidate(c) => {
                    self.peer_ice.push(c);
                }
                Signal::ConnectAt(d) => {
                    let d = d.duration_since(SystemTime::now()).unwrap_or_default();
                    self.connect_at = Some(Instant::now() + d);
                }
                Signal::NextPoll(d) => {
                    let d = d.duration_since(SystemTime::now()).unwrap_or_default();
                    self.next_poll = Instant::now() + d;
                }
            };
        }
    }

    pub async fn wait_for_poll(&mut self) {
        wait_until(self.next_poll).await;
        // Safeguard: default delay
        self.next_poll = Instant::now() + Duration::from_secs(10);
    }

    pub async fn poll(&mut self) -> Result<(), JsValue> {
        // Ident if not already
        let token = match self.token.as_ref() {
            Some(t) => t,
            None => {
                let token = ident(&self.url).await?;
                self.token = Some(token);

                self.token.as_ref().unwrap()
            }
        };

        // Move all values from queue
        let signals: Vec<_> = self.signal_queue.drain(0..).collect();
        let signals = poll(&self.url, token, signals).await?;
        self.handle_signals(signals);

        Ok(())
    }

    pub fn send_join_room(&mut self, code: String) {
        self.signal_queue.push(Signal::JoinRoom(code));
    }

    pub fn send_sdp(&mut self, sdp: String) {
        self.signal_queue.push(Signal::SetSDP(sdp));
    }

    pub fn send_ice(&mut self, mut ice: Vec<IceCandidate>) {
        self.signal_queue
            .extend(ice.drain(0..).map(Signal::AddCandidate));
    }
}
