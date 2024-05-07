use std::collections::HashMap;

use rand::{rngs::SmallRng, seq::SliceRandom, Rng, SeedableRng};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use web_time::{Duration, SystemTime, UNIX_EPOCH};
use worker::*;

type IceCandidate = (String, Option<String>, Option<u16>);

#[derive(Serialize, Deserialize, Clone)]
enum Signal {
    SetSDP(String),
    AddCandidate(IceCandidate),
    JoinRoom(String),
    ConnectAt(SystemTime),
}

#[derive(Serialize, Deserialize)]
struct Peer {
    sent_sdp: bool,
    queue: Vec<Signal>,
}

#[derive(Serialize, Deserialize)]
struct Room {
    offer: Peer,
    answer: Peer,
}
trait Metadata: From<HashMap<String, String>> + Into<HashMap<String, String>> {}

struct AuthMetadata {
    kill_at: Option<SystemTime>,
}
impl Metadata for AuthMetadata {}
impl From<HashMap<String, String>> for AuthMetadata {
    fn from(value: HashMap<String, String>) -> Self {
        let kill_at = value.get("kill_at").unwrap();
        if kill_at.is_empty() {
            return AuthMetadata { kill_at: None };
        }
        let kill_at = kill_at.parse().unwrap();
        let kill_at = UNIX_EPOCH + Duration::from_secs(kill_at);
        AuthMetadata {
            kill_at: Some(kill_at),
        }
    }
}
impl From<AuthMetadata> for HashMap<String, String> {
    fn from(value: AuthMetadata) -> Self {
        let mut map = HashMap::new();
        let kill_at = match value.kill_at {
            Some(kill_at) => kill_at
                .duration_since(UNIX_EPOCH)
                .expect("time travel?")
                .as_secs()
                .to_string(),
            None => "".to_owned(),
        };
        map.insert("kill_at".to_owned(), kill_at);
        map
    }
}

struct PeerMetadata {
    token: String,
    last_read: SystemTime,
}

struct RoomMetadata {
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
            let last_read = match value.get(&(prefix.to_owned() + "last_read")) {
                Some(l) => l,
                None => {
                    continue;
                }
            };
            if token.is_empty() || last_read.is_empty() {
                continue;
            }
            let token = token.to_owned();
            let last_read = last_read.parse().unwrap();
            let last_read = UNIX_EPOCH + Duration::from_secs(last_read);

            *slot = Some(PeerMetadata { token, last_read });
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
                    .last_read
                    .duration_since(UNIX_EPOCH)
                    .expect("time travel?")
                    .as_secs();
                map.insert(prefix.to_owned() + "token", peer.token);
                map.insert(prefix.to_owned() + "last_read", last.to_string());
            } else {
                map.insert(prefix.to_owned() + "token", "".to_owned());
                map.insert(prefix.to_owned() + "last_read", "".to_owned());
            }
        }

        map
    }
}

#[derive(Serialize, Deserialize)]
struct Auth {
    room: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct IdentResponse {
    token: String,
}

fn random_string(rng: &mut impl Rng, len: u8) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

    (0..len)
        .map(|_| *ALPHABET.choose(rng).unwrap() as char)
        .collect()
}

async fn new_token(bucket: &Bucket, prefix: impl Into<String>, len: u8) -> Result<String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time travel?")
        .as_secs();
    let mut rng = SmallRng::seed_from_u64(now);
    let prefix: String = prefix.into();

    loop {
        let token = random_string(&mut rng, len);

        // Retry if object already exists
        let obj = bucket.head(prefix.clone() + &token).await?;
        if obj.is_none() {
            return Ok(token);
        }
    }
}

async fn read_with_meta<M, T>(bucket: &Bucket, key: impl Into<String>) -> Result<Option<(M, T)>>
where
    M: Metadata,
    T: DeserializeOwned,
{
    if let Some(obj) = bucket.get(key).execute().await? {
        let metadata = obj.custom_metadata()?;
        let body = obj.body().unwrap();
        return Ok(Some((
            metadata.into(),
            serde_bare::de::from_slice(&body.bytes().await?).unwrap(),
        )));
    }
    Ok(None)
}

