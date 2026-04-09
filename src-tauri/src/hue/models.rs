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
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};

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

impl LightStateUpdate {
    pub(crate) fn to_payload(&self) -> HueLightStatePayload {
        HueLightStatePayload {
            on: self.on,
            brightness: self.brightness,
            saturation: self.saturation,
            hue: self.hue,
            transition_time: self.transition_time,
        }
    }
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

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum HueApiResponse<T> {
    Success { success: T },
    Error { error: HueApiErrorBody },
}

#[derive(Debug, Deserialize)]
pub(crate) struct HueApiErrorBody {
    #[serde(rename = "type")]
    pub error_type: u16,
    pub address: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateUserSuccessPayload {
    pub username: String,
    #[serde(default)]
    pub clientkey: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawHueLight {
    pub name: String,
    #[serde(default)]
    pub state: RawHueLightState,
    #[serde(rename = "type")]
    pub light_type: Option<String>,
    #[serde(default)]
    pub modelid: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct RawHueLightState {
    pub on: Option<bool>,
    pub bri: Option<u8>,
    pub sat: Option<u8>,
    pub hue: Option<u16>,
    #[serde(default)]
    pub xy: Option<[f32; 2]>,
    pub reachable: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawHueSensor {
    pub name: String,
    #[serde(rename = "type")]
    pub sensor_type: Option<String>,
    #[serde(default)]
    pub modelid: Option<String>,
    #[serde(default)]
    pub state: Value,
    #[serde(default)]
    pub config: Value,
}

impl From<(String, RawHueLight)> for Light {
    fn from((id, raw): (String, RawHueLight)) -> Self {
        Self {
            id,
            name: raw.name,
            is_on: raw.state.on,
            brightness: raw.state.bri,
            saturation: raw.state.sat,
            hue: raw.state.hue,
            xy: raw.state.xy,
            reachable: raw.state.reachable,
            light_type: raw.light_type,
            model_id: raw.modelid,
        }
    }
}

impl From<(String, RawHueSensor)> for Sensor {
    fn from((id, raw): (String, RawHueSensor)) -> Self {
        Self {
            id,
            name: raw.name,
            sensor_type: raw.sensor_type,
            model_id: raw.modelid,
            reachable: raw.config.get("reachable").and_then(Value::as_bool),
            battery: raw
                .config
                .get("battery")
                .and_then(Value::as_u64)
                .and_then(|value| u8::try_from(value).ok()),
            last_updated: raw
                .state
                .get("lastupdated")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned),
            summary: sensor_summary(&raw.state),
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawHueScene {
    pub name: String,
    #[serde(default)]
    pub group: Option<String>,
    #[serde(default)]
    pub lights: Vec<String>,
    #[serde(rename = "type", default)]
    pub scene_type: Option<String>,
    #[serde(default)]
    pub recycle: bool,
}

impl From<(String, RawHueScene)> for Scene {
    fn from((id, raw): (String, RawHueScene)) -> Self {
        Self {
            id,
            name: raw.name,
            group_id: raw.group,
            light_count: raw.lights.len(),
            scene_type: raw.scene_type,
            preview_color_soft: None,
            preview_color_main: None,
            preview_color_deep: None,
        }
    }
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct RawHueSceneDetail {
    #[serde(default)]
    pub lightstates: HashMap<String, RawHueSceneLightState>,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct RawHueSceneLightState {
    pub on: Option<bool>,
    pub bri: Option<u8>,
    #[serde(default)]
    pub xy: Option<[f32; 2]>,
    pub sat: Option<u8>,
    pub hue: Option<u16>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawHueGroup {
    pub name: String,
    #[serde(default)]
    pub lights: Vec<String>,
    #[serde(rename = "type")]
    pub group_type: String,
}

impl TryFrom<(String, RawHueGroup)> for Group {
    type Error = ();

    fn try_from((id, raw): (String, RawHueGroup)) -> Result<Self, Self::Error> {
        let kind = match raw.group_type.as_str() {
            "Room" => GroupKind::Room,
            "Zone" => GroupKind::Zone,
            _ => return Err(()),
        };

        Ok(Self {
            id,
            name: raw.name,
            kind,
            light_ids: raw.lights,
        })
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct HueLightStatePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on: Option<bool>,
    #[serde(rename = "bri", skip_serializing_if = "Option::is_none")]
    pub brightness: Option<u8>,
    #[serde(rename = "sat", skip_serializing_if = "Option::is_none")]
    pub saturation: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hue: Option<u16>,
    #[serde(rename = "transitiontime", skip_serializing_if = "Option::is_none")]
    pub transition_time: Option<u16>,
}

impl HueLightStatePayload {
    pub(crate) fn is_empty(&self) -> bool {
        self.on.is_none()
            && self.brightness.is_none()
            && self.saturation.is_none()
            && self.hue.is_none()
            && self.transition_time.is_none()
    }
}

pub(crate) type RawLightsResponse = BTreeMap<String, RawHueLight>;
pub(crate) type RawSensorsResponse = BTreeMap<String, RawHueSensor>;
pub(crate) type RawGroupsResponse = BTreeMap<String, RawHueGroup>;
pub(crate) type RawScenesResponse = BTreeMap<String, RawHueScene>;
pub(crate) type RawSceneDetailResponse = RawHueSceneDetail;
pub(crate) type RawStateChangeSuccess = HashMap<String, Value>;
pub(crate) type RawSceneCreateSuccess = HashMap<String, String>;

#[derive(Debug, Deserialize)]
pub(crate) struct ClipV2ListResponse<T> {
    pub data: Vec<T>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawEntertainmentArea {
    pub id: String,
    pub metadata: RawEntertainmentAreaMetadata,
    #[serde(default)]
    pub configuration_type: Option<String>,
    #[serde(default)]
    pub status: RawEntertainmentAreaStatus,
    #[serde(default)]
    pub channels: Vec<RawEntertainmentChannel>,
    #[serde(default)]
    pub light_services: Vec<RawClipV2ResourceServiceIdentifier>,
    #[serde(default)]
    pub locations: HashMap<String, Value>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawEntertainmentAreaMetadata {
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum RawEntertainmentAreaStatus {
    String(String),
    Object {
        #[serde(default = "default_inactive_status")]
        status: String,
    },
}

impl Default for RawEntertainmentAreaStatus {
    fn default() -> Self {
        Self::String(default_inactive_status())
    }
}

impl RawEntertainmentAreaStatus {
    fn into_status(self) -> String {
        match self {
            Self::String(status) => status,
            Self::Object { status } => status,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawEntertainmentChannel {
    pub channel_id: u8,
    pub position: RawEntertainmentPosition,
    #[serde(default)]
    pub members: Vec<RawEntertainmentChannelMember>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawEntertainmentChannelMember {
    #[serde(default)]
    pub service: Option<RawClipV2ResourceServiceIdentifier>,
    #[serde(default)]
    pub rid: Option<String>,
    #[serde(default)]
    pub rtype: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawClipV2ResourceServiceIdentifier {
    pub rid: String,
    #[serde(default)]
    pub rtype: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawEntertainmentPosition {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawAutomation {
    pub id: String,
    #[serde(default, rename = "type")]
    pub automation_type: Option<String>,
    pub metadata: RawAutomationMetadata,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub script_id: Option<RawClipV2ResourceIdentifier>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawAutomationMetadata {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawClipV2ResourceIdentifier {
    pub rid: String,
}

impl From<RawAutomation> for Automation {
    fn from(raw: RawAutomation) -> Self {
        Self {
            id: raw.id,
            name: raw.metadata.name,
            enabled: raw.enabled,
            automation_type: raw.automation_type,
            script_id: raw.script_id.map(|script| script.rid),
        }
    }
}

impl From<RawEntertainmentArea> for EntertainmentArea {
    fn from(raw: RawEntertainmentArea) -> Self {
        let mut light_ids = raw
            .channels
            .iter()
            .flat_map(|channel| channel.members.iter())
            .filter_map(RawEntertainmentChannelMember::candidate_service_rid)
            .collect::<Vec<_>>();
        light_ids.extend(
            raw.light_services
                .iter()
                .filter(|service| !is_excluded_member_service_type(service.rtype.as_deref()))
                .map(|service| service.rid.clone()),
        );
        light_ids.extend(raw.locations.keys().cloned());
        light_ids.sort();
        light_ids.dedup();

        Self {
            id: raw.id,
            name: raw.metadata.name,
            configuration_type: raw.configuration_type,
            status: raw.status.into_status(),
            channels: raw
                .channels
                .into_iter()
                .map(EntertainmentChannel::from)
                .collect(),
            light_ids,
        }
    }
}

impl RawEntertainmentChannelMember {
    fn candidate_service_rid(&self) -> Option<String> {
        if let Some(service) = self.service.as_ref() {
            if !is_excluded_member_service_type(service.rtype.as_deref())
                && !service.rid.trim().is_empty()
            {
                return Some(service.rid.trim().to_string());
            }
        }

        let rid = self.rid.as_deref()?.trim();
        if rid.is_empty() || is_excluded_member_service_type(self.rtype.as_deref()) {
            return None;
        }
        Some(rid.to_string())
    }
}

fn is_excluded_member_service_type(rtype: Option<&str>) -> bool {
    rtype.is_some_and(|rtype| rtype.eq_ignore_ascii_case("device"))
}

impl From<RawEntertainmentChannel> for EntertainmentChannel {
    fn from(raw: RawEntertainmentChannel) -> Self {
        Self {
            channel_id: raw.channel_id,
            position: EntertainmentPosition::from(raw.position),
        }
    }
}

impl From<RawEntertainmentPosition> for EntertainmentPosition {
    fn from(raw: RawEntertainmentPosition) -> Self {
        Self {
            x: raw.x,
            y: raw.y,
            z: raw.z,
        }
    }
}

fn default_inactive_status() -> String {
    "inactive".to_string()
}

fn sensor_summary(state: &Value) -> Option<String> {
    if let Some(presence) = state.get("presence").and_then(Value::as_bool) {
        return Some(if presence {
            "Motion detected".to_string()
        } else {
            "No motion".to_string()
        });
    }

    if let Some(temperature) = state.get("temperature").and_then(Value::as_i64) {
        return Some(format!("{:.1}°C", temperature as f32 / 100.0));
    }

    if let Some(daylight) = state.get("daylight").and_then(Value::as_bool) {
        return Some(if daylight {
            "Daylight".to_string()
        } else {
            "No daylight".to_string()
        });
    }

    if let Some(dark) = state.get("dark").and_then(Value::as_bool) {
        return Some(if dark {
            "Dark".to_string()
        } else {
            "Bright".to_string()
        });
    }

    if let Some(lightlevel) = state.get("lightlevel").and_then(Value::as_i64) {
        return Some(format!("Light level {lightlevel}"));
    }

    if let Some(button_event) = state.get("buttonevent").and_then(Value::as_i64) {
        return Some(format!("Button event {button_event}"));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::{
        Automation, ClipV2ListResponse, CreateUserSuccessPayload, EntertainmentArea, Group,
        GroupKind, HueApiResponse, HueLightStatePayload, Light, LightStateUpdate, RawAutomation,
        RawEntertainmentArea, RawGroupsResponse, RawLightsResponse, RawScenesResponse,
        RawSensorsResponse, Scene, Sensor,
    };

    #[test]
    fn parses_create_user_success() {
        let raw = r#"[{"success":{"username":"token-123","clientkey":"client-key"}}]"#;
        let response: Vec<HueApiResponse<CreateUserSuccessPayload>> =
            serde_json::from_str(raw).unwrap();

        match &response[0] {
            HueApiResponse::Success { success } => {
                assert_eq!(success.username, "token-123");
                assert_eq!(success.clientkey.as_deref(), Some("client-key"));
            }
            HueApiResponse::Error { .. } => panic!("expected success payload"),
        }
    }

    #[test]
    fn converts_raw_lights_to_public_models() {
        let raw = r#"{
            "1": {
                "name": "Kitchen",
                "type": "Extended color light",
                "modelid": "LCT015",
                "state": {
                    "on": true,
                    "bri": 200,
                    "sat": 150,
                    "hue": 10000,
                    "reachable": true
                }
            }
        }"#;

        let response: RawLightsResponse = serde_json::from_str(raw).unwrap();
        let lights: Vec<Light> = response.into_iter().map(Light::from).collect();

        assert_eq!(lights[0].id, "1");
        assert_eq!(lights[0].name, "Kitchen");
        assert_eq!(lights[0].brightness, Some(200));
        assert_eq!(
            lights[0].light_type.as_deref(),
            Some("Extended color light")
        );
    }

    #[test]
    fn serializes_only_requested_state_fields() {
        let update = LightStateUpdate {
            on: Some(true),
            brightness: Some(128),
            saturation: None,
            hue: Some(45000),
            transition_time: None,
        };

        let payload = update.to_payload();
        assert!(!payload.is_empty());

        let encoded = serde_json::to_value(payload).unwrap();
        assert_eq!(encoded["on"], true);
        assert_eq!(encoded["bri"], 128);
        assert_eq!(encoded["hue"], 45000);
        assert!(encoded.get("sat").is_none());
        assert!(encoded.get("transitiontime").is_none());
    }

    #[test]
    fn recognizes_empty_state_payload() {
        let payload = HueLightStatePayload {
            on: None,
            brightness: None,
            saturation: None,
            hue: None,
            transition_time: None,
        };

        assert!(payload.is_empty());
    }

    #[test]
    fn converts_raw_scenes_to_public_models() {
        let raw = r#"{
            "abcd1234": {
                "name": "Evening",
                "group": "2",
                "lights": ["1", "3", "5"],
                "type": "GroupScene",
                "recycle": false
            }
        }"#;

        let response: RawScenesResponse = serde_json::from_str(raw).unwrap();
        let scenes: Vec<Scene> = response.into_iter().map(Scene::from).collect();

        assert_eq!(scenes[0].id, "abcd1234");
        assert_eq!(scenes[0].name, "Evening");
        assert_eq!(scenes[0].group_id.as_deref(), Some("2"));
        assert_eq!(scenes[0].light_count, 3);
    }

    #[test]
    fn converts_raw_sensors_to_public_models() {
        let raw = r#"{
            "12": {
                "name": "Hall motion",
                "type": "ZLLPresence",
                "modelid": "SML001",
                "state": {
                    "presence": true,
                    "lastupdated": "2026-04-04T14:00:00"
                },
                "config": {
                    "reachable": true,
                    "battery": 87
                }
            }
        }"#;

        let response: RawSensorsResponse = serde_json::from_str(raw).unwrap();
        let sensors: Vec<Sensor> = response.into_iter().map(Sensor::from).collect();

        assert_eq!(sensors.len(), 1);
        assert_eq!(sensors[0].name, "Hall motion");
        assert_eq!(sensors[0].sensor_type.as_deref(), Some("ZLLPresence"));
        assert_eq!(sensors[0].battery, Some(87));
        assert_eq!(sensors[0].summary.as_deref(), Some("Motion detected"));
    }

    #[test]
    fn filters_room_and_zone_groups() {
        let raw = r#"{
            "1": {
                "name": "Living Room",
                "lights": ["1", "2"],
                "type": "Room"
            },
            "2": {
                "name": "Downstairs",
                "lights": ["1", "4"],
                "type": "Zone"
            },
            "3": {
                "name": "Unsupported",
                "lights": ["8"],
                "type": "LightGroup"
            }
        }"#;

        let response: RawGroupsResponse = serde_json::from_str(raw).unwrap();
        let groups: Vec<Group> = response
            .into_iter()
            .filter_map(|entry| Group::try_from(entry).ok())
            .collect();

        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].kind, GroupKind::Room);
        assert_eq!(groups[1].kind, GroupKind::Zone);
    }

    #[test]
    fn converts_raw_automations_to_public_models() {
        let raw = r#"{
            "data": [
                {
                    "id": "automation-1",
                    "type": "behavior_instance",
                    "metadata": { "name": "Morning lights" },
                    "enabled": true,
                    "script_id": { "rid": "script-1", "rtype": "behavior_script" }
                },
                {
                    "id": "automation-2",
                    "metadata": { "name": "Night routine" }
                }
            ]
        }"#;

        let response: ClipV2ListResponse<RawAutomation> = serde_json::from_str(raw).unwrap();
        let automations: Vec<Automation> =
            response.data.into_iter().map(Automation::from).collect();

        assert_eq!(automations.len(), 2);
        assert_eq!(automations[0].name, "Morning lights");
        assert_eq!(automations[0].enabled, Some(true));
        assert_eq!(
            automations[0].automation_type.as_deref(),
            Some("behavior_instance")
        );
        assert_eq!(automations[0].script_id.as_deref(), Some("script-1"));
        assert_eq!(automations[1].enabled, None);
    }

    #[test]
    fn converts_entertainment_area_members_to_light_ids() {
        let raw = r#"{
            "data": [
                {
                    "id": "area-1",
                    "metadata": { "name": "Lounge" },
                    "configuration_type": "music",
                    "status": "inactive",
                    "channels": [
                        {
                            "channel_id": 1,
                            "position": { "x": 0.0, "y": 0.0, "z": 0.0 },
                            "members": [
                                { "service": { "rid": "12", "rtype": "light" } },
                                { "service": { "rid": "alpha", "rtype": "device" } }
                            ]
                        },
                        {
                            "channel_id": 2,
                            "position": { "x": 1.0, "y": 0.0, "z": 0.0 },
                            "members": [
                                { "service": { "rid": "7", "rtype": "light" } },
                                { "service": { "rid": "12", "rtype": "light" } }
                            ]
                        }
                    ]
                }
            ]
        }"#;

        let response: ClipV2ListResponse<RawEntertainmentArea> = serde_json::from_str(raw).unwrap();
        let area = EntertainmentArea::from(response.data.into_iter().next().unwrap());

        assert_eq!(area.id, "area-1");
        assert_eq!(area.channels.len(), 2);
        assert_eq!(area.light_ids, vec!["12".to_string(), "7".to_string()]);
    }

    #[test]
    fn converts_entertainment_area_light_services_and_locations_to_light_ids() {
        let raw = r#"{
            "data": [
                {
                    "id": "area-2",
                    "metadata": { "name": "Whole house" },
                    "channels": [],
                    "light_services": [
                        { "rid": "rid-1", "rtype": "entertainment" },
                        { "rid": "rid-2", "rtype": "light" }
                    ],
                    "locations": {
                        "rid-3": { "x": 0.0, "y": 0.0, "z": 0.0 }
                    }
                }
            ]
        }"#;

        let response: ClipV2ListResponse<RawEntertainmentArea> = serde_json::from_str(raw).unwrap();
        let area = EntertainmentArea::from(response.data.into_iter().next().unwrap());

        assert_eq!(area.id, "area-2");
        assert_eq!(
            area.light_ids,
            vec![
                "rid-1".to_string(),
                "rid-2".to_string(),
                "rid-3".to_string()
            ]
        );
    }
}
