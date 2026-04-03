use crate::hue::config::HueBridgeConfig;
use crate::hue::error::HueError;
use crate::hue::models::{
    CreateUserSuccessPayload, DiscoveredBridge, Group, HueApiResponse, Light, LightStateUpdate,
    RawGroupsResponse, RawLightsResponse, RawSceneCreateSuccess, RawScenesResponse,
    RawStateChangeSuccess, RegisteredApp, Scene,
};
use reqwest::Client;
use serde_json::Value;

pub struct HueBridgeClient {
    http: Client,
    config: HueBridgeConfig,
}

impl HueBridgeClient {
    pub fn new(config: HueBridgeConfig) -> Result<Self, HueError> {
        let http = Client::builder()
            .danger_accept_invalid_certs(true)
            .build()?;

        Ok(Self { http, config })
    }

    pub async fn discover_bridges() -> Result<Vec<DiscoveredBridge>, HueError> {
        let response = Client::new()
            .get("https://discovery.meethue.com/")
            .send()
            .await?
            .error_for_status()?;

        response
            .json::<Vec<DiscoveredBridge>>()
            .await
            .map_err(Into::into)
    }

    pub async fn create_user(&self, device_type: &str) -> Result<RegisteredApp, HueError> {
        let device_type = device_type.trim();
        if device_type.is_empty() {
            return Err(HueError::InvalidConfig("device type is required"));
        }

        let response = self
            .http
            .post(self.config.api_base_url())
            .json(&serde_json::json!({ "devicetype": device_type }))
            .send()
            .await?
            .error_for_status()?;

        let created = extract_first_success::<CreateUserSuccessPayload>(
            response
                .json::<Vec<HueApiResponse<CreateUserSuccessPayload>>>()
                .await?,
        )?;

        Ok(RegisteredApp {
            username: created.username,
            client_key: created.clientkey,
        })
    }

    pub async fn list_lights(&self) -> Result<Vec<Light>, HueError> {
        let endpoint = format!("{}/lights", self.config.authenticated_api_base_url()?);

        let body = self
            .http
            .get(endpoint)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let lights_response = match serde_json::from_str::<RawLightsResponse>(&body) {
            Ok(lights) => lights,
            Err(_) => return Err(extract_api_error(&body)),
        };

        let mut lights: Vec<Light> = lights_response.into_iter().map(Light::from).collect();
        lights.sort_by(|left, right| compare_light_ids(&left.id, &right.id));
        Ok(lights)
    }

    pub async fn list_scenes(&self) -> Result<Vec<Scene>, HueError> {
        let endpoint = format!("{}/scenes", self.config.authenticated_api_base_url()?);

        let body = self
            .http
            .get(endpoint)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let scenes_response = match serde_json::from_str::<RawScenesResponse>(&body) {
            Ok(scenes) => scenes,
            Err(_) => return Err(extract_api_error(&body)),
        };

        let mut scenes: Vec<Scene> = scenes_response
            .into_iter()
            .filter_map(|(id, raw)| {
                if raw.recycle {
                    None
                } else {
                    Some(Scene::from((id, raw)))
                }
            })
            .collect();

        scenes.sort_by(|left, right| left.name.cmp(&right.name).then(left.id.cmp(&right.id)));
        Ok(scenes)
    }

    pub async fn list_groups(&self) -> Result<Vec<Group>, HueError> {
        let endpoint = format!("{}/groups", self.config.authenticated_api_base_url()?);

        let body = self
            .http
            .get(endpoint)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let groups_response = match serde_json::from_str::<RawGroupsResponse>(&body) {
            Ok(groups) => groups,
            Err(_) => return Err(extract_api_error(&body)),
        };

        let mut groups: Vec<Group> = groups_response
            .into_iter()
            .filter_map(|entry| Group::try_from(entry).ok())
            .collect();

        groups.sort_by(|left, right| left.name.cmp(&right.name).then(left.id.cmp(&right.id)));
        Ok(groups)
    }

    pub async fn set_light_state(
        &self,
        light_id: &str,
        state: &LightStateUpdate,
    ) -> Result<(), HueError> {
        let light_id = light_id.trim();
        if light_id.is_empty() {
            return Err(HueError::InvalidConfig("light ID is required"));
        }

        let payload = state.to_payload();
        if payload.is_empty() {
            return Err(HueError::InvalidConfig(
                "at least one light state field is required",
            ));
        }

        let endpoint = format!(
            "{}/lights/{light_id}/state",
            self.config.authenticated_api_base_url()?
        );

        let response = self
            .http
            .put(endpoint)
            .json(&payload)
            .send()
            .await?
            .error_for_status()?;

        ensure_success_only(
            response
                .json::<Vec<HueApiResponse<RawStateChangeSuccess>>>()
                .await?,
        )
    }

    pub async fn activate_scene(
        &self,
        scene_id: &str,
        group_id: Option<&str>,
    ) -> Result<(), HueError> {
        let scene_id = scene_id.trim();
        if scene_id.is_empty() {
            return Err(HueError::InvalidConfig("scene ID is required"));
        }

        let endpoint = format!(
            "{}/groups/{}/action",
            self.config.authenticated_api_base_url()?,
            group_id.unwrap_or("0")
        );

        let response = self
            .http
            .put(endpoint)
            .json(&serde_json::json!({ "scene": scene_id }))
            .send()
            .await?
            .error_for_status()?;

        ensure_success_only(
            response
                .json::<Vec<HueApiResponse<RawStateChangeSuccess>>>()
                .await?,
        )
    }