async fn read<T>(bucket: &Bucket, key: impl Into<String>) -> Result<Option<T>>
where
    T: DeserializeOwned,
{
    if let Some(obj) = bucket.get(key).execute().await? {
        let body = obj.body().unwrap();
        return Ok(Some(
            serde_bare::de::from_slice(&body.bytes().await?).unwrap(),
        ));
    }
    Ok(None)
}

async fn save_with_meta<M, T>(
    bucket: &Bucket,
    key: impl Into<String>,
    data: &T,
    metadata: M,
) -> Result<()>
where
    M: Metadata,
    T: Serialize,
{
    bucket
        .put(key, serde_bare::ser::to_vec(data).unwrap())
        .custom_metadata(metadata)
        .execute()
        .await?;
    Ok(())
}

async fn create_room(bucket: &Bucket, token: &str, signals: Vec<Signal>) -> Result<String> {
    let code = new_token(bucket, "room:", 6).await?;
    save_with_meta(
        bucket,
        "auth:".to_owned() + token,
        &Auth {
            room: Some(code.clone()),
        },
        AuthMetadata { kill_at: None },
    )
    .await?;
    save_with_meta(
        bucket,
        "room:".to_owned() + &code,
        &Room {
            offer: Peer {
                sent_sdp: signals.iter().any(|s| matches!(s, Signal::SetSDP(_))),
                queue: vec![],
            },
            answer: Peer {
                sent_sdp: false,
                queue: signals,
            },
        },
        RoomMetadata {
            offer: PeerMetadata {
                token: token.to_owned(),
                last_read: SystemTime::now(),
            },
            answer: None,
        },
    )
    .await?;
    Ok(code)
}

async fn update_meta(
    bucket: &Bucket,
    meta: &mut RoomMetadata,
    room: &mut Room,
    token: &str,
    code: &str,
) -> Result<bool> {
    if meta.offer.token == token {
        meta.offer.last_read = SystemTime::now();
        return Ok(true);
    }

    if let Some(ref mut peer) = meta.answer {
        if peer.token != token {
            return Ok(false);
        }
        peer.last_read = SystemTime::now();
        return Ok(true);
    }

    save_with_meta(
        bucket,
        "auth:".to_owned() + token,
        &Auth {
            room: Some(code.to_owned()),
        },
        AuthMetadata { kill_at: None },
    )
    .await?;
    room.answer.queue.push(Signal::JoinRoom(code.to_owned()));
    meta.answer = Some(PeerMetadata {
        token: token.to_owned(),
        last_read: SystemTime::now(),
    });

    Ok(true)
}

async fn ident(env: Env) -> Result<Response> {
    let bucket = env.bucket("rtc")?;
    let token = new_token(&bucket, "auth:", 32).await?;
    save_with_meta(
        &bucket,
        "auth:".to_owned() + &token,
        &Auth { room: None },
        AuthMetadata {
            kill_at: Some(SystemTime::now() + Duration::from_secs(20)),
        },
    )
    .await?;
    Response::from_json(&IdentResponse { token })
}

