pub mod capi_cluster;
pub mod capi_clusterclass;
pub mod fleet_addon_config;
pub mod fleet_cluster;
#[cfg(feature = "agent-initiated")]
pub mod fleet_cluster_registration_token;
pub mod fleet_clustergroup;

pub mod generate_patches;
pub mod generate_patches_request;
pub mod generate_patches_response;
pub mod validate_topology_request;
pub mod validate_topology_response;
pub mod discover_patches_request;
pub mod discover_patches_response;
