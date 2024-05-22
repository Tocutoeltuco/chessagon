use serde::{de::DeserializeOwned, Deserialize, Serialize};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::{window, Request, RequestInit, RequestMode, Response};
use web_time::SystemTime;

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
    token: String,
    pub room: String,
    pub peer_sdp: Option<String>,
    pub peer_ice: Vec<IceCandidate>,
    pub connect_at: Option<SystemTime>,
}

impl SignalClient {
    async fn new_bare(url: &str) -> Result<Self, JsValue> {
        Ok(SignalClient {
            url: url.to_string(),
            token: ident(url).await?,
            room: "".to_string(),
            peer_sdp: None,
            peer_ice: vec![],
            connect_at: None,
        })
    }

    pub async fn new_as_host(
        url: &str,
        sdp: String,
        mut ice: Vec<IceCandidate>,
    ) -> Result<Self, JsValue> {
        let mut obj = SignalClient::new_bare(url).await?;
        let mut signals = vec![Signal::SetSDP(sdp)];
        signals.extend(ice.drain(0..).map(Signal::AddCandidate));
        obj.poll_with_signals(signals).await?;

        if obj.room.is_empty() {
            panic!("couldn't create room");
        }

        Ok(obj)
    }

    pub async fn new_as_guest(url: &str, room: String) -> Result<Self, JsValue> {
        let mut obj = SignalClient::new_bare(url).await?;
        obj.poll_with_signals(vec![Signal::JoinRoom(room.clone())])
            .await?;

        if obj.room.is_empty() || obj.peer_sdp.is_none() || obj.peer_ice.is_empty() {
            panic!("couldn't join room");
        }

        Ok(obj)
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
                Signal::ConnectAt(a) => {
                    self.connect_at = Some(a);
                }
            };
        }
    }

    async fn poll_with_signals(&mut self, signals: Vec<Signal>) -> Result<(), JsValue> {
        let signals = poll(&self.url, &self.token, signals).await?;
        self.handle_signals(signals);
        Ok(())
    }

    pub async fn poll(&mut self) -> Result<(), JsValue> {
        self.poll_with_signals(vec![]).await
    }

    pub async fn send_sdp(&mut self, sdp: String) -> Result<(), JsValue> {
        self.poll_with_signals(vec![Signal::SetSDP(sdp)]).await
    }

    pub async fn send_ice(&mut self, mut ice: Vec<IceCandidate>) -> Result<(), JsValue> {
        self.poll_with_signals(ice.drain(0..).map(Signal::AddCandidate).collect())
            .await
    }
}
