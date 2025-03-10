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
    Serde(serde_json::Error),
    InvalidUrlOrKeyName,
}

impl KVConfig {
    pub fn new(url: String, token: String) -> KVConfig {
        KVConfig { url, token }
    }

    fn url<K: ConfigKey>(&self, key: &K, ovride: &Option<String>) -> Result<String, KVFetchError> {
        let key_and_ovride = match ovride {
            Some(ovride) => format!("{}:{}", key, ovride),
            None => key.to_string(),
        };

        let url = format!("{}/{}", self.url, key_and_ovride);

        Ok(url)
    }

    async fn get_value_from_url<K: ConfigKey>(
        &self,
        url: String,
        fallback: bool,
    ) -> Result<<K as ConfigKey>::Value, KVFetchError> {
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
                        Err(err) => return Err(KVFetchError::Serde(err)),
                        Ok(value) => value,
                    };

                    value
                }
                404 => match (<K as ConfigKey>::fallback(), fallback) {
                    (Some(value), true) => value,
                    _ => return Err(KVFetchError::KeyNotFound),
                },
                status_code => return Err(KVFetchError::StatusNotOk(status_code)),
            },
        };

        Ok(value)
    }

    pub async fn get<K: ConfigKey>(
        &self,
        key: K,
        ovride: Option<String>,
    ) -> Result<K::Value, KVFetchError> {
        let url = self.url(&key, &ovride)?;

        match self.get_value_from_url::<K>(url, false).await {
            Err(KVFetchError::KeyNotFound) => {
                let url = self.url(&key, &None::<String>)?;
                self.get_value_from_url::<K>(url, true).await
            }
            result => result,
        }
    }

    pub async fn set<K: ConfigKey>(
        &self,
        key: K,
        value: K::Value,
        ovride: Option<String>,
    ) -> Result<(), KVFetchError> {
        let url = self.url(&key, &ovride)?;

        let value = match serde_json::to_string(&value) {
            Err(err) => return Err(KVFetchError::Serde(err)),
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
