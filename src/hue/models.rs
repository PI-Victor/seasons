// SPDX-License-Identifier: Apache-2.0
//
// Copyright 2026 Victor Palade
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
    #[serde(default)]
    pub light_ids: Vec<String>,
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
pub struct Automation {
    pub id: String,
    pub name: String,
    pub enabled: Option<bool>,
    pub automation_type: Option<String>,
    pub script_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AutomationConfigEntry {
    pub key: String,
    pub value: AutomationConfigValue,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", tag = "kind", content = "value")]
pub enum AutomationConfigValue {
    Object(Vec<AutomationConfigEntry>),
    Array(Vec<AutomationConfigValue>),
    String(String),
    Number(String),
    Bool(bool),
    Null,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AutomationDetail {
    pub id: String,
    pub name: String,
    pub enabled: Option<bool>,
    pub automation_type: Option<String>,
    pub script_id: Option<String>,
    pub script_name: Option<String>,
    pub script_type: Option<String>,
    #[serde(default)]
    pub configuration: Option<AutomationConfigValue>,
    pub instance_json: String,
    pub script_json: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SetAutomationEnabledRequest {
    pub connection: BridgeConnection,
    pub automation_id: String,
    pub enabled: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAutomationRequest {
    pub connection: BridgeConnection,
    pub automation_id: String,
    pub name: String,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub configuration: Option<AutomationConfigValue>,
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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AudioSyncPreview {
    pub entertainment_area_id: String,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub intensity: f32,
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
    NeonPulse,
    Prism,
    VocalGlow,
    FireIce,
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
pub struct Sensor {
    pub id: String,
    pub name: String,
    pub sensor_type: Option<String>,
    pub model_id: Option<String>,
    pub reachable: Option<bool>,
    pub battery: Option<u8>,
    pub last_updated: Option<String>,
    pub summary: Option<String>,
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