async fn poll(mut req: Request, env: Env) -> Result<Response> {
    let token = match req.headers().get("Authorization")? {
        Some(token) => token,
        None => {
            return Response::error("Missing authorization token.", 403);
        }
    };
    let signals = req.json::<Vec<Signal>>().await?;
    if signals.iter().any(|s| matches!(s, Signal::ConnectAt(_))) {
        return Response::error("Can't send signal ConnectAt.", 400);
    }

    let bucket = env.bucket("rtc")?;

    let user = read::<Auth>(&bucket, "auth:".to_owned() + &token).await?;
    let user = match user {
        Some(user) => user,
        None => {
            return Response::error("Invalid authorization token.", 403);
        }
    };

    let join = signals.iter().find(|s| matches!(s, Signal::JoinRoom(_)));
    let code = match (user.room, join) {
        (Some(room), None) => room,
        (None, Some(Signal::JoinRoom(code))) => {
            // Join a room
            code.to_owned()
        }
        (None, None) => {
            // Create room
            let code = create_room(&bucket, &token, signals).await?;
            return Response::from_json(&vec![Signal::JoinRoom(code)]);
        }
        (Some(_), Some(_)) => {
            return Response::error("Already in a room.", 400);
        }
        (None, Some(_)) => {
            return Response::error("server logic error.", 500);
        }
    };
    let opt = read_with_meta::<RoomMetadata, Room>(&bucket, "room:".to_owned() + &code).await?;
    let (mut meta, mut room) = match opt {
        Some(obj) => obj,
        None => {
            return Response::error("Invalid room code.", 400);
        }
    };

    if !update_meta(&bucket, &mut meta, &mut room, &token, &code).await? {
        return Response::error("Invalid room code.", 400);
    }

    let loc;
    let (rem, rem_meta);
    if meta.offer.token == token {
        loc = &mut room.offer;
        (rem, rem_meta) = (&mut room.answer, meta.answer.as_mut());
    } else {
        loc = &mut room.answer;
        (rem, rem_meta) = (&mut room.offer, Some(&mut meta.offer));
    }

    if signals.iter().any(|s| matches!(s, Signal::SetSDP(_))) {
        if loc.sent_sdp {
            return Response::error("Already sent SDP.", 400);
        }

        loc.sent_sdp = true;
        if rem.sent_sdp {
            // Connect
            let connect = rem_meta.unwrap().last_read + Duration::from_secs(15);
            loc.queue.push(Signal::ConnectAt(connect));
            rem.queue.push(Signal::ConnectAt(connect));
        }
    }

    let queue: Vec<_> = loc.queue.drain(0..).collect();

    rem.queue.extend(
        signals
            .iter()
            .filter(|s| !matches!(s, Signal::JoinRoom(_)))
            .cloned(),
    );

    save_with_meta(&bucket, "room:".to_owned() + &code, &room, meta).await?;

    Response::from_json(&queue)
}

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    if !matches!(req.method(), Method::Post) {
        return Response::error("Method Not Allowed", 405);
    }

    let path = req.path();
    if path == "/ident" {
        return ident(env).await;
    } else if path == "/poll" {
        return poll(req, env).await;
    }

    Response::error("Page Not Found", 404)
}

#[event(scheduled)]
async fn cleanup(_evt: ScheduledEvent, env: Env, _ctx: ScheduleContext) {
    let bucket = env.bucket("rtc").expect("missing R2 bucket");
    let objects = bucket
        .list()
        .include(vec![Include::CustomMetadata])
        .execute()
        .await
        .expect("couldn't list rooms");

    let now = SystemTime::now();
    let kill_after = Duration::from_secs(20);
    let mut to_delete = vec![];

    for obj in objects.objects().iter() {
        if obj.key().starts_with("auth:") {
            let meta: AuthMetadata = obj.custom_metadata().unwrap().into();

            if let Some(kill_at) = meta.kill_at {
                if now >= kill_at {
                    to_delete.push(obj.key());
                }
            }
        } else {
            let meta: RoomMetadata = obj.custom_metadata().unwrap().into();

            let mut kill = false;
            if let Some(ref answer) = meta.answer {
                kill = now >= answer.last_read + kill_after;
            }
            kill = kill || now >= meta.offer.last_read + kill_after;

            if kill {
                to_delete.push("auth:".to_owned() + &meta.offer.token);
                if let Some(ref answer) = meta.answer {
                    to_delete.push("auth:".to_owned() + &answer.token);
                }
                to_delete.push(obj.key());
            }
        }
    }

    console_log!("deleting {:?}", to_delete);
    for key in to_delete.iter() {
        bucket.delete(key).await.unwrap();
    }
}
