use futures::{Future, StreamExt};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use web_sys::RtcSdpType;
use web_time::{Duration, SystemTime};

use super::buffer::Buffer;
use super::p2p::Connection;
use super::signal::SignalClient;

const SERVER: &str = "https://signalling.tocu.workers.dev";

#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    fn setTimeout(closure: &Closure<dyn FnMut()>, millis: u32) -> f64;
    fn clearTimeout(token: f64);
}

fn run_in(delay: Duration, closure: &Closure<dyn FnMut()>) -> f64 {
    setTimeout(closure, delay.as_millis() as u32)
}

async fn wait_for(delay: Duration) {
    let (mut tx, mut rx) = futures_channel::mpsc::channel(1);
    let handler: Box<dyn FnMut()> = Box::new(move || {
        tx.try_send(()).unwrap();
    });
    let handler = Closure::wrap(handler);
    run_in(delay, &handler);

    rx.next().await;
}

async fn wait_until(at: SystemTime) {
    let delay = at
        .duration_since(SystemTime::now())
        .unwrap_or(Duration::from_millis(0));
    wait_for(delay).await;
}

async fn wait_for_connect(signal: &mut SignalClient) -> Result<(), JsValue> {
    while signal.connect_at.is_none() {
        wait_for(Duration::from_secs(10)).await;
        signal.poll().await?;
    }

    wait_until(signal.connect_at.unwrap()).await;

    Ok(())
}

async fn wrap<F>(onerror: Option<Box<dyn FnMut(JsValue)>>, fut: F)
where
    F: Future<Output = Result<(), JsValue>> + 'static,
{
    let result = fut.await;

    if let Err(e) = result {
        if let Some(mut handler) = onerror {
            handler(e);
        }
    }
}

fn bind_handler(conn: &Connection, mut handler: Box<dyn FnMut(&Connection)>) -> Box<dyn FnMut()> {
    let conn = conn.clone();
    Box::new(move || handler(&conn))
}

fn bind_handler_1<T>(
    conn: &Connection,
    mut handler: Box<dyn FnMut(&Connection, T)>,
) -> Box<dyn FnMut(T)>
where
    T: 'static,
{
    let conn = conn.clone();
    Box::new(move |value: T| handler(&conn, value))
}

pub struct Connector {
    onopen: Option<Box<dyn FnMut(&Connection)>>,
    onmessage: Option<Box<dyn FnMut(&Connection, Buffer)>>,
    onclose: Option<Box<dyn FnMut()>>,
    onroom: Option<Box<dyn FnMut(String)>>,
    onerror: Option<Box<dyn FnMut(JsValue)>>,
}

impl Connector {
    pub fn new() -> Self {
        Connector {
            onopen: None,
            onmessage: None,
            onclose: None,
            onroom: None,
            onerror: None,
        }
    }

    pub fn set_onopen(&mut self, handler: Box<dyn FnMut(&Connection)>) {
        self.onopen = Some(handler);
    }
    pub fn set_onmessage(&mut self, handler: Box<dyn FnMut(&Connection, Buffer)>) {
        self.onmessage = Some(handler);
    }
    pub fn set_onclose(&mut self, handler: Box<dyn FnMut()>) {
        self.onclose = Some(handler);
    }
    pub fn set_onroom(&mut self, handler: Box<dyn FnMut(String)>) {
        self.onroom = Some(handler);
    }
    pub fn set_onerror(&mut self, handler: Box<dyn FnMut(JsValue)>) {
        self.onerror = Some(handler);
    }

    fn new_connection(&mut self) -> Connection {
        let conn = Connection::new();

        if let Some(handler) = self.onopen.take() {
            conn.set_onopen(bind_handler(&conn, handler));
        }
        if let Some(handler) = self.onmessage.take() {
            conn.set_onmessage(bind_handler_1(&conn, handler));
        }
        if let Some(handler) = self.onclose.take() {
            conn.set_onclose(handler);
        }

        conn
    }

    async fn run_as_host(mut self) -> Result<(), JsValue> {
        let conn = self.new_connection();
        let (sdp, candidates) = conn.prepare(RtcSdpType::Offer, None, vec![]).await?;

        let mut signal = SignalClient::new_as_host(SERVER, sdp, candidates).await?;
        if let Some(mut handler) = self.onroom {
            handler(signal.room.clone());
        }

        wait_for_connect(&mut signal).await?;
        conn.set_remote(RtcSdpType::Answer, signal.peer_sdp.clone().unwrap())
            .await?;

        wait_for(Duration::from_secs(5)).await;
        signal.poll().await?;
        conn.add_ice_candidates(signal.peer_ice).await?;

        Ok(())
    }

    async fn run_as_guest(mut self, code: String) -> Result<(), JsValue> {
        let conn = self.new_connection();

        let mut signal = SignalClient::new_as_guest(SERVER, code).await?;
        let sdp = conn.create_answer(signal.peer_sdp.clone().unwrap()).await?;

        signal.send_sdp(sdp.clone()).await?;
        if let Some(mut handler) = self.onroom {
            handler(signal.room.clone());
        }

        wait_for_connect(&mut signal).await?;

        let (_, candidates) = conn
            .prepare(RtcSdpType::Answer, Some(sdp), signal.peer_ice.clone())
            .await?;
        signal.send_ice(candidates).await?;

        Ok(())
    }

    pub fn start_as_host(mut self) {
        spawn_local(wrap(self.onerror.take(), self.run_as_host()));
    }

    pub fn start_as_guest(mut self, code: String) {
        spawn_local(wrap(self.onerror.take(), self.run_as_guest(code)));
    }
}
