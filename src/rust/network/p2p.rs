use std::{cell::RefCell, rc::Rc};

use js_sys::{Array, Reflect, Uint8Array};
use serde::Serialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    Event, MessageEvent, RtcConfiguration, RtcDataChannel, RtcDataChannelEvent, RtcDataChannelInit,
    RtcDataChannelType, RtcIceCandidate, RtcIceCandidateInit, RtcPeerConnection,
    RtcPeerConnectionIceEvent, RtcSdpType, RtcSessionDescriptionInit,
};

use super::buffer::Buffer;

// Sets an event on a target and forgets about the closure
// (leaves it up to JS' GC to drop the object)
macro_rules! set_event {
    ($target: expr, $handler: expr, $setter: ident) => {
        let handler = Closure::wrap($handler);
        $target.$setter(Some(handler.as_ref().unchecked_ref()));
        handler.forget();
    };
}

pub type IceCandidate = (String, Option<String>, Option<u16>);

fn default_ice_servers() -> JsValue {
    #[derive(Serialize)]
    struct IceServerConfig {
        urls: Vec<String>,
        username: String,
        credential: String,
    }

    serde_wasm_bindgen::to_value(&[
        IceServerConfig {
            urls: vec![
                "stun:stun1.l.google.com:19302".to_owned(),
                "stun:global.stun.twilio.com:3478".to_owned(),
            ],
            username: "".to_owned(),
            credential: "".to_owned(),
        },
        IceServerConfig {
            urls: vec![
                "turn:openrelay.metered.ca:80".to_owned(),
                "turn:openrelay.metered.ca:443".to_owned(),
            ],
            username: "openrelayproject".to_owned(),
            credential: "openrelayproject".to_owned(),
        },
    ])
    .unwrap()
}

fn ser_candidate(candidate: &RtcIceCandidate) -> IceCandidate {
    (
        candidate.candidate(),
        candidate.sdp_mid(),
        candidate.sdp_m_line_index(),
    )
}

fn de_candidate(candidate: &IceCandidate) -> RtcIceCandidate {
    let mut init = RtcIceCandidateInit::new(&candidate.0);
    init.sdp_mid(candidate.1.as_deref());
    init.sdp_m_line_index(candidate.2);
    RtcIceCandidate::new(&init).expect("couldn't deserialize ice candidate")
}

#[derive(Clone, Debug)]
pub struct Connection {
    conn: RtcPeerConnection,
    channel: RtcDataChannel,
    ice_rx: Rc<RefCell<Option<futures_channel::mpsc::UnboundedReceiver<Option<RtcIceCandidate>>>>>,
    open: Rc<RefCell<bool>>,
}

impl Connection {
    pub fn new() -> Self {
        let mut conf = RtcConfiguration::new();
        conf.ice_servers(&default_ice_servers());
        let conn = RtcPeerConnection::new_with_configuration(&conf)
            .expect("can't create RtcPeerConnection");

        // Not attaching this handler causes firefox to disconnect.
        let handler: Box<dyn FnMut(_)> = Box::new(move |_event: JsValue| {});
        set_event!(conn, handler, set_oniceconnectionstatechange);

        let mut conf = RtcDataChannelInit::new();
        conf.id(0);
        conf.negotiated(true);
        let channel = conn.create_data_channel_with_data_channel_dict("chessagon", &conf);
        channel.set_binary_type(RtcDataChannelType::Arraybuffer);

        Connection {
            conn,
            channel,
            ice_rx: Default::default(),
            open: Default::default(),
        }
    }

    pub fn close(&self) {
        *self.open.borrow_mut() = false;
        self.channel.set_onopen(None);
        self.channel.set_onmessage(None);
        self.channel.set_onclose(None);
        self.channel.close();
        self.conn.close();
    }

    pub fn send(&self, packet: Buffer) {
        // TODO: Handle error on channel.send
        let packet: Vec<u8> = packet.into();
        let _ = self.channel.send_with_u8_array(packet.as_slice());
    }

    pub fn set_onopen(&self, mut handler: Box<dyn FnMut()>) {
        let open = self.open.clone();
        let handler: Box<dyn FnMut(_)> = Box::new(move |_event: RtcDataChannelEvent| {
            *open.borrow_mut() = false;
            handler();
        });
        set_event!(self.channel, handler, set_onopen);
    }