    pub async fn create_scene(
        &self,
        group_id: &str,
        scene_name: &str,
        light_ids: &[String],
    ) -> Result<Scene, HueError> {
        let group_id = group_id.trim();
        if group_id.is_empty() {
            return Err(HueError::InvalidConfig("group ID is required"));
        }

        let scene_name = scene_name.trim();
        if scene_name.is_empty() {
            return Err(HueError::InvalidConfig("scene name is required"));
        }

        if light_ids.is_empty() {
            return Err(HueError::InvalidConfig(
                "at least one light is required to create a scene",
            ));
        }

        let endpoint = format!("{}/scenes", self.config.authenticated_api_base_url()?);

        let response = self
            .http
            .post(endpoint)
            .json(&serde_json::json!({
                "name": scene_name,
                "group": group_id,
                "lights": light_ids,
                "type": "GroupScene",
                "recycle": false,
            }))
            .send()
            .await?
            .error_for_status()?;

        let scene_id = extract_created_scene_id(
            response
                .json::<Vec<HueApiResponse<RawSceneCreateSuccess>>>()
                .await?,
        )?;

        let capture_endpoint = format!(
            "{}/scenes/{scene_id}",
            self.config.authenticated_api_base_url()?
        );

        let capture_response = self
            .http
            .put(capture_endpoint)
            .json(&serde_json::json!({ "storelightstate": true }))
            .send()
            .await?
            .error_for_status()?;

        ensure_success_only(
            capture_response
                .json::<Vec<HueApiResponse<RawStateChangeSuccess>>>()
                .await?,
        )?;

        Ok(Scene {
            id: scene_id,
            name: scene_name.to_string(),
            group_id: Some(group_id.to_string()),
            light_count: light_ids.len(),
            scene_type: Some("GroupScene".to_string()),
        })
    }
}

fn extract_first_success<T>(responses: Vec<HueApiResponse<T>>) -> Result<T, HueError> {
    if let Some(response) = responses.into_iter().next() {
        return match response {
            HueApiResponse::Success { success } => Ok(success),
            HueApiResponse::Error { error } => Err(error.into()),
        };
    }

    Err(HueError::UnexpectedResponse(
        "the bridge returned an empty success payload",
    ))
}

fn ensure_success_only(
    responses: Vec<HueApiResponse<RawStateChangeSuccess>>,
) -> Result<(), HueError> {
    if responses.is_empty() {
        return Err(HueError::UnexpectedResponse(
            "the bridge returned an empty state update response",
        ));
    }

    for response in responses {
        if let HueApiResponse::Error { error } = response {
            return Err(error.into());
        }
    }

    Ok(())
}

fn extract_created_scene_id(
    responses: Vec<HueApiResponse<RawSceneCreateSuccess>>,
) -> Result<String, HueError> {
    if responses.is_empty() {
        return Err(HueError::UnexpectedResponse(
            "the bridge returned an empty scene creation response",
        ));
    }

    for response in responses {
        match response {
            HueApiResponse::Success { success } => {
                for (key, value) in success {
                    if key.starts_with("/scenes/") {
                        return Ok(value);
                    }
                }
            }
            HueApiResponse::Error { error } => return Err(error.into()),
        }
    }

    Err(HueError::UnexpectedResponse(
        "the bridge did not report a created scene identifier",
    ))
}

fn extract_api_error(body: &str) -> HueError {
    match serde_json::from_str::<Vec<HueApiResponse<Value>>>(body) {
        Ok(responses) => {
            for response in responses {
                if let HueApiResponse::Error { error } = response {
                    return error.into();
                }
            }

            HueError::UnexpectedResponse("the bridge returned an unsupported response body")
        }
        Err(_) => HueError::UnexpectedResponse("unable to decode the bridge response body"),
    }
}

fn compare_light_ids(left: &str, right: &str) -> std::cmp::Ordering {
    match (left.parse::<u32>(), right.parse::<u32>()) {
        (Ok(left_id), Ok(right_id)) => left_id.cmp(&right_id),
        _ => left.cmp(right),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        compare_light_ids, ensure_success_only, extract_api_error, extract_created_scene_id,
        extract_first_success,
    };
    use crate::hue::models::{CreateUserSuccessPayload, HueApiResponse};
    use std::collections::HashMap;

    #[test]
    fn prefers_numeric_light_ordering() {
        assert!(compare_light_ids("2", "10").is_lt());
        assert!(compare_light_ids("alpha", "beta").is_lt());
    }

    #[test]
    fn extracts_first_success_value() {
        let responses = vec![HueApiResponse::Success {
            success: CreateUserSuccessPayload {
                username: "token-123".to_string(),
                clientkey: None,
            },
        }];

        let success = extract_first_success(responses).unwrap();
        assert_eq!(success.username, "token-123");
    }

    #[test]
    fn accepts_successful_state_updates() {
        let responses = vec![HueApiResponse::Success {
            success: HashMap::<String, serde_json::Value>::new(),
        }];

        assert!(ensure_success_only(responses).is_ok());
    }

    #[test]
    fn turns_api_error_payloads_into_hue_errors() {
        let body =
            r#"[{"error":{"type":1,"address":"/lights","description":"unauthorized user"}}]"#;

        let error = extract_api_error(body);
        assert_eq!(
            error.to_string(),
            "Hue bridge API error 1 at /lights: unauthorized user"
        );
    }

    #[test]
    fn extracts_created_scene_id() {
        let responses = vec![HueApiResponse::Success {
            success: HashMap::from([(
                "/scenes/desk-quiet-focus".to_string(),
                "desk-quiet-focus".to_string(),
            )]),
        }];

        let scene_id = extract_created_scene_id(responses).unwrap();
        assert_eq!(scene_id, "desk-quiet-focus");
    }
}
