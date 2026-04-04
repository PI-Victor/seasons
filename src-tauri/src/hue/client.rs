use crate::hue::config::HueBridgeConfig;
use crate::hue::error::HueError;
use crate::hue::models::{
    Automation, ClipV2ListResponse, CreateUserSuccessPayload, DiscoveredBridge, EntertainmentArea,
    Group, HueApiResponse, Light, LightStateUpdate, RawEntertainmentArea, RawGroupsResponse,
    RawLightsResponse, RawSceneCreateSuccess, RawSceneDetailResponse, RawScenesResponse,
    RawSensorsResponse, RawStateChangeSuccess, RegisteredApp, Scene, Sensor,
};
use reqwest::Client;
use serde_json::Value;
use tracing::{debug, info};

const HUE_APPLICATION_KEY_HEADER: &str = "hue-application-key";
const HUE_APPLICATION_ID_HEADER: &str = "hue-application-id";

pub struct HueBridgeClient {
    http: Client,
    config: HueBridgeConfig,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SceneCatalogEntry {
    id: String,
    name: String,
    group_id: Option<String>,
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
            .json(&serde_json::json!({
                "devicetype": device_type,
                "generateclientkey": true
            }))
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

    pub async fn resolve_application_id(&self) -> Result<String, HueError> {
        debug!(auth_url = %self.config.auth_v1_url(), "requesting hue-application-id");
        let response = self
            .http
            .get(self.config.auth_v1_url())
            .header(HUE_APPLICATION_KEY_HEADER, self.config.application_key()?)
            .send()
            .await?
            .error_for_status()?;

        let application_id = response
            .headers()
            .get(HUE_APPLICATION_ID_HEADER)
            .and_then(|value| value.to_str().ok())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or(HueError::UnexpectedResponse(
                "the bridge did not return a hue-application-id header",
            ))?;

        Ok(application_id.to_string())
    }

    pub async fn list_entertainment_areas(&self) -> Result<Vec<EntertainmentArea>, HueError> {
        debug!(clip_v2_url = %self.config.clip_v2_base_url(), "listing Hue entertainment areas");
        let endpoint = format!(
            "{}/resource/entertainment_configuration",
            self.config.clip_v2_base_url()
        );

        let body = self
            .http
            .get(endpoint)
            .header(HUE_APPLICATION_KEY_HEADER, self.config.application_key()?)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let mut areas = serde_json::from_str::<ClipV2ListResponse<RawEntertainmentArea>>(&body)
            .map_err(|_| extract_api_error(&body))?
            .data
            .into_iter()
            .map(EntertainmentArea::from)
            .collect::<Vec<_>>();
        areas.sort_by(|left, right| left.name.cmp(&right.name).then(left.id.cmp(&right.id)));
        info!(count = areas.len(), "loaded Hue entertainment areas");
        Ok(areas)
    }

    pub async fn list_automations(&self) -> Result<Vec<Automation>, HueError> {
        debug!(clip_v2_url = %self.config.clip_v2_base_url(), "listing Hue automations");
        let endpoint = format!(
            "{}/resource/behavior_instance",
            self.config.clip_v2_base_url()
        );

        let body = self
            .http
            .get(endpoint)
            .header(HUE_APPLICATION_KEY_HEADER, self.config.application_key()?)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let value = serde_json::from_str::<Value>(&body).map_err(|_| extract_api_error(&body))?;
        let data = value
            .get("data")
            .and_then(Value::as_array)
            .ok_or_else(|| extract_api_error(&body))?;

        let mut automations = data
            .iter()
            .filter_map(automation_from_value)
            .collect::<Vec<_>>();
        automations.sort_by(|left, right| left.name.cmp(&right.name).then(left.id.cmp(&right.id)));
        Ok(automations)
    }

    pub async fn set_automation_enabled(
        &self,
        automation_id: &str,
        enabled: bool,
    ) -> Result<(), HueError> {
        let automation_id = automation_id.trim();
        if automation_id.is_empty() {
            return Err(HueError::InvalidConfig("automation ID is required"));
        }

        let endpoint = format!(
            "{}/resource/behavior_instance/{automation_id}",
            self.config.clip_v2_base_url()
        );

        let body = self
            .http
            .put(endpoint)
            .header(HUE_APPLICATION_KEY_HEADER, self.config.application_key()?)
            .json(&serde_json::json!({ "enabled": enabled }))
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        ensure_success_body(&body, true, true)
    }

    pub async fn get_automation_detail(
        &self,
        automation_id: &str,
    ) -> Result<crate::hue::models::AutomationDetail, HueError> {
        let automation_id = automation_id.trim();
        if automation_id.is_empty() {
            return Err(HueError::InvalidConfig("automation ID is required"));
        }

        let instance_value = self
            .fetch_clip_v2_resource("behavior_instance", automation_id)
            .await?;
        let automation = automation_from_value(&instance_value).ok_or(
            HueError::UnexpectedResponse("the bridge returned an unsupported automation payload"),
        )?;
        let instance_json = serde_json::to_string_pretty(&instance_value).map_err(|_| {
            HueError::UnexpectedResponse(
                "the bridge returned automation details that could not be formatted",
            )
        })?;

        let script_json = if let Some(script_id) = automation.script_id.as_deref() {
            match self
                .fetch_clip_v2_resource("behavior_script", script_id)
                .await
            {
                Ok(script_value) => serde_json::to_string_pretty(&script_value).ok(),
                Err(_) => None,
            }
        } else {
            None
        };

        Ok(crate::hue::models::AutomationDetail {
            id: automation.id,
            name: automation.name,
            enabled: automation.enabled,
            script_id: automation.script_id,
            instance_json,
            script_json,
        })
    }

    pub async fn start_entertainment_area(&self, area_id: &str) -> Result<(), HueError> {
        info!(area_id, "sending Hue entertainment area start action");
        self.set_entertainment_area_action(area_id, "start").await?;
        self.wait_for_entertainment_area_status(area_id, "active")
            .await
    }

    pub async fn stop_entertainment_area(&self, area_id: &str) -> Result<(), HueError> {
        info!(area_id, "sending Hue entertainment area stop action");
        self.set_entertainment_area_action(area_id, "stop").await?;
        self.wait_for_entertainment_area_status(area_id, "inactive")
            .await
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

        for scene in &mut scenes {
            if let Some(preview) = self.fetch_scene_preview(scene.id.as_str()).await? {
                scene.preview_color_soft = Some(preview.soft);
                scene.preview_color_main = Some(preview.main);
                scene.preview_color_deep = Some(preview.deep);
            }
        }

        scenes.sort_by(|left, right| left.name.cmp(&right.name).then(left.id.cmp(&right.id)));
        Ok(scenes)
    }

    pub async fn list_sensors(&self) -> Result<Vec<Sensor>, HueError> {
        let endpoint = format!("{}/sensors", self.config.authenticated_api_base_url()?);

        let body = self
            .http
            .get(endpoint)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let sensors_response = match serde_json::from_str::<RawSensorsResponse>(&body) {
            Ok(sensors) => sensors,
            Err(_) => return Err(extract_api_error(&body)),
        };

        let mut sensors: Vec<Sensor> = sensors_response.into_iter().map(Sensor::from).collect();
        sensors.sort_by(|left, right| left.name.cmp(&right.name).then(left.id.cmp(&right.id)));
        Ok(sensors)
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
        let existing_scenes = self.fetch_scene_catalog().await?;

        let body = self
            .http
            .post(endpoint)
            .json(&group_scene_create_body(scene_name, group_id))
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let scene_id = match extract_created_scene_id_from_body(&body) {
            Ok(scene_id) => scene_id,
            Err(_) => {
                let refreshed_scenes = self.fetch_scene_catalog().await?;
                recover_created_scene_id(
                    &existing_scenes,
                    &refreshed_scenes,
                    scene_name,
                    Some(group_id),
                )
                .ok_or(HueError::UnexpectedResponse(
                    "the bridge did not report a created scene identifier",
                ))?
            }
        };

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

        let preview = self.fetch_scene_preview(scene_id.as_str()).await?;

        Ok(Scene {
            id: scene_id,
            name: scene_name.to_string(),
            group_id: Some(group_id.to_string()),
            light_count: light_ids.len(),
            scene_type: Some("GroupScene".to_string()),
            preview_color_soft: preview.as_ref().map(|preview| preview.soft.clone()),
            preview_color_main: preview.as_ref().map(|preview| preview.main.clone()),
            preview_color_deep: preview.as_ref().map(|preview| preview.deep.clone()),
        })
    }

    pub async fn delete_scene(&self, scene_id: &str) -> Result<(), HueError> {
        let scene_id = scene_id.trim();
        if scene_id.is_empty() {
            return Err(HueError::InvalidConfig("scene ID is required"));
        }

        let endpoint = format!(
            "{}/scenes/{scene_id}",
            self.config.authenticated_api_base_url()?
        );

        let body = self
            .http
            .delete(endpoint)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        ensure_success_body(&body, true, true)
    }

    async fn fetch_scene_preview(&self, scene_id: &str) -> Result<Option<ScenePreview>, HueError> {
        let endpoint = format!(
            "{}/scenes/{scene_id}",
            self.config.authenticated_api_base_url()?
        );

        let body = self
            .http
            .get(endpoint)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let detail = match serde_json::from_str::<RawSceneDetailResponse>(&body) {
            Ok(detail) => detail,
            Err(_) => return Err(extract_api_error(&body)),
        };

        Ok(ScenePreview::from_lightstates(detail.lightstates.values()))
    }

    async fn fetch_scene_catalog(&self) -> Result<Vec<SceneCatalogEntry>, HueError> {
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

        let mut scenes = scenes_response
            .into_iter()
            .filter_map(|(id, raw)| {
                if raw.recycle {
                    None
                } else {
                    Some(SceneCatalogEntry {
                        id,
                        name: raw.name,
                        group_id: raw.group,
                    })
                }
            })
            .collect::<Vec<_>>();
        scenes.sort_by(|left, right| left.name.cmp(&right.name).then(left.id.cmp(&right.id)));
        Ok(scenes)
    }

    async fn fetch_entertainment_area(&self, area_id: &str) -> Result<EntertainmentArea, HueError> {
        let area_id = area_id.trim();
        if area_id.is_empty() {
            return Err(HueError::InvalidConfig("entertainment area ID is required"));
        }

        let endpoint = format!(
            "{}/resource/entertainment_configuration/{area_id}",
            self.config.clip_v2_base_url()
        );

        let body = self
            .http
            .get(endpoint)
            .header(HUE_APPLICATION_KEY_HEADER, self.config.application_key()?)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let area = serde_json::from_str::<ClipV2ListResponse<RawEntertainmentArea>>(&body)
            .map_err(|_| extract_api_error(&body))?
            .data
            .into_iter()
            .next()
            .ok_or(HueError::UnexpectedResponse(
                "the bridge returned an empty entertainment area response",
            ))?;

        Ok(EntertainmentArea::from(area))
    }

    async fn fetch_clip_v2_resource(
        &self,
        resource_type: &str,
        resource_id: &str,
    ) -> Result<Value, HueError> {
        let endpoint = format!(
            "{}/resource/{resource_type}/{resource_id}",
            self.config.clip_v2_base_url()
        );

        let body = self
            .http
            .get(endpoint)
            .header(HUE_APPLICATION_KEY_HEADER, self.config.application_key()?)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let value = serde_json::from_str::<Value>(&body).map_err(|_| extract_api_error(&body))?;
        value
            .get("data")
            .and_then(Value::as_array)
            .and_then(|items| items.first())
            .cloned()
            .ok_or_else(|| extract_api_error(&body))
    }

    async fn set_entertainment_area_action(
        &self,
        area_id: &str,
        action: &str,
    ) -> Result<(), HueError> {
        let area_id = area_id.trim();
        if area_id.is_empty() {
            return Err(HueError::InvalidConfig("entertainment area ID is required"));
        }

        let endpoint = format!(
            "{}/resource/entertainment_configuration/{area_id}",
            self.config.clip_v2_base_url()
        );

        self.http
            .put(endpoint)
            .header(HUE_APPLICATION_KEY_HEADER, self.config.application_key()?)
            .json(&serde_json::json!({ "action": action }))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    async fn wait_for_entertainment_area_status(
        &self,
        area_id: &str,
        expected_status: &str,
    ) -> Result<(), HueError> {
        for _ in 0..20 {
            let area = self.fetch_entertainment_area(area_id).await?;
            if area.status.eq_ignore_ascii_case(expected_status) {
                return Ok(());
            }

            tokio::time::sleep(std::time::Duration::from_millis(150)).await;
        }

        Err(HueError::UnexpectedResponse(
            "the entertainment area did not reach the requested status in time",
        ))
    }
}

fn group_scene_create_body(scene_name: &str, group_id: &str) -> serde_json::Value {
    serde_json::json!({
        "name": scene_name,
        "group": group_id,
        "type": "GroupScene",
        "recycle": false,
    })
}

struct ScenePreview {
    soft: String,
    main: String,
    deep: String,
}

impl ScenePreview {
    fn from_lightstates<'a, I>(lightstates: I) -> Option<Self>
    where
        I: IntoIterator<Item = &'a crate::hue::models::RawHueSceneLightState>,
    {
        let colors: Vec<(u8, u8, u8)> = lightstates
            .into_iter()
            .filter_map(scene_lightstate_to_rgb)
            .collect();

        if colors.is_empty() {
            return None;
        }

        let main_rgb = average_rgb(&colors);
        let soft_rgb = tint_rgb(main_rgb, 0.34);
        let deep_rgb = shade_rgb(main_rgb, 0.28);
        let soft = rgb_to_css(soft_rgb);
        let main = rgb_to_css(main_rgb);
        let deep = rgb_to_css(deep_rgb);

        Some(Self { soft, main, deep })
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

fn ensure_success_body(
    body: &str,
    empty_ok: bool,
    opaque_success_ok: bool,
) -> Result<(), HueError> {
    if empty_ok && body.trim().is_empty() {
        return Ok(());
    }

    match serde_json::from_str::<Vec<HueApiResponse<RawStateChangeSuccess>>>(body) {
        Ok(responses) => ensure_success_only(responses),
        Err(_) => {
            if let Ok(responses) = serde_json::from_str::<Vec<HueApiResponse<Value>>>(body) {
                for response in responses {
                    if let HueApiResponse::Error { error } = response {
                        return Err(error.into());
                    }
                }
                return Ok(());
            }

            if opaque_success_ok {
                Ok(())
            } else {
                Err(extract_api_error(body))
            }
        }
    }
}

fn extract_created_scene_id_from_body(body: &str) -> Result<String, HueError> {
    let responses = serde_json::from_str::<Vec<HueApiResponse<RawSceneCreateSuccess>>>(body)
        .map_err(|_| extract_api_error(body))?;
    extract_created_scene_id(responses)
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
                        if let Some(scene_id) = scene_id_from_path(&key) {
                            return Ok(scene_id.to_string());
                        }
                    }

                    if let Some(scene_id) = scene_id_from_path(&value) {
                        return Ok(scene_id.to_string());
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

fn scene_id_from_path(value: &str) -> Option<&str> {
    value
        .strip_prefix("/scenes/")
        .filter(|scene_id| !scene_id.is_empty())
}

fn recover_created_scene_id(
    existing_scenes: &[SceneCatalogEntry],
    refreshed_scenes: &[SceneCatalogEntry],
    scene_name: &str,
    group_id: Option<&str>,
) -> Option<String> {
    let existing_ids = existing_scenes
        .iter()
        .map(|scene| scene.id.as_str())
        .collect::<std::collections::HashSet<_>>();

    refreshed_scenes
        .iter()
        .find(|scene| {
            !existing_ids.contains(scene.id.as_str())
                && scene.name == scene_name
                && scene.group_id.as_deref() == group_id
        })
        .map(|scene| scene.id.clone())
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

fn automation_from_value(value: &Value) -> Option<Automation> {
    let id = value.get("id")?.as_str()?.trim();
    if id.is_empty() {
        return None;
    }

    let name = value
        .get("metadata")
        .and_then(|metadata| metadata.get("name"))
        .and_then(Value::as_str)
        .or_else(|| value.get("name").and_then(Value::as_str))
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .unwrap_or(id)
        .to_string();

    let enabled = value.get("enabled").and_then(Value::as_bool);
    let script_id = value
        .get("script_id")
        .and_then(|script| {
            script
                .get("rid")
                .and_then(Value::as_str)
                .or_else(|| script.as_str())
        })
        .map(ToOwned::to_owned);

    Some(Automation {
        id: id.to_string(),
        name,
        enabled,
        script_id,
    })
}

fn compare_light_ids(left: &str, right: &str) -> std::cmp::Ordering {
    match (left.parse::<u32>(), right.parse::<u32>()) {
        (Ok(left_id), Ok(right_id)) => left_id.cmp(&right_id),
        _ => left.cmp(right),
    }
}

fn scene_lightstate_to_rgb(
    lightstate: &crate::hue::models::RawHueSceneLightState,
) -> Option<(u8, u8, u8)> {
    if matches!(lightstate.on, Some(false)) {
        return None;
    }

    let brightness = lightstate.bri?;
    if brightness == 0 {
        return None;
    }

    if let Some([x, y]) = lightstate.xy {
        return xy_brightness_to_rgb(x, y, brightness);
    }

    Some(hue_sat_bri_to_rgb(
        lightstate.hue.unwrap_or(8_400),
        lightstate.sat.unwrap_or(56),
        brightness,
    ))
}

fn average_rgb(colors: &[(u8, u8, u8)]) -> (u8, u8, u8) {
    let count = colors.len().max(1) as u32;
    let (red_sum, green_sum, blue_sum) = colors.iter().fold(
        (0_u32, 0_u32, 0_u32),
        |(red_sum, green_sum, blue_sum), (red, green, blue)| {
            (
                red_sum + u32::from(*red),
                green_sum + u32::from(*green),
                blue_sum + u32::from(*blue),
            )
        },
    );

    (
        (red_sum / count) as u8,
        (green_sum / count) as u8,
        (blue_sum / count) as u8,
    )
}

fn tint_rgb((red, green, blue): (u8, u8, u8), amount: f32) -> (u8, u8, u8) {
    blend_rgb((red, green, blue), (255, 255, 255), amount)
}

fn shade_rgb((red, green, blue): (u8, u8, u8), amount: f32) -> (u8, u8, u8) {
    blend_rgb((red, green, blue), (0, 0, 0), amount)
}

fn blend_rgb(
    (red_a, green_a, blue_a): (u8, u8, u8),
    (red_b, green_b, blue_b): (u8, u8, u8),
    amount: f32,
) -> (u8, u8, u8) {
    let amount = amount.clamp(0.0, 1.0);
    let blend = |left: u8, right: u8| -> u8 {
        (f32::from(left) * (1.0 - amount) + f32::from(right) * amount).round() as u8
    };

    (
        blend(red_a, red_b),
        blend(green_a, green_b),
        blend(blue_a, blue_b),
    )
}

fn rgb_to_css((red, green, blue): (u8, u8, u8)) -> String {
    format!("rgb({red} {green} {blue})")
}

fn xy_brightness_to_rgb(x: f32, y: f32, brightness: u8) -> Option<(u8, u8, u8)> {
    if !(0.0..=1.0).contains(&x) || !(0.0..=1.0).contains(&y) || y <= f32::EPSILON {
        return None;
    }

    let z = 1.0 - x - y;
    if z < 0.0 {
        return None;
    }

    let luminance = f32::from(brightness) / 254.0;
    let x_xyz = (luminance / y) * x;
    let z_xyz = (luminance / y) * z;

    let mut red = x_xyz * 1.656492 - luminance * 0.354851 - z_xyz * 0.255038;
    let mut green = -x_xyz * 0.707196 + luminance * 1.655397 + z_xyz * 0.036152;
    let mut blue = x_xyz * 0.051713 - luminance * 0.121364 + z_xyz * 1.011_53;

    red = red.max(0.0);
    green = green.max(0.0);
    blue = blue.max(0.0);

    let max_channel = red.max(green).max(blue);
    if max_channel > 1.0 {
        red /= max_channel;
        green /= max_channel;
        blue /= max_channel;
    }

    Some((
        gamma_correct(red),
        gamma_correct(green),
        gamma_correct(blue),
    ))
}

fn gamma_correct(value: f32) -> u8 {
    let corrected = if value <= 0.003_130_8 {
        12.92 * value
    } else {
        1.055 * value.powf(1.0 / 2.4) - 0.055
    };

    (corrected.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn hue_sat_bri_to_rgb(hue: u16, saturation: u8, brightness: u8) -> (u8, u8, u8) {
    let hue = f32::from(hue) * 360.0 / 65_535.0;
    let saturation = f32::from(saturation) / 254.0;
    let value = f32::from(brightness) / 254.0;

    hsv_to_rgb_float(hue, saturation, value)
}

fn hsv_to_rgb_float(hue: f32, saturation: f32, value: f32) -> (u8, u8, u8) {
    let hue = wrap_hue(hue);

    if saturation <= f32::EPSILON {
        let channel = (value * 255.0).round() as u8;
        return (channel, channel, channel);
    }

    let chroma = value * saturation;
    let hue_sector = hue / 60.0;
    let secondary = chroma * (1.0 - ((hue_sector % 2.0) - 1.0).abs());
    let match_value = value - chroma;

    let (red, green, blue) = match hue_sector as u8 {
        0 => (chroma, secondary, 0.0),
        1 => (secondary, chroma, 0.0),
        2 => (0.0, chroma, secondary),
        3 => (0.0, secondary, chroma),
        4 => (secondary, 0.0, chroma),
        _ => (chroma, 0.0, secondary),
    };

    (
        ((red + match_value) * 255.0).round() as u8,
        ((green + match_value) * 255.0).round() as u8,
        ((blue + match_value) * 255.0).round() as u8,
    )
}

fn wrap_hue(hue: f32) -> f32 {
    let wrapped = hue % 360.0;
    if wrapped < 0.0 {
        wrapped + 360.0
    } else {
        wrapped
    }
}

#[cfg(test)]
mod tests {
    use super::{
        compare_light_ids, ensure_success_body, ensure_success_only, extract_api_error,
        extract_created_scene_id, extract_created_scene_id_from_body, extract_first_success,
        group_scene_create_body, recover_created_scene_id, scene_id_from_path, SceneCatalogEntry,
        ScenePreview,
    };
    use crate::hue::models::{CreateUserSuccessPayload, HueApiResponse, RawHueSceneLightState};
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
    fn accepts_empty_delete_response_body() {
        assert!(ensure_success_body("", true, true).is_ok());
        assert!(ensure_success_body("   ", true, true).is_ok());
    }

    #[test]
    fn accepts_opaque_delete_success_body() {
        assert!(ensure_success_body("OK", true, true).is_ok());
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

    #[test]
    fn derives_distinct_scene_previews_from_xy_lightstates() {
        let warm = RawHueSceneLightState {
            on: Some(true),
            bri: Some(229),
            xy: Some([0.485, 0.4543]),
            sat: None,
            hue: None,
        };
        let vivid = RawHueSceneLightState {
            on: Some(true),
            bri: Some(128),
            xy: Some([0.2207, 0.083]),
            sat: None,
            hue: None,
        };

        let warm_preview = ScenePreview::from_lightstates([&warm]).unwrap();
        let vivid_preview = ScenePreview::from_lightstates([&vivid]).unwrap();

        assert_ne!(warm_preview.main, vivid_preview.main);
        assert!(warm_preview.main.starts_with("rgb("));
        assert!(vivid_preview.main.starts_with("rgb("));
    }

    #[test]
    fn groupscene_body_uses_group_without_lights_parameter() {
        let body = group_scene_create_body("Quiet Focus", "7");

        assert_eq!(body["name"], "Quiet Focus");
        assert_eq!(body["group"], "7");
        assert_eq!(body["type"], "GroupScene");
        assert_eq!(body["recycle"], false);
        assert!(body.get("lights").is_none());
    }

    #[test]
    fn extracts_scene_id_from_scene_path() {
        assert_eq!(
            scene_id_from_path("/scenes/desk-evening"),
            Some("desk-evening")
        );
        assert_eq!(scene_id_from_path("desk-evening"), None);
        assert_eq!(scene_id_from_path("/scenes/"), None);
    }

    #[test]
    fn created_scene_id_can_come_from_success_key() {
        let responses = vec![HueApiResponse::Success {
            success: HashMap::from([("/scenes/desk-evening".to_string(), "created".to_string())]),
        }];

        let scene_id = extract_created_scene_id(responses).unwrap();
        assert_eq!(scene_id, "desk-evening");
    }

    #[test]
    fn extracts_created_scene_id_from_body_payload() {
        let body = r#"[{"success":{"/scenes/desk-evening":"created"}}]"#;

        let scene_id = extract_created_scene_id_from_body(body).unwrap();
        assert_eq!(scene_id, "desk-evening");
    }

    #[test]
    fn recovers_created_scene_id_from_scene_catalog_diff() {
        let before = vec![SceneCatalogEntry {
            id: "old".to_string(),
            name: "Read".to_string(),
            group_id: Some("7".to_string()),
        }];
        let after = vec![
            SceneCatalogEntry {
                id: "old".to_string(),
                name: "Read".to_string(),
                group_id: Some("7".to_string()),
            },
            SceneCatalogEntry {
                id: "new-scene-id".to_string(),
                name: "Read".to_string(),
                group_id: Some("8".to_string()),
            },
            SceneCatalogEntry {
                id: "fresh".to_string(),
                name: "Read".to_string(),
                group_id: Some("7".to_string()),
            },
        ];

        let scene_id = recover_created_scene_id(&before, &after, "Read", Some("7")).unwrap();
        assert_eq!(scene_id, "fresh");
    }
}
