use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use web_time::{Duration, SystemTime, UNIX_EPOCH};

use crate::{
    auth::{Auth, AuthInfo, GRACE_PERIOD},
    db::{BucketInfo, Data, Metadata},
    poll::{Killable, Signal},
};

const FIRST_POLL: u64 = 1;
const POLL: u64 = 10;
const CONNECT: u64 = 5;
const FAST_POLL: u64 = 1;

pub type Room = Data<RoomData, RoomMetadata, RoomInfo>;

pub struct RoomInfo {}
impl BucketInfo for RoomInfo {
    const PREFIX: &'static str = "room";
    const KEY_LENGTH: u8 = 6;
}

#[derive(Serialize, Deserialize, Default)]
pub struct Peer {
    sent_sdp: bool,
    ice_done: bool,
    queue: Vec<Signal>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct RoomData {
    sent_connect: bool,
    offer: Peer,
    answer: Peer,
}

pub struct PeerMetadata {
    token: String,
    next_poll: SystemTime,
}
impl Default for PeerMetadata {
    fn default() -> Self {
        PeerMetadata {
            token: Default::default(),
            next_poll: SystemTime::now() + Duration::from_secs(FIRST_POLL),
        }
    }
}

#[derive(Default)]
pub struct RoomMetadata {
    offer: PeerMetadata,
    answer: Option<PeerMetadata>,
}

impl Metadata for RoomMetadata {}
impl From<HashMap<String, String>> for RoomMetadata {
    fn from(value: HashMap<String, String>) -> Self {
        let mut offer = None;
        let mut answer = None;

        for (prefix, slot) in [("offer_", &mut offer), ("answer_", &mut answer)] {
            let token = match value.get(&(prefix.to_owned() + "token")) {
                Some(t) => t,
                None => {
                    continue;
                }
            };
            let next_poll = match value.get(&(prefix.to_owned() + "next_poll")) {
                Some(l) => l,
                None => {
                    continue;
                }
            };
            if token.is_empty() || next_poll.is_empty() {
                continue;
            }
            let token = token.to_owned();
            let next_poll = next_poll.parse().unwrap();
            let next_poll = UNIX_EPOCH + Duration::from_secs(next_poll);

            *slot = Some(PeerMetadata { token, next_poll });
        }

        RoomMetadata {
            offer: offer.unwrap(),
            answer,
        }
    }
}
impl From<RoomMetadata> for HashMap<String, String> {
    fn from(value: RoomMetadata) -> Self {
        let mut map = HashMap::new();

        for (prefix, opt) in [("offer_", Some(value.offer)), ("answer_", value.answer)] {
            if let Some(peer) = opt {
                let last = peer
                    .next_poll
                    .duration_since(UNIX_EPOCH)
                    .expect("time travel?")
                    .as_secs();
                map.insert(prefix.to_owned() + "token", peer.token);
                map.insert(prefix.to_owned() + "next_poll", last.to_string());
            } else {
                map.insert(prefix.to_owned() + "token", "".to_owned());
                map.insert(prefix.to_owned() + "next_poll", "".to_owned());
            }
        }

        map
    }
}

impl Room {
    pub fn join_room(&mut self, peer: &mut Auth) -> bool {
        let is_offer = self.meta.offer.token.is_empty();
        let is_guest = self.meta.answer.is_none();

        if !is_offer && !is_guest {
            // Room is full.
            return false;
        }
        let data = self.data.as_mut().expect("invalid state");

        let token;
        let queue;
        if is_offer {
            token = &mut self.meta.offer.token;
            queue = &mut data.offer.queue;
        } else {
            self.meta.answer = Some(Default::default());
            token = &mut self.meta.answer.as_mut().unwrap().token;
            queue = &mut data.answer.queue;
        }

        peer.join_room(&self.key);
        *token = peer.key.clone();
        queue.push(Signal::JoinRoom(self.key.clone()));

        self.modified = true;
        true
    }

    pub fn poll(&mut self, peer: &Auth) {
        let duration = if self.meta.answer.is_some() {
            // Fast polling after both parties are connected
            Duration::from_secs(FAST_POLL)
        } else {
            Duration::from_secs(POLL)
        };
        let next_poll = SystemTime::now() + duration;

        if self.meta.offer.token == peer.key {
            self.meta.offer.next_poll = next_poll;
        } else {
            self.meta.answer.as_mut().expect("invalid state").next_poll = next_poll;
        };

        self.modified = true;
    }

    fn try_set_connect(&mut self) {
        let data = self.data.as_mut().expect("invalid state");

        if data.sent_connect {
            return;
        }
        // Need both SDPs
        if !data.offer.sent_sdp || !data.answer.sent_sdp {
            return;
        }
        // Need at least one ICE list to be done
        if !data.offer.ice_done && !data.answer.ice_done {
            return;
        }

        let offer = self.meta.offer.next_poll;
        let answer = self.meta.answer.as_ref().unwrap().next_poll;
        let connect_at = offer.max(answer) + Duration::from_secs(CONNECT);

        let signal = Signal::ConnectAt(connect_at);
        data.sent_connect = true;
        data.offer.queue.push(signal.clone());
        data.answer.queue.push(signal.clone());

        self.modified = true;
    }

    pub fn send_signal<S>(&mut self, sender: &Auth, signals: S)
    where
        S: IntoIterator<Item = Signal>,
    {
        let data = self.data.as_mut().expect("invalid state");
        let peer;
        let queue;
        if self.meta.offer.token == sender.key {
            peer = &mut data.offer;
            queue = &mut data.answer.queue;
        } else {
            peer = &mut data.answer;
            queue = &mut data.offer.queue;
        }

        for signal in signals.into_iter() {
            if signal.can_send() {
                self.modified = true;
                queue.push(signal.clone());
            }

            match signal {
                Signal::SetSDP(_) => {
                    self.modified = true;
                    peer.sent_sdp = true;
                }
                Signal::AddCandidate(ice) => {
                    if ice.0.is_empty() {
                        self.modified = true;
                        peer.ice_done = true;
                    }
                }
                _ => {}
            };
        }

        self.try_set_connect();
    }

    pub fn pull_signals(&mut self, reader: &Auth) -> Vec<Signal> {
        let data = self.data.as_mut().expect("invalid state");
        let next_poll;
        let queue;
        if self.meta.offer.token == reader.key {
            next_poll = &self.meta.offer.next_poll;
            queue = &mut data.offer.queue;
        } else {
            next_poll = &self.meta.answer.as_ref().unwrap().next_poll;
            queue = &mut data.answer.queue;
        }

        if !queue.is_empty() {
            self.modified = true;
        }

        let mut queue: Vec<_> = std::mem::take(queue);
        queue.push(Signal::NextPoll(*next_poll));
        queue
    }
}

impl Killable for Room {
    fn get_keys_to_kill(&self) -> Vec<String> {
        let mut kill_at = self.meta.offer.next_poll;
        if let Some(ref answer) = self.meta.answer {
            kill_at = kill_at.min(answer.next_poll);
        }
        let kill_at = kill_at + Duration::from_secs(GRACE_PERIOD);

        let mut keys = vec![];
        if SystemTime::now() >= kill_at {
            keys.push(format!("{}:{}", RoomInfo::PREFIX, self.key));
            keys.push(format!("{}:{}", AuthInfo::PREFIX, self.meta.offer.token));

            if let Some(ref answer) = self.meta.answer {
                keys.push(format!("{}:{}", AuthInfo::PREFIX, answer.token));
            }
        }

        keys
    }
}