    pub fn set_onmessage(&self, mut handler: Box<dyn FnMut(Buffer)>) {
        let handler: Box<dyn FnMut(_)> = Box::new(move |event: MessageEvent| {
            let data = Uint8Array::new(&event.data()).to_vec();
            handler(data.into());
        });
        set_event!(self.channel, handler, set_onmessage);
    }

    pub fn set_onclose(&self, mut handler: Box<dyn FnMut()>) {
        let open = self.open.clone();
        let handler: Box<dyn FnMut(_)> = Box::new(move |_event: Event| {
            *open.borrow_mut() = false;
            handler();
        });
        set_event!(self.channel, handler, set_onclose);
    }

    pub fn is_open(&self) -> bool {
        *self.open.borrow()
    }

    fn listen_ice_candidates(&self) {
        // Prepares a listener for onicecandidate events, and
        // sends them to a mpsc channel
        let (tx, rx) = futures_channel::mpsc::unbounded();
        *self.ice_rx.borrow_mut() = Some(rx);
        let handler: Box<dyn FnMut(_)> = Box::new(move |event: RtcPeerConnectionIceEvent| {
            // This will send None when no more candidates are available
            let candidate = event.candidate().filter(|c| !c.candidate().is_empty());
            tx.unbounded_send(candidate).unwrap();
        });
        set_event!(self.conn, handler, set_onicecandidate);
    }

    pub fn poll_ice_candidates(&self) -> Vec<IceCandidate> {
        let mut ice_rx = self.ice_rx.borrow_mut();
        let rx = match ice_rx.as_mut() {
            Some(rx) => rx,
            None => return vec![],
        };

        let mut candidates = vec![];
        while let Ok(opt) = rx.try_next() {
            match opt.flatten() {
                Some(ice) => candidates.push(ser_candidate(&ice)),
                None => {
                    // Push end of list
                    candidates.push(("".to_owned(), Some("".to_owned()), Some(0)));
                    ice_rx.take();
                    break;
                }
            };
        }

        candidates
    }

    async fn create_sdp(&self, ty: RtcSdpType) -> Result<String, JsValue> {
        let promise = if ty == RtcSdpType::Offer {
            self.conn.create_offer()
        } else {
            self.conn.create_answer()
        };

        let object = JsFuture::from(promise).await?;
        let sdp = Reflect::get(&object, &JsValue::from_str("sdp"))?
            .as_string()
            .unwrap();

        Ok(sdp)
    }

    pub async fn add_ice_candidates(
        &self,
        offer_candidates: Vec<IceCandidate>,
    ) -> Result<(), JsValue> {
        let offer_candidates: Vec<_> = offer_candidates.iter().map(de_candidate).collect();

        let promises = Array::new();
        for cand in offer_candidates {
            let promise = self
                .conn
                .add_ice_candidate_with_opt_rtc_ice_candidate(Some(&cand));
            promises.push(&JsValue::from(promise));
        }
        JsFuture::from(js_sys::Promise::all(&promises)).await?;

        Ok(())
    }

    pub async fn set_remote(&self, ty: RtcSdpType, sdp: String) -> Result<(), JsValue> {
        let mut desc = RtcSessionDescriptionInit::new(ty);
        desc.sdp(&sdp);
        JsFuture::from(self.conn.set_remote_description(&desc)).await?;

        Ok(())
    }

    pub async fn create_answer(&self, offer: String) -> Result<String, JsValue> {
        self.set_remote(RtcSdpType::Offer, offer).await?;
        let answer = self.create_sdp(RtcSdpType::Answer).await?;
        Ok(answer)
    }

    pub async fn prepare(&self, ty: RtcSdpType, sdp: Option<String>) -> Result<String, JsValue> {
        self.listen_ice_candidates();

        // Create SDP if not given
        let sdp = match sdp {
            Some(sdp) => sdp,
            None => self.create_sdp(ty).await?,
        };
        // Set connection's description
        // This initiates ICE candidate gathering
        let mut desc = RtcSessionDescriptionInit::new(ty);
        desc.sdp(&sdp);
        JsFuture::from(self.conn.set_local_description(&desc)).await?;

        Ok(sdp)
    }
}
