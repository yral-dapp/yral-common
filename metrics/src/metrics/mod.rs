pub mod cents_withdrawal;
pub mod like_video;
pub mod tides_turned;
pub mod video_duration_watched;
pub mod video_watched;

use sealed_metric::SealedMetric;
use serde::Serialize;
use web_time::{SystemTime, UNIX_EPOCH};

pub mod sealed_metric {
    use std::fmt::Debug;

    use candid::Principal;
    use serde::Serialize;

    pub trait SealedMetric: Serialize + Debug {
        fn tag(&self) -> String;
        fn user_id(&self) -> Option<String>;
        fn user_canister(&self) -> Option<Principal>;
    }
}

pub trait Metric: SealedMetric {}

impl<T: SealedMetric> Metric for T {}

#[derive(Serialize, Clone, Copy, Debug)]
pub enum EventSource {
    PumpNDumpWorker,
    Yral,
}

impl EventSource {
    pub fn page_location(&self) -> String {
        match self {
            EventSource::PumpNDumpWorker => "https://pumpdump.wtf/".to_string(),
            EventSource::Yral => "https://yral.com/".to_string(),
        }
    }

    pub fn host(page_location: &str) -> String {
        let url = reqwest::Url::parse(page_location).unwrap();
        url.host_str().unwrap().to_string()
    }
}

#[derive(Serialize, Debug)]
pub struct MetricEvent<M: Metric> {
    pub source: EventSource,
    pub tag: String,
    pub user_id: Option<String>,
    pub metric: M,
    pub unix_timestamp_secs: u64,
    pub page_location: String,
    pub host: String,
}

impl<M: Metric> MetricEvent<M> {
    pub fn new(source: EventSource, metric: M) -> Self {
        let page_location = source.page_location();

        Self {
            source,
            tag: metric.tag(),
            user_id: metric.user_id(),
            metric,
            unix_timestamp_secs: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            page_location: page_location.clone(),
            host: EventSource::host(page_location.as_str()),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct MetricEventList<M: Metric> {
    pub source: EventSource,
    pub tag: String,
    pub metric: Vec<MetricEvent<M>>,
    pub user_id: Option<String>,
    pub unix_timestamp_secs: u64,
}

impl<M: Metric> MetricEventList<M> {
    pub fn new(source: EventSource, tag: String, metric: Vec<MetricEvent<M>>) -> Self {
        Self {
            source,
            tag,
            metric,
            user_id: None,
            unix_timestamp_secs: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}
