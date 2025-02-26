use controllers::{
    addon_config::{AddonConfigSyncError, DynamicWatcherError},
    BundleError, SyncError,
};
use futures::channel::mpsc::TrySendError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Kube Error: {0}")]
    KubeError(#[from] kube::Error),

    #[error("Config fetch error: {0}")]
    ConfigFetch(#[source] kube::Error),

    #[error("Bundle error: {0}")]
    BundleError(#[from] BundleError),

    #[error("Fleet error: {0}")]
    FleetError(#[from] SyncError),

    #[error("Fleet config error: {0}")]
    FleetConfigError(#[from] AddonConfigSyncError),

    #[error("Dynamic watcher error: {0}")]
    DynamicWatcherError(#[from] DynamicWatcherError),

    #[error("Namespace trigger error: {0}")]
    TriggerError(#[from] TrySendError<()>),

    #[error("Finalizer Error: {0}")]
    // NB: awkward type because finalizer::Error embeds the reconciler error (which is this)
    // so boxing this error to break cycles
    FinalizerError(#[source] Box<kube::runtime::finalizer::Error<Error>>),

    #[error("IllegalDocument")]
    IllegalDocument,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

impl Error {
    pub fn metric_label(&self) -> String {
        format!("{self:?}").to_lowercase()
    }
}

/// Expose all controller components used by main
pub mod controller;
pub use crate::controller::*;
pub mod api;
pub mod controllers;
pub mod predicates;

/// Log and trace integrations
pub mod telemetry;

/// Metrics
mod metrics;
pub use metrics::Metrics;

/*
#[cfg(test)] pub mod fixtures;
*/
