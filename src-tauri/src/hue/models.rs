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
pub(crate) type RawGroupsResponse = BTreeMap<String, RawHueGroup>;
pub(crate) type RawScenesResponse = BTreeMap<String, RawHueScene>;
pub(crate) type RawSceneDetailResponse = RawHueSceneDetail;
pub(crate) type RawStateChangeSuccess = HashMap<String, Value>;
pub(crate) type RawSceneCreateSuccess = HashMap<String, String>;

#[cfg(test)]
mod tests {
    use super::{
        CreateUserSuccessPayload, Group, GroupKind, HueApiResponse, HueLightStatePayload, Light,
        LightStateUpdate, RawGroupsResponse, RawLightsResponse, RawScenesResponse, Scene,
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
}
