#[cfg(feature = "js")]
pub mod js_spawn;
pub mod mock;
pub mod vectordb;

use std::{error::Error, future::Future};

use crate::metrics::{EventSource, Metric, MetricEvent};

pub trait MetricEventTx: Send {
    type Error: Error;

    fn push<M: Metric + Send + 'static>(
        &self,
        ev: MetricEvent<M>,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait LocalMetricEventTx {
    type Error: Error;

    fn push_local<M: Metric + Send + 'static>(
        &self,
        ev: MetricEvent<M>,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>>;
}

impl<T: MetricEventTx> LocalMetricEventTx for T {
    type Error = <Self as MetricEventTx>::Error;

    async fn push_local<M: Metric + Send + 'static>(
        &self,
        ev: MetricEvent<M>,
    ) -> Result<(), Self::Error> {
        MetricEventTx::push(self, ev).await
    }
}

pub struct LocalMetricTx<Tx> {
    source: EventSource,
    tx: Tx,
}

impl<Tx: LocalMetricEventTx> LocalMetricTx<Tx> {
    pub fn new(source: EventSource, tx: Tx) -> Self {
        Self { source, tx }
    }

    pub async fn push(&self, metric: impl Metric + Send + 'static) -> Result<(), Tx::Error> {
        self.tx
            .push_local(MetricEvent::new(self.source, metric))
            .await
    }
}

pub struct MetricTx<Tx> {
    source: EventSource,
    tx: Tx,
}

impl<Tx: MetricEventTx> MetricTx<Tx> {
    pub fn new(source: EventSource, tx: Tx) -> Self {
        Self { source, tx }
    }

    pub async fn push(&self, metric: impl Metric + Send + 'static) -> Result<(), Tx::Error> {
        self.tx.push(MetricEvent::new(self.source, metric)).await
    }
}
