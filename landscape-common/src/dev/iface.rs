use serde::{Deserialize, Serialize};

use super::wifi::LandscapeWifiInterface;
use crate::config_service::iface::{IfaceZoneType, NetworkIfaceConfig};
use crate::dev::LandscapeInterface;

#[derive(Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct BridgeCreate {
    pub name: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AddController {
    pub link_name: String,
    pub link_ifindex: u32,
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(required = true, nullable = true))]
    pub master_name: Option<String>,
    #[serde(default)]
    #[cfg_attr(feature = "openapi", schema(required = true, nullable = true))]
    pub master_ifindex: Option<u32>,
}

#[derive(Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ChangeZone {
    pub iface_name: String,
    pub zone: IfaceZoneType,
}

#[derive(Serialize, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct IfaceTopology {
    #[serde(flatten)]
    pub config: NetworkIfaceConfig,
    #[serde(flatten)]
    pub status: LandscapeInterface,
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub wifi_info: Option<LandscapeWifiInterface>,
}

#[derive(Serialize, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct IfaceInfo {
    pub config: NetworkIfaceConfig,
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub status: Option<LandscapeInterface>,
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub wifi_info: Option<LandscapeWifiInterface>,
}

#[derive(Serialize, Debug, Clone)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct RawIfaceInfo {
    pub status: LandscapeInterface,
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub wifi_info: Option<LandscapeWifiInterface>,
}

#[derive(Clone, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct IfacesInfo {
    pub managed: Vec<IfaceInfo>,
    pub unmanaged: Vec<RawIfaceInfo>,
}
