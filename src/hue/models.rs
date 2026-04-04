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
    #[serde(default)]
    pub client_key: Option<String>,
    #[serde(default)]
    pub application_id: Option<String>,
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
    #[serde(default)]
    pub client_key: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EntertainmentArea {
    pub id: String,
    pub name: String,
    pub configuration_type: Option<String>,
    pub status: String,
    pub channels: Vec<EntertainmentChannel>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EntertainmentChannel {
    pub channel_id: u8,
    pub position: EntertainmentPosition,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EntertainmentPosition {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AudioSyncStartRequest {
    pub connection: BridgeConnection,
    pub entertainment_area_id: String,
    #[serde(default)]
    pub pipewire_target_object: Option<String>,
    #[serde(default)]
    pub speed_mode: AudioSyncSpeedMode,
    #[serde(default)]
    pub color_palette: AudioSyncColorPalette,
    #[serde(default)]
    pub base_color_hex: Option<String>,
    #[serde(default)]
    pub brightness_ceiling: Option<u8>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AudioSyncUpdateRequest {
    #[serde(default)]
    pub speed_mode: AudioSyncSpeedMode,
    #[serde(default)]
    pub color_palette: AudioSyncColorPalette,
    #[serde(default)]
    pub base_color_hex: Option<String>,
    #[serde(default)]
    pub brightness_ceiling: Option<u8>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AudioSyncStartResult {
    pub connection: BridgeConnection,
    pub entertainment_area_id: String,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum AudioSyncSpeedMode {
    Slow,
    #[default]
    Medium,
    High,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum AudioSyncColorPalette {
    #[default]
    CurrentRoom,
    Sunset,
    Aurora,
    Ocean,
    Rose,
    Mono,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PipeWireOutputTarget {
    pub target_object: String,
    pub name: String,
    pub description: String,
    pub media_class: String,
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
