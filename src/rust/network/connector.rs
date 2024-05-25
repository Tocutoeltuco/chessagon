use futures::Future;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use web_sys::RtcSdpType;

use crate::utils::wait_until;

use super::buffer::Buffer;
use super::p2p::Connection;
use super::signal::SignalClient;

const SERVER: &str = "https://signalling.tocu.workers.dev";

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
    signal: SignalClient,
    onestablishing: Option<Box<dyn FnMut()>>,
    onopen: Option<Box<dyn FnMut(&Connection)>>,
    onmessage: Option<Box<dyn FnMut(&Connection, Buffer)>>,
    onclose: Option<Box<dyn FnMut()>>,
    onroom: Option<Box<dyn FnMut(String)>>,
    onerror: Option<Box<dyn FnMut(JsValue)>>,
}

impl Connector {
    pub fn new() -> Self {
        Connector {
            signal: SignalClient::new(SERVER),
            onestablishing: None,
            onopen: None,
            onmessage: None,
            onclose: None,
            onroom: None,
            onerror: None,
        }
    }

    pub fn set_onestablishing(&mut self, handler: Box<dyn FnMut()>) {
        self.onestablishing = Some(handler);
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

    async fn drain_ice(&mut self, conn: &Connection) -> Result<(), JsValue> {
        conn.add_ice_candidates(self.signal.peer_ice.drain(0..).collect())
            .await?;
        Ok(())
    }

    async fn poll(&mut self, conn: &Connection) -> Result<(), JsValue> {
        self.signal.wait_for_poll().await;
        self.signal.send_ice(conn.poll_ice_candidates());
        self.signal.poll().await?;
        Ok(())
    }

    async fn wait_for_connect(&mut self, conn: &Connection, drain: bool) -> Result<(), JsValue> {
        while self.signal.connect_at.is_none() {
            if !self.signal.can_poll() {
                panic!("can't poll but didn't connect?");
            }

            self.poll(conn).await?;
            if drain {
                self.drain_ice(conn).await?;
            }
        }

        if let Some(ref mut handler) = self.onestablishing {
            handler();
        }
        wait_until(self.signal.connect_at.unwrap()).await;
        Ok(())
    }

    async fn poll_until_done(&mut self, conn: &Connection) -> Result<(), JsValue> {
        while self.signal.can_poll() {
            self.poll(conn).await?;
            self.drain_ice(conn).await?;
        }
        Ok(())
    }

    async fn run_as_host(mut self) -> Result<(), JsValue> {
        let conn = self.new_connection();
        let sdp = conn.prepare(RtcSdpType::Offer, None).await?;
        self.signal.send_sdp(sdp);
        self.poll(&conn).await?;

        if self.signal.room.is_empty() {
            panic!("couldn't create room");
        }

        if let Some(ref mut handler) = self.onroom {
            handler(self.signal.room.clone());
        }

        self.wait_for_connect(&conn, true).await?;
        conn.set_remote(RtcSdpType::Answer, self.signal.peer_sdp.clone().unwrap())
            .await?;

        self.poll_until_done(&conn).await?;
        Ok(())
    }

    async fn run_as_guest(mut self, code: String) -> Result<(), JsValue> {
        let conn = self.new_connection();
        self.signal.send_join_room(code);
        self.poll(&conn).await?;

        if self.signal.room.is_empty() || self.signal.peer_sdp.is_none() {
            panic!("couldn't join room");
        }

        let sdp = conn
            .create_answer(self.signal.peer_sdp.clone().unwrap())
            .await?;
        self.signal.send_sdp(sdp.clone());
        self.poll(&conn).await?;

        if let Some(ref mut handler) = self.onroom {
            handler(self.signal.room.clone());
        }

        self.wait_for_connect(&conn, false).await?;
        conn.prepare(RtcSdpType::Answer, Some(sdp)).await?;
        self.drain_ice(&conn).await?;

        self.poll_until_done(&conn).await?;
        Ok(())
    }

    pub fn start_as_host(mut self) {
        spawn_local(wrap(self.onerror.take(), self.run_as_host()));
    }

    pub fn start_as_guest(mut self, code: String) {
        spawn_local(wrap(self.onerror.take(), self.run_as_guest(code)));
    }
}
