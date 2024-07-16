use std::{collections::BTreeMap, sync::Arc};

use chrono::Utc;
use k8s_openapi::NamespaceResourceScope;
use kube::{
    runtime::{controller::Action, finalizer},
    Api, ResourceExt as _,
};
use serde::{de::DeserializeOwned, Serialize};
use tracing::instrument;

use crate::{
    api::{
        capi_cluster::Cluster,
        fleet_clustergroup::{ClusterGroup, ClusterGroupSelector, ClusterGroupSpec},
    },
    controllers::controller::FLEET_FINALIZER,
    telemetry, Error,
};

use super::{
    controller::{get_or_create, Context},
    GroupSyncError, SyncError,
};

pub static ALL_CLUSTER_CLASS_LABEL: &str = "clusterclasses.fleet.addons.cluster.x-k8s.io";
pub static ALL_CLUSTER_LABEL: &str = "clusters.fleet.addons.cluster.x-k8s.io";

#[instrument(skip_all, fields(trace_id = display(telemetry::get_trace_id()), name = cluster.name_any(), namespace = cluster.namespace()))]
async fn ensure_groups_exist(cluster: Arc<Cluster>, ctx: Arc<Context>) -> crate::Result<Action> {
    let namespace = cluster.namespace().unwrap_or_default();
    let api = Api::namespaced(ctx.client.clone(), namespace.as_str());

    let all_cluster_group = ClusterGroup::new(
        "fleet-addon-clusters",
        ClusterGroupSpec {
            selector: Some(ClusterGroupSelector {
                match_labels: Some(BTreeMap::from([(ALL_CLUSTER_LABEL.into(), "".into())])),
                ..Default::default()
            }),
        },
    );
    let all_classes_group = ClusterGroup::new(
        "fleet-addon-clusterclasses",
        ClusterGroupSpec {
            selector: Some(ClusterGroupSelector {
                match_labels: Some(BTreeMap::from([(
                    ALL_CLUSTER_CLASS_LABEL.into(),
                    "".into(),
                )])),
                ..Default::default()
            }),
        },
    );
    finalizer(&api, FLEET_FINALIZER, cluster, |event| async {
        match event {
            finalizer::Event::Apply(c) => {
                get_or_create(ctx.clone(), all_cluster_group)
                    .await
                    .map_err(Into::<GroupSyncError>::into)
                    .map_err(Into::<SyncError>::into)?;
                get_or_create(ctx, all_classes_group)
                    .await
                    .map_err(Into::<GroupSyncError>::into)
                    .map_err(Into::<SyncError>::into)
            }
            finalizer::Event::Cleanup(c) => todo!(),
        }
    })
    .await.map_err(Into::into)?;
    // .map_err(|e| Error::FinalizerError(Box::new(e)));

    Ok(Action::await_change())
}
