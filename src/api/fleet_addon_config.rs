use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// This provides a config for fleet addon functionality
#[derive(CustomResource, Deserialize, Serialize, Clone, Default, Debug, JsonSchema)]
#[kube(
    kind = "FleetAddonConfig",
    group = "addons.cluster.x-k8s.io",
    version = "v1alpha1"
)]
pub struct FleetAddonConfigSpec {
    /// Cluster class controller settings
    pub cluster_class: Option<ClusterClassConfig>,
    /// Cluster controller settings
    pub cluster: Option<ClusterConfig>,
}

impl Default for FleetAddonConfig {
    fn default() -> Self {
        Self {
            metadata: Default::default(),
            spec: FleetAddonConfigSpec {
                cluster_class: Some(ClusterClassConfig {
                    enabled: Some(true),
                }),
                cluster: Some(ClusterConfig {
                    enabled: Some(true),
                }),
            },
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct ClusterClassConfig {
    /// Enable clusterClass controller functionality.
    ///
    /// This will create Fleet ClusterGroups for each ClusterClaster with the same name.
    pub enabled: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct ClusterConfig {
    /// Enable Cluster config funtionality.
    ///
    /// This will create Fleet Cluster for each Cluster with the same name.
    /// In case the cluster specifies topology.class, the name of the ClusterClass
    /// will be added to the Fleet Cluster labels.
    pub enabled: Option<bool>,
}