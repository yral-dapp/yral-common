pub mod cents_withdrawal;

use sealed_metric::SealedMetric;
use serde::Serialize;
use web_time::{SystemTime, UNIX_EPOCH};

mod sealed_metric {
    use std::fmt::Debug;

    use serde::Serialize;

    pub trait SealedMetric: Serialize + Debug {
        fn tag(&self) -> String;
    }
}

pub trait Metric: SealedMetric {}

impl<T: SealedMetric> Metric for T {}

#[derive(Serialize, Clone, Copy, Debug)]
pub enum EventSource {
    PumpNDumpWorker,
}

#[derive(Serialize, Debug)]
pub struct MetricEvent<M: Metric> {
    pub source: EventSource,
    pub tag: String,
    pub metric: M,
    pub unix_timestamp_secs: u64,
}

impl<M: Metric> MetricEvent<M> {
    pub fn new(source: EventSource, metric: M) -> Self {
        Self {
            source,
            tag: metric.tag(),
            metric,
            unix_timestamp_secs: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}
