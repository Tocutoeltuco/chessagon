use std::{collections::HashMap, marker::PhantomData};

use rand::{rngs::SmallRng, seq::SliceRandom, Rng, SeedableRng};
use serde::{de::DeserializeOwned, Serialize};
use web_time::{SystemTime, UNIX_EPOCH};
use worker::{Bucket, Object, Result};

use crate::poll::Readable;

fn random_string(rng: &mut impl Rng, len: u8) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

    (0..len)
        .map(|_| *ALPHABET.choose(rng).unwrap() as char)
        .collect()
}

async fn new_key(bucket: &Bucket, prefix: impl Into<String>, len: u8) -> Result<String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time travel?")
        .as_secs();
    let mut rng = SmallRng::seed_from_u64(now);
    let prefix: String = format!("{}:", prefix.into());

    loop {
        let token = random_string(&mut rng, len);

        // Retry if object already exists
        let obj = bucket.head(prefix.clone() + &token).await?;
        if obj.is_none() {
            return Ok(token);
        }
    }
}

pub trait Metadata: From<HashMap<String, String>> + Into<HashMap<String, String>> {}

pub trait BucketInfo {
    const PREFIX: &'static str = "";
    const KEY_LENGTH: u8 = 0;
}

pub struct Data<O, M, B> {
    pub modified: bool,
    pub key: String,
    pub data: Option<O>,
    pub meta: M,
    info: PhantomData<B>,
}

impl<O, M, B> Readable for Data<O, M, B>
where
    O: Serialize + DeserializeOwned + Default,
    M: Metadata + Default,
    B: BucketInfo,
{
    async fn read(obj: &Object) -> Result<Self> {
        Self::_read(Self::remove_prefix(obj.key()), obj).await
    }
}

impl<O, M, B> Data<O, M, B>
where
    O: Serialize + DeserializeOwned + Default,
    M: Metadata + Default,
    B: BucketInfo,
{
    fn remove_prefix(key: String) -> String {
        key.get((B::PREFIX.len() + 1)..)
            .expect("invalid key")
            .to_owned()
    }

    pub async fn create(bucket: &Bucket) -> Result<Self> {
        let key = new_key(bucket, B::PREFIX, B::KEY_LENGTH).await?;
        Ok(Self {
            modified: true,
            key,
            data: Some(Default::default()),
            meta: Default::default(),
            info: PhantomData,
        })
    }

    pub async fn load(bucket: &Bucket, key: &str) -> Result<Option<Self>> {
        match bucket
            .get(format!("{}:{}", B::PREFIX, &key).clone())
            .execute()
            .await?
        {
            Some(obj) => Ok(Some(Self::_read(key.to_owned(), &obj).await?)),
            None => Ok(None),
        }
    }

    async fn _read(key: String, obj: &Object) -> Result<Self> {
        let meta: M = obj.custom_metadata()?.into();
        let data = match obj.body() {
            Some(b) => Some(b.bytes().await?),
            None => None,
        };
        let data = data.map(|d| serde_bare::de::from_slice(&d).unwrap());

        Ok(Self {
            modified: false,
            key,
            data,
            meta,
            info: PhantomData,
        })
    }

    pub async fn write(self, bucket: &Bucket) -> Result<()> {
        if !self.modified {
            return Ok(());
        }

        let key = format!("{}:{}", B::PREFIX, self.key);
        let data = self.data.as_ref().unwrap();
        bucket
            .put(key, serde_bare::ser::to_vec(data).unwrap())
            .custom_metadata(self.meta)
            .execute()
            .await?;
        Ok(())
    }
}
