use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use web_time::{Duration, SystemTime, UNIX_EPOCH};

use crate::{
    db::{BucketInfo, Data, Metadata},
    poll::Killable,
};

pub const GRACE_PERIOD: u64 = 20;

pub type Auth = Data<AuthData, AuthMetadata, AuthInfo>;

pub struct AuthInfo {}
impl BucketInfo for AuthInfo {
    const PREFIX: &'static str = "auth";
    const KEY_LENGTH: u8 = 32;
}

#[derive(Serialize, Deserialize, Default)]
pub struct AuthData {
    room: Option<String>,
}

pub struct AuthMetadata {
    kill_at: Option<SystemTime>,
}
impl Default for AuthMetadata {
    fn default() -> Self {
        AuthMetadata {
            kill_at: Some(SystemTime::now() + Duration::from_secs(GRACE_PERIOD)),
        }
    }
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

impl Auth {
    pub fn get_room(&self) -> Option<&String> {
        let data = self.data.as_ref().expect("invalid state");
        data.room.as_ref()
    }

    pub fn join_room(&mut self, code: &str) {
        let data = self.data.as_mut().expect("invalid state");
        data.room = Some(code.to_owned());
        self.meta.kill_at = None;
        self.modified = true;
    }
}

impl Killable for Auth {
    fn get_keys_to_kill(&self) -> Vec<String> {
        match self.meta.kill_at {
            Some(at) => {
                if SystemTime::now() >= at {
                    vec![format!("{}:{}", AuthInfo::PREFIX, self.key)]
                } else {
                    vec![]
                }
            }
            None => vec![],
        }
    }
}
