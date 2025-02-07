use worker::kv::KvStore;
use yral_config_keys::ConfigKey;

pub struct KVConfig {
    store: KvStore,
}

#[derive(Debug)]
pub enum KVFetchError {
    KvError(worker::kv::KvError),
    KeyNotFound,
    Serde(serde_json::Error),
}

impl KVConfig {
    pub fn new(store: KvStore) -> KVConfig {
        KVConfig { store }
    }

    pub async fn get<K: ConfigKey>(&self, key: K) -> Result<K::Value, KVFetchError> {
        let value = match self.store.get(&key.to_string()).text().await {
            Err(err) => return Err(KVFetchError::KvError(err)),
            Ok(value) => match value {
                Some(value) => match serde_json::from_reader(value.as_bytes()) {
                    Err(err) => return Err(KVFetchError::Serde(err)),
                    Ok(value) => value,
                },
                None => match <K as ConfigKey>::fallback() {
                    Some(value) => value,
                    None => return Err(KVFetchError::KeyNotFound),
                },
            },
        };

        Ok(value)
    }

    pub async fn set<K: ConfigKey>(&self, key: K, value: K::Value) -> Result<(), KVFetchError> {
        let value = match serde_json::to_string(&value) {
            Err(err) => return Err(KVFetchError::Serde(err)),
            Ok(value) => value,
        };

        match self.store.put(&key.to_string(), &value) {
            Err(err) => return Err(KVFetchError::KvError(err)),
            Ok(builder) => match builder.execute().await {
                Ok(()) => Ok(()),
                Err(err) => return Err(KVFetchError::KvError(err)),
            },
        }
    }
}
