use std::sync::Arc;

use crate::Error;
use chrono::{DateTime, Utc};
use educe::Educe;
use kube::{
    runtime::events::{Recorder, Reporter},
    Client, ResourceExt,
};
use prometheus::{histogram_opts, opts, HistogramVec, IntCounter, IntCounterVec, Registry};
use serde::Serialize;
use tokio::time::Instant;

/// Creates a new `IntCounter` for tracking total reconciliations.
fn default_reconciliations() -> IntCounter {
    IntCounter::new("caapf_controller_reconciliations_total", "reconciliations").unwrap()
}

/// Creates a new `IntCounterVec` for tracking reconciliation failures, labeled by instance name and error type.
fn default_failures() -> IntCounterVec {
    IntCounterVec::new(
        opts!(
            "caapf_controller_reconciliation_errors_total",
            "reconciliation errors",
        ),
        &["instance", "error"],
    )
    .unwrap()
}

/// Creates a new `HistogramVec` for tracking the duration of reconcile operations.
fn default_reconcile_duration() -> HistogramVec {
    HistogramVec::new(
        histogram_opts!(
            "caapf_controller_reconcile_duration_seconds",
            "The duration of reconcile to complete in seconds"
        )
        .buckets(vec![0.01, 0.1, 0.25, 0.5, 1., 5., 15., 60.]),
        &[],
    )
    .unwrap()
}

#[derive(Clone, Educe)]
#[educe(Default)]
pub struct Metrics {
    #[educe(Default(expression = default_reconciliations()))]
    pub reconciliations: IntCounter,
    #[educe(Default(expression = default_failures()))]
    pub failures: IntCounterVec,
    #[educe(Default(expression = default_reconcile_duration()))]
    pub reconcile_duration: HistogramVec,
}

impl Metrics {
    /// Register API metrics to start tracking them.
    ///
    /// Registers the `reconciliations`, `failures`, and `reconcile_duration` metrics
    /// with the provided Prometheus `Registry`.
    pub fn register(self, registry: &Registry) -> Result<Self, prometheus::Error> {
        registry.register(Box::new(self.reconcile_duration.clone()))?;
        registry.register(Box::new(self.failures.clone()))?;
        registry.register(Box::new(self.reconciliations.clone()))?;
        Ok(self)
    }

    /// Records a reconciliation failure.
    ///
    /// Increments the `failures` counter, labeling the failure with the name of the
    /// Kubernetes resource instance and the type of error that occurred.
    #[allow(clippy::needless_pass_by_value)]
    pub fn reconcile_failure<C: kube::Resource>(&self, obj: Arc<C>, e: &Error) {
        self.failures
            .with_label_values(&[obj.name_any(), e.metric_label()])
            .inc();
    }

    /// Increments the reconciliation counter and starts a duration measurement.
    ///
    /// This method should be called at the beginning of a reconciliation loop. It increments
    /// the `reconciliations` counter and returns a `ReconcileMeasurer` which, when dropped,
    /// will record the duration of the reconciliation.
    #[must_use]
    pub fn count_and_measure(&self) -> ReconcileMeasurer {
        self.reconciliations.inc();
        ReconcileMeasurer {
            start: Instant::now(),
            metric: self.reconcile_duration.clone(),
        }
    }
}

/// Diagnostics to be exposed by the web server
#[derive(Clone, Serialize, Educe)]
#[educe(Default)]
pub struct Diagnostics {
    #[serde(deserialize_with = "from_ts")]
    #[educe(Default(expression = Utc::now()))]
    pub last_event: DateTime<Utc>,
    #[serde(skip)]
    #[educe(Default = "doc-controller")]
    pub reporter: Reporter,
}

impl Diagnostics {
    /// Creates a new Kubernetes event recorder.
    pub fn recorder(&self, client: Client) -> Recorder {
        Recorder::new(client, self.reporter.clone())
    }
}

/// Function duration measurer
pub struct ReconcileMeasurer {
    start: Instant,
    metric: HistogramVec,
}

impl Drop for ReconcileMeasurer {
    /// Records the duration of the reconciliation when the `ReconcileMeasurer` is dropped.
    ///
    /// This method is automatically called when the `ReconcileMeasurer` goes out of scope.
    /// It calculates the elapsed time since the `ReconcileMeasurer` was created and
    /// observes this duration in the associated `reconcile_duration` histogram metric.
    fn drop(&mut self) {
        #[allow(clippy::cast_precision_loss)]
        let duration = self.start.elapsed().as_millis() as f64 / 1000.0;
        self.metric.with_label_values::<&str>(&[]).observe(duration);
    }
}
