use serde::{Deserialize, Serialize};
use web_time::SystemTime;
use worker::{console_log, Bucket, Env, Include, Object, Request, Response, Result};

use crate::{
    auth::{Auth, AuthInfo},
    db::BucketInfo,
    room::Room,
};

pub type IceCandidate = (String, Option<String>, Option<u16>);
pub trait Readable: Sized {
    async fn read(obj: &Object) -> Result<Self>;
}
pub trait Killable: Readable {
    fn get_keys_to_kill(&self) -> Vec<String>;
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Signal {
    SetSDP(String),
    AddCandidate(IceCandidate),
    JoinRoom(String),
    ConnectAt(SystemTime),
    NextPoll(SystemTime),
}

impl Signal {
    pub fn can_send(&self) -> bool {
        match self {
            Self::SetSDP(_) => true,
            Self::AddCandidate(_) => true,
            Self::JoinRoom(_) => false,
            Self::ConnectAt(_) => false,
            Self::NextPoll(_) => false,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct IdentResponse {
    token: String,
}

pub async fn ident(env: Env) -> Result<Response> {
    let bucket = env.bucket("rtc")?;
    let auth = Auth::create(&bucket).await?;
    let token = auth.key.clone();
    auth.write(&bucket).await?;
    Response::from_json(&IdentResponse { token })
}

pub async fn poll(mut req: Request, env: Env) -> Result<Response> {
    let token = match req.headers().get("Authorization")? {
        Some(token) => token,
        None => return Response::error("Missing token.", 403),
    };
    let signals = req.json::<Vec<Signal>>().await?;
    if signals
        .iter()
        .filter(|s| !matches!(s, Signal::JoinRoom(_)))
        .any(|s| !s.can_send())
    {
        return Response::error("Invalid signals: can't send.", 400);
    }

    let bucket = env.bucket("rtc")?;
    let mut user = match Auth::load(&bucket, &token).await? {
        Some(user) => user,
        None => return Response::error("Invalid token.", 403),
    };

    let do_poll;
    let mut room = match user.get_room() {
        Some(code) => {
            // User is in room
            do_poll = true;
            let room = match Room::load(&bucket, code).await? {
                Some(room) => room,
                None => return Response::error("Room expired.", 400),
            };

            if room.is_done() {
                return Response::error("Room expired.", 400);
            }

            room
        }
        None => {
            // User is not in room. They're creating or joining.
            do_poll = false;
            let room = match signals.iter().find(|s| matches!(s, Signal::JoinRoom(_))) {
                Some(Signal::JoinRoom(code)) => Room::load(&bucket, code).await?,
                None => Some(Room::create(&bucket).await?),
                Some(_) => return Response::error("server logic error.", 500),
            };
            let mut room = match room {
                Some(room) => room,
                None => return Response::error("Room not found.", 404),
            };

            if !room.join_room(&mut user) {
                return Response::error("Room is full.", 400);
            }
            room
        }
    };

    room.send_signal(&user, signals);
    if do_poll {
        room.poll(&user);
    }
    let signals = room.pull_signals(&user);

    room.write(&bucket).await?;
    user.write(&bucket).await?;

    Response::from_json(&signals)
}

async fn get_keys<D>(obj: &Object) -> Vec<String>
where
    D: Killable,
{
    D::read(obj)
        .await
        .unwrap_or_else(|_| panic!("couldn't read object {}", obj.key()))
        .get_keys_to_kill()
}

pub async fn cleanup(bucket: Bucket) {
    let objects = bucket
        .list()
        .include(vec![Include::CustomMetadata])
        .execute()
        .await
        .expect("couldn't list objects");

    let mut to_delete = vec![];
    for obj in objects.objects().iter() {
        let keys = if obj.key().starts_with(AuthInfo::PREFIX) {
            get_keys::<Auth>(obj).await
        } else {
            get_keys::<Room>(obj).await
        };
        to_delete.extend(keys);
    }

    console_log!("deleting {:?}", to_delete);
    for key in to_delete.iter() {
        bucket.delete(key).await.unwrap();
    }
}
