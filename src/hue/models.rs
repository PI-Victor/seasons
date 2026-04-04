use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredBridge {
    pub id: String,
    #[serde(alias = "internalipaddress")]
    pub internal_ip_address: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BridgeConnection {
    pub bridge_ip: String,
    pub username: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserRequest {
    pub bridge_ip: String,
    pub device_type: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RegisteredApp {
    pub username: String,
    pub client_key: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Light {
    pub id: String,
    pub name: String,
    pub is_on: Option<bool>,
    pub brightness: Option<u8>,
    pub saturation: Option<u8>,
    pub hue: Option<u16>,
    pub xy: Option<[f32; 2]>,
    pub reachable: Option<bool>,
    pub light_type: Option<String>,
    pub model_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LightStateUpdate {
    pub on: Option<bool>,
    pub brightness: Option<u8>,
    pub saturation: Option<u8>,
    pub hue: Option<u16>,
    pub transition_time: Option<u16>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SetLightStateRequest {
    pub bridge_ip: String,
    pub username: String,
    pub light_id: String,
    pub state: LightStateUpdate,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Scene {
    pub id: String,
    pub name: String,
    pub group_id: Option<String>,
    pub light_count: usize,
    pub scene_type: Option<String>,
    pub preview_color_soft: Option<String>,
    pub preview_color_main: Option<String>,
    pub preview_color_deep: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActivateSceneRequest {
    pub bridge_ip: String,
    pub username: String,
    pub scene_id: String,
    pub group_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSceneRequest {
    pub bridge_ip: String,
    pub username: String,
    pub scene_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreateSceneRequest {
    pub bridge_ip: String,
    pub username: String,
    pub group_id: String,
    pub scene_name: String,
    pub light_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub id: String,
    pub name: String,
    pub kind: GroupKind,
    pub light_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum GroupKind {
    Room,
    Zone,
}
