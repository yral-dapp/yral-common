use yral_config_keys::ConfigKey;

pub struct KVConfig {
    url: String,
    token: String,
}

#[derive(Debug)]
pub enum KVFetchError {
    Client(reqwest::Error),
    KeyNotFound,
    StatusNotOk(u16),
    Decode(reqwest::Error),
    Parse(serde_json::Error),
    InvalidUrlOrKeyName,
}

impl KVConfig {
    pub fn new(url: String, token: String) -> KVConfig {
        KVConfig { url, token }
    }

    fn url<K: ConfigKey>(&self, key: K) -> Result<String, KVFetchError> {
        let url = url::Url::parse(&self.url)
            .map(|url| url.join(&key.to_string()).map(|url| url.to_string()));

        let Ok(Ok(url)) = url else {
            return Err(KVFetchError::InvalidUrlOrKeyName);
        };

        Ok(url)
    }

    pub async fn get<K: ConfigKey>(&self, key: K) -> Result<K::Value, KVFetchError> {
        let url = self.url(key)?;

        let client = reqwest::Client::new();
        let value = match client
            .get(url)
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await
        {
            Err(err) => return Err(KVFetchError::Client(err)),
            Ok(resp) => match resp.status().as_u16() {
                200 => {
                    let value = match resp.text().await {
                        Err(err) => return Err(KVFetchError::Decode(err)),
                        Ok(value) => value,
                    };

                    let value = match serde_json::from_reader(value.as_bytes()) {
                        Err(err) => return Err(KVFetchError::Parse(err)),
                        Ok(value) => value,
                    };

                    value
                }
                404 => match <K as ConfigKey>::fallback() {
                    Some(value) => value,
                    None => return Err(KVFetchError::KeyNotFound),
                },
                status_code => return Err(KVFetchError::StatusNotOk(status_code)),
            },
        };

        Ok(value)
    }

    pub async fn set<K: ConfigKey>(&self, key: K, value: K::Value) -> Result<(), KVFetchError> {
        let url = self.url(key)?;

        let value = match serde_json::to_string(&value) {
            Err(err) => return Err(KVFetchError::Parse(err)),
            Ok(value) => value,
        };

        let client = reqwest::Client::new();
        let _ = match client
            .post(url)
            .body(value)
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await
        {
            Err(err) => return Err(KVFetchError::Client(err)),
            Ok(resp) => match resp.status().as_u16() {
                200 => resp,
                status_code => return Err(KVFetchError::StatusNotOk(status_code)),
            },
        };

        Ok(())
    }
}
