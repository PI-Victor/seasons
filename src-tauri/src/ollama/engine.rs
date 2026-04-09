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

use crate::app_state::{AudioSyncPreferences, load_audio_sync_preferences, load_ollama_settings};
use crate::audio::{AudioSyncManager, capture};
use crate::hue::models::{GroupKind, LightStateUpdate};
use crate::hue::{
    ActivateSceneRequest, AudioSyncColorPalette, AudioSyncSpeedMode, AudioSyncStartRequest,
    AudioSyncUpdateRequest, Automation, BridgeConnection, EntertainmentArea, Group,
    HueBridgeClient, HueBridgeConfig, Light, PipeWireOutputTarget, Scene, Sensor,
    SetAutomationEnabledRequest,
};
use crate::ollama::models::{
    ExecuteOllamaCommandRequest, ExecuteOllamaCommandResult, OllamaActionOutcome,
    OllamaChatResponse, OllamaSettings, PlannedAction, PlannedResponse,
};
use reqwest::Client;
use serde::Serialize;
use serde_json::{Map, Value, json};
use tokio::time::{Duration, timeout};
use tracing::debug;

const STATUS_OK: &str = "ok";
const STATUS_ERROR: &str = "error";
const STATUS_PARTIAL: &str = "partial";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DeviceContextItem {
    id: String,
    name: String,
    source: String,
    device_type: String,
    reachable: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PlanningContext {
    lights: Vec<Light>,
    rooms: Vec<Group>,
    zones: Vec<Group>,
    scenes: Vec<Scene>,
    entertainment_areas: Vec<EntertainmentArea>,
    automations: Vec<Automation>,
    devices: Vec<DeviceContextItem>,
    pipewire_output_targets: Vec<PipeWireOutputTarget>,
    audio_sync_preferences: AudioSyncPreferences,
    warnings: Vec<String>,
}

pub async fn execute_ollama_command(
    request: ExecuteOllamaCommandRequest,
    audio_sync: &AudioSyncManager,
) -> Result<ExecuteOllamaCommandResult, String> {
    let input = request.input.trim().to_string();
    if input.is_empty() {
        return Err("Command text is required.".to_string());
    }

    let settings = load_ollama_settings()?;
    validate_ollama_settings(&settings)?;

    let bridge_config = HueBridgeConfig::authenticated(
        request.connection.bridge_ip.clone(),
        request.connection.username.clone(),
    )
    .map_err(|error| error.to_string())?;
    let hue_client = HueBridgeClient::new(bridge_config).map_err(|error| error.to_string())?;

    let lights = hue_client
        .list_lights()
        .await
        .map_err(|error| format!("Failed to list bridge lights: {error}"))?;
    let scenes = hue_client
        .list_scenes()
        .await
        .map_err(|error| format!("Failed to list bridge scenes: {error}"))?;
    let groups = hue_client
        .list_groups()
        .await
        .map_err(|error| format!("Failed to list bridge groups: {error}"))?;
    let sensors = hue_client.list_sensors().await.unwrap_or_default();

    let mut warnings = Vec::new();
    let entertainment_areas = match hue_client.list_entertainment_areas().await {
        Ok(areas) => areas,
        Err(error) => {
            warnings.push(format!("Could not list entertainment areas: {error}"));
            Vec::new()
        }
    };
    let automations = match hue_client.list_automations().await {
        Ok(items) => items,
        Err(error) => {
            warnings.push(format!("Could not list automations: {error}"));
            Vec::new()
        }
    };
    let pipewire_output_targets = match capture::list_output_targets() {
        Ok(targets) => targets,
        Err(error) => {
            warnings.push(format!("Could not list audio outputs: {error}"));
            Vec::new()
        }
    };
    let audio_sync_preferences = load_audio_sync_preferences().unwrap_or_default();

    let rooms: Vec<Group> = groups
        .iter()
        .filter(|group| matches!(group.kind, GroupKind::Room))
        .cloned()
        .collect();
    let zones: Vec<Group> = groups
        .iter()
        .filter(|group| matches!(group.kind, GroupKind::Zone))
        .cloned()
        .collect();
    let devices = build_device_context(&lights, &sensors);
    let context = PlanningContext {
        lights: lights.clone(),
        rooms,
        zones,
        scenes: scenes.clone(),
        entertainment_areas: entertainment_areas.clone(),
        automations: automations.clone(),
        devices,
        pipewire_output_targets: pipewire_output_targets.clone(),
        audio_sync_preferences: audio_sync_preferences.clone(),
        warnings,
    };

    let context_json = serde_json::to_string_pretty(&context)
        .map_err(|error| format!("Failed to serialize bridge context for Ollama: {error}"))?;
    let planned = request_action_plan(&settings, &input, &context_json).await?;

    let mut bridge_state_changed = false;
    let mut updated_connection: Option<BridgeConnection> = None;
    let mut actions = Vec::new();
    let mut active_connection = request.connection.clone();

    for action in planned.actions {
        match action {
            PlannedAction::SetLightState {
                light_id,
                on,
                brightness,
                saturation,
                hue,
                transition_time,
            } => {
                let Some(_) = lights.iter().find(|light| light.id == light_id) else {
                    actions.push(action_error(
                        "setLightState",
                        format!("light:{light_id}"),
                        "Unknown light ID for this bridge snapshot.".to_string(),
                    ));
                    continue;
                };

                let state = match build_light_state_update(
                    on,
                    brightness,
                    saturation,
                    hue,
                    transition_time,
                ) {
                    Ok(state) => state,
                    Err(error) => {
                        actions.push(action_error(
                            "setLightState",
                            format!("light:{light_id}"),
                            error,
                        ));
                        continue;
                    }
                };

                match hue_client.set_light_state(&light_id, &state).await {
                    Ok(()) => {
                        bridge_state_changed = true;
                        actions.push(action_ok(
                            "setLightState",
                            format!("light:{light_id}"),
                            "Light state updated.".to_string(),
                        ));
                    }
                    Err(error) => actions.push(action_error(
                        "setLightState",
                        format!("light:{light_id}"),
                        error.to_string(),
                    )),
                }
            }
            PlannedAction::SetGroupState {
                group_id,
                on,
                brightness,
                saturation,
                hue,
                transition_time,
            } => {
                let Some(group) = groups.iter().find(|group| group.id == group_id) else {
                    actions.push(action_error(
                        "setGroupState",
                        format!("group:{group_id}"),
                        "Unknown room/zone ID for this bridge snapshot.".to_string(),
                    ));
                    continue;
                };

                let state = match build_light_state_update(
                    on,
                    brightness,
                    saturation,
                    hue,
                    transition_time,
                ) {
                    Ok(state) => state,
                    Err(error) => {
                        actions.push(action_error(
                            "setGroupState",
                            format!("group:{group_id}"),
                            error,
                        ));
                        continue;
                    }
                };

                if group.light_ids.is_empty() {
                    actions.push(action_error(
                        "setGroupState",
                        format!("group:{group_id}"),
                        "The selected room/zone has no lights.".to_string(),
                    ));
                    continue;
                }

                let mut success_count = 0_usize;
                let mut failures = Vec::new();
                for light_id in &group.light_ids {
                    match hue_client.set_light_state(light_id, &state).await {
                        Ok(()) => success_count += 1,
                        Err(error) => failures.push(format!("{light_id}: {error}")),
                    }
                }

                if success_count == group.light_ids.len() {
                    bridge_state_changed = true;
                    actions.push(action_ok(
                        "setGroupState",
                        format!("group:{group_id}"),
                        format!(
                            "Updated {} light{}",
                            success_count,
                            pluralize(success_count)
                        ),
                    ));
                } else if success_count > 0 {
                    bridge_state_changed = true;
                    actions.push(action_partial(
                        "setGroupState",
                        format!("group:{group_id}"),
                        format!(
                            "Updated {success_count}/{} lights. Failures: {}",
                            group.light_ids.len(),
                            failures.join("; ")
                        ),
                    ));
                } else {
                    actions.push(action_error(
                        "setGroupState",
                        format!("group:{group_id}"),
                        failures.join("; "),
                    ));
                }
            }
            PlannedAction::ActivateScene { scene_id, group_id } => {
                let Some(scene) = scenes.iter().find(|scene| scene.id == scene_id) else {
                    actions.push(action_error(
                        "activateScene",
                        format!("scene:{scene_id}"),
                        "Unknown scene ID for this bridge snapshot.".to_string(),
                    ));
                    continue;
                };

                let resolved_group_id = group_id.or_else(|| scene.group_id.clone());
                let request = ActivateSceneRequest {
                    bridge_ip: active_connection.bridge_ip.clone(),
                    username: active_connection.username.clone(),
                    scene_id: scene_id.clone(),
                    group_id: resolved_group_id.clone(),
                };

                match hue_client
                    .activate_scene(&request.scene_id, request.group_id.as_deref())
                    .await
                {
                    Ok(()) => {
                        bridge_state_changed = true;
                        actions.push(action_ok(
                            "activateScene",
                            format!("scene:{scene_id}"),
                            "Scene activated.".to_string(),
                        ));
                    }
                    Err(error) => actions.push(action_error(
                        "activateScene",
                        format!("scene:{scene_id}"),
                        error.to_string(),
                    )),
                }
            }
            PlannedAction::SetAutomationEnabled {
                automation_id,
                enabled,
            } => {
                let Some(_) = automations
                    .iter()
                    .find(|automation| automation.id == automation_id)
                else {
                    actions.push(action_error(
                        "setAutomationEnabled",
                        format!("automation:{automation_id}"),
                        "Unknown automation ID for this bridge snapshot.".to_string(),
                    ));
                    continue;
                };

                let request = SetAutomationEnabledRequest {
                    connection: active_connection.clone(),
                    automation_id: automation_id.clone(),
                    enabled,
                };

                match hue_client
                    .set_automation_enabled(&request.automation_id, request.enabled)
                    .await
                {
                    Ok(()) => {
                        bridge_state_changed = true;
                        actions.push(action_ok(
                            "setAutomationEnabled",
                            format!("automation:{automation_id}"),
                            if enabled {
                                "Automation enabled.".to_string()
                            } else {
                                "Automation disabled.".to_string()
                            },
                        ));
                    }
                    Err(error) => actions.push(action_error(
                        "setAutomationEnabled",
                        format!("automation:{automation_id}"),
                        error.to_string(),
                    )),
                }
            }
            PlannedAction::StartAudioSync {
                entertainment_area_id,
                pipewire_target_object,
                speed_mode,
                color_palette,
                base_color_hex,
                brightness_ceiling,
            } => {
                let area_id = clean_optional_string(entertainment_area_id)
                    .or_else(|| {
                        audio_sync_preferences
                            .selected_entertainment_area_id
                            .as_ref()
                            .and_then(|value| clean_optional_string(Some(value.to_string())))
                    })
                    .or_else(|| entertainment_areas.first().map(|area| area.id.clone()));

                let Some(entertainment_area_id) = area_id else {
                    actions.push(action_error(
                        "startAudioSync",
                        "audio-sync".to_string(),
                        "No entertainment area is available. Create one in the Hue app first."
                            .to_string(),
                    ));
                    continue;
                };

                let Some(area) = entertainment_areas
                    .iter()
                    .find(|area| area.id == entertainment_area_id)
                    .cloned()
                else {
                    actions.push(action_error(
                        "startAudioSync",
                        "audio-sync".to_string(),
                        "Requested entertainment area is not present in the current snapshot."
                            .to_string(),
                    ));
                    continue;
                };

                let pipewire_target_object =
                    clean_optional_string(pipewire_target_object).or_else(|| {
                        audio_sync_preferences
                            .selected_pipewire_target_object
                            .as_ref()
                            .and_then(|value| clean_optional_string(Some(value.to_string())))
                    });

                let speed_mode = parse_audio_sync_speed_mode(
                    speed_mode.as_deref(),
                    audio_sync_preferences.selected_sync_speed_mode,
                );
                let color_palette = parse_audio_sync_color_palette(
                    color_palette.as_deref(),
                    audio_sync_preferences.selected_sync_color_palette,
                );

                let request = AudioSyncStartRequest {
                    connection: active_connection.clone(),
                    entertainment_area_id: entertainment_area_id.clone(),
                    pipewire_target_object,
                    speed_mode,
                    color_palette,
                    base_color_hex: normalize_hex_color(base_color_hex),
                    brightness_ceiling: brightness_ceiling.map(|value| value.clamp(1, 254) as u8),
                };

                match timeout(
                    Duration::from_secs(6),
                    audio_sync.start(
                        request.connection.clone(),
                        area,
                        request.pipewire_target_object.clone(),
                        request.speed_mode,
                        request.color_palette,
                        request.base_color_hex.clone(),
                        request.brightness_ceiling,
                    ),
                )
                .await
                {
                    Ok(Ok(result)) => {
                        bridge_state_changed = true;
                        active_connection = result.connection.clone();
                        updated_connection = Some(result.connection.clone());
                        actions.push(action_ok(
                            "startAudioSync",
                            "audio-sync".to_string(),
                            format!(
                                "Audio sync started for entertainment area {}.",
                                result.entertainment_area_id
                            ),
                        ));
                    }
                    Ok(Err(error)) => actions.push(action_error(
                        "startAudioSync",
                        "audio-sync".to_string(),
                        error.to_string(),
                    )),
                    Err(_) => actions.push(action_error(
                        "startAudioSync",
                        "audio-sync".to_string(),
                        "Starting audio sync timed out.".to_string(),
                    )),
                }
            }
            PlannedAction::StopAudioSync {} => match audio_sync.stop().await {
                Ok(()) => {
                    bridge_state_changed = true;
                    actions.push(action_ok(
                        "stopAudioSync",
                        "audio-sync".to_string(),
                        "Audio sync stopped.".to_string(),
                    ));
                }
                Err(error) => actions.push(action_error(
                    "stopAudioSync",
                    "audio-sync".to_string(),
                    error.to_string(),
                )),
            },
            PlannedAction::UpdateAudioSync {
                speed_mode,
                color_palette,
                base_color_hex,
                brightness_ceiling,
            } => {
                let request = AudioSyncUpdateRequest {
                    speed_mode: parse_audio_sync_speed_mode(
                        speed_mode.as_deref(),
                        audio_sync_preferences.selected_sync_speed_mode,
                    ),
                    color_palette: parse_audio_sync_color_palette(
                        color_palette.as_deref(),
                        audio_sync_preferences.selected_sync_color_palette,
                    ),
                    base_color_hex: normalize_hex_color(base_color_hex),
                    brightness_ceiling: brightness_ceiling.map(|value| value.clamp(1, 254) as u8),
                };

                match audio_sync.update(
                    request.speed_mode,
                    request.color_palette,
                    request.base_color_hex.clone(),
                    request.brightness_ceiling,
                ) {
                    Ok(()) => {
                        bridge_state_changed = true;
                        actions.push(action_ok(
                            "updateAudioSync",
                            "audio-sync".to_string(),
                            "Audio sync settings updated.".to_string(),
                        ));
                    }
                    Err(error) => actions.push(action_error(
                        "updateAudioSync",
                        "audio-sync".to_string(),
                        error.to_string(),
                    )),
                }
            }
        }
    }

    let assistant_message = if planned.assistant_message.trim().is_empty() {
        if actions.is_empty() {
            "No executable actions were returned by Ollama.".to_string()
        } else {
            "Executed the requested lighting actions.".to_string()
        }
    } else if actions.is_empty() {
        format!(
            "{} (No executable actions were returned.)",
            planned.assistant_message.trim()
        )
    } else {
        planned.assistant_message.trim().to_string()
    };

    Ok(ExecuteOllamaCommandResult {
        assistant_message,
        actions,
        bridge_state_changed,
        updated_connection,
    })
}

pub async fn probe_ollama_connection(settings: OllamaSettings) -> Result<(), String> {
    validate_ollama_settings(&settings)?;
    let endpoint = ollama_tags_endpoint(&settings.base_url)?;
    let timeout_seconds = settings.request_timeout_seconds.clamp(5, 120);
    let client = Client::builder()
        .timeout(Duration::from_secs(timeout_seconds))
        .build()
        .map_err(|error| format!("Failed to create Ollama HTTP client: {error}"))?;

    let mut request = client.get(endpoint);
    if let Some(api_key) = settings
        .api_key
        .as_ref()
        .and_then(|value| clean_optional_string(Some(value.to_string())))
    {
        request = request.bearer_auth(api_key);
    }

    let response = request
        .send()
        .await
        .map_err(|error| format!("Failed to contact Ollama: {error}"))?;
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|error| format!("Failed to read Ollama response: {error}"))?;

    if !status.is_success() {
        return Err(format!(
            "Ollama returned HTTP {}: {}",
            status.as_u16(),
            truncate_for_error(&body)
        ));
    }

    Ok(())
}

async fn request_action_plan(
    settings: &OllamaSettings,
    user_input: &str,
    context_json: &str,
) -> Result<PlannedResponse, String> {
    let initial = request_action_plan_once(
        settings,
        context_json,
        vec![json!({
            "role": "user",
            "content": user_input,
        })],
    )
    .await?;

    if !should_force_execution_retry(user_input, &initial) {
        return Ok(initial);
    }

    debug!(
        user_input,
        assistant_message = initial.assistant_message,
        "ollama returned no actions; attempting planning repair"
    );

    let previous_plan_json = serde_json::to_string(&initial)
        .unwrap_or_else(|_| "{\"assistantMessage\":\"\",\"actions\":[]}".to_string());
    let repaired = request_action_plan_once(
        settings,
        context_json,
        vec![
            json!({
                "role": "user",
                "content": user_input,
            }),
            json!({
                "role": "assistant",
                "content": previous_plan_json,
            }),
            json!({
                "role": "user",
                "content": repair_prompt_for_empty_actions(),
            }),
        ],
    )
    .await?;

    if !should_force_execution_retry(user_input, &repaired) {
        return Ok(repaired);
    }

    debug!(
        user_input,
        assistant_message = repaired.assistant_message,
        "ollama repair still returned no actions; attempting strict execution repair"
    );

    request_action_plan_once(
        settings,
        context_json,
        vec![
            json!({
                "role": "user",
                "content": user_input,
            }),
            json!({
                "role": "assistant",
                "content": previous_plan_json,
            }),
            json!({
                "role": "user",
                "content": strict_repair_prompt_for_empty_actions(),
            }),
        ],
    )
    .await
}

async fn request_action_plan_once(
    settings: &OllamaSettings,
    context_json: &str,
    mut messages: Vec<Value>,
) -> Result<PlannedResponse, String> {
    let endpoint = ollama_chat_endpoint(&settings.base_url)?;
    let timeout_seconds = settings.request_timeout_seconds.clamp(5, 120);
    let client = Client::builder()
        .timeout(Duration::from_secs(timeout_seconds))
        .build()
        .map_err(|error| format!("Failed to create Ollama HTTP client: {error}"))?;

    messages.insert(
        0,
        json!({
            "role": "system",
            "content": planning_prompt(context_json),
        }),
    );

    let request_body = json!({
        "model": settings.model.trim(),
        "stream": false,
        "format": planning_schema(),
        "messages": messages,
    });

    let mut request = client.post(endpoint).json(&request_body);
    if let Some(api_key) = settings
        .api_key
        .as_ref()
        .and_then(|value| clean_optional_string(Some(value.to_string())))
    {
        request = request.bearer_auth(api_key);
    }

    let response = request
        .send()
        .await
        .map_err(|error| format!("Failed to contact Ollama: {error}"))?;
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|error| format!("Failed to read Ollama response: {error}"))?;

    debug!(
        raw_response = %pretty_json_for_log(&body),
        "ollama planning raw response"
    );

    if !status.is_success() {
        return Err(format!(
            "Ollama returned HTTP {}: {}",
            status.as_u16(),
            truncate_for_error(&body)
        ));
    }

    let parsed = serde_json::from_str::<OllamaChatResponse>(&body)
        .map_err(|error| format!("Failed to decode Ollama chat response: {error}"))?;
    debug!(
        message_content = %pretty_json_for_log(&parsed.message.content),
        "ollama planning message content"
    );
    let planned = parse_planned_response(&parsed.message.content)?;
    debug!(
        assistant_message = planned.assistant_message,
        action_count = planned.actions.len(),
        actions = ?planned.actions,
        "ollama planning parsed response"
    );
    Ok(planned)
}

fn repair_prompt_for_empty_actions() -> &'static str {
    "Your previous response returned zero actions.\n\
Return JSON matching the same schema.\n\
If the command changes lights, rooms, zones, scenes, automations, or audio sync, include executable actions with valid IDs from context.\n\
If the command is informational only, keep actions empty and explain in assistantMessage.\n\
Treat imperative or declarative control statements as commands that should produce actions.\n\
assistantMessage must be a human-readable sentence, never a placeholder token such as turnOnLightsResponse.\n\
Do not use targetId/properties wrapper objects. Use the exact action fields from schema (for example groupId/on/brightness).\n\
Do not return prose outside JSON."
}

fn strict_repair_prompt_for_empty_actions() -> &'static str {
    "Your previous response returned zero actions.\n\
Return JSON matching the same schema, and include at least one executable action when the command is controllable.\n\
Only keep actions empty if the user explicitly requested information/listing with no state change.\n\
For turn on/off requests, include on: true/false in the action.\n\
assistantMessage must be plain natural language, never a symbolic label.\n\
Do not paraphrase the command. Output JSON only."
}

fn planning_prompt(context_json: &str) -> String {
    format!(
        "You are the control planner for the Seasons Hue app.\n\
You must return JSON matching the provided schema.\n\
Use IDs from the context exactly. Never invent IDs.\n\
If the user asks what exists (lights, rooms, zones, devices, scenes, entertainment areas, automations), use assistantMessage and keep actions empty.\n\
For color requests, convert color names to Hue values: hue in 0..65535 and saturation in 0..254.\n\
Brightness must be 1..254 when provided.\n\
Use setGroupState for room/zone level power/color changes.\n\
Use startAudioSync/updateAudioSync/stopAudioSync for entertainment sync actions.\n\
For power commands (on/off), always include on: true/false in at least one action.\n\
assistantMessage must be a short natural-language sentence, not a symbolic token.\n\
Never use targetId/properties wrappers. Use only schema fields such as groupId/lightId/on/brightness/saturation/hue/transitionTime.\n\
Do not include fields outside schema.\n\
\nBridge context:\n{context_json}"
    )
}

fn planning_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "assistantMessage": { "type": "string" },
            "actions": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "type": {
                            "type": "string",
                            "enum": [
                                "setLightState",
                                "setGroupState",
                                "activateScene",
                                "setAutomationEnabled",
                                "startAudioSync",
                                "stopAudioSync",
                                "updateAudioSync"
                            ]
                        },
                        "lightId": { "type": "string" },
                        "groupId": { "type": "string" },
                        "sceneId": { "type": "string" },
                        "automationId": { "type": "string" },
                        "enabled": { "type": "boolean" },
                        "on": { "type": "boolean" },
                        "brightness": { "type": "integer", "minimum": 1, "maximum": 254 },
                        "saturation": { "type": "integer", "minimum": 0, "maximum": 254 },
                        "hue": { "type": "integer", "minimum": 0, "maximum": 65535 },
                        "transitionTime": { "type": "integer", "minimum": 0, "maximum": 65535 },
                        "entertainmentAreaId": { "type": "string" },
                        "pipewireTargetObject": { "type": "string" },
                        "speedMode": { "type": "string", "enum": ["slow", "medium", "high"] },
                        "colorPalette": {
                            "type": "string",
                            "enum": [
                                "currentRoom",
                                "sunset",
                                "aurora",
                                "ocean",
                                "rose",
                                "mono",
                                "neonPulse",
                                "synthwave",
                                "prism",
                                "vocalGlow",
                                "fireIce"
                            ]
                        },
                        "baseColorHex": { "type": "string" },
                        "brightnessCeiling": { "type": "integer", "minimum": 1, "maximum": 254 }
                    },
                    "required": ["type"],
                    "additionalProperties": false
                }
            }
        },
        "required": ["assistantMessage", "actions"],
        "additionalProperties": false
    })
}

fn parse_planned_response(content: &str) -> Result<PlannedResponse, String> {
    let trimmed = content.trim_start_matches('\u{feff}').trim();
    if trimmed.is_empty() {
        return Ok(PlannedResponse::default());
    }

    if let Some(parsed) = parse_candidate_payload(trimmed) {
        return Ok(parsed);
    }

    if let Some(stripped) = strip_markdown_json_block(trimmed) {
        if let Some(parsed) = parse_candidate_payload(&stripped) {
            return Ok(parsed);
        }
    }

    if let Some(candidate) = extract_first_json_object(trimmed) {
        if let Some(parsed) = parse_candidate_payload(candidate) {
            return Ok(parsed);
        }
    }

    Err(format!(
        "Ollama did not return valid structured JSON. Raw output: {}",
        truncate_for_error(trimmed)
    ))
}

fn parse_candidate_payload(candidate: &str) -> Option<PlannedResponse> {
    if let Ok(parsed) = serde_json::from_str::<PlannedResponse>(candidate) {
        return Some(parsed);
    }
    if let Ok(value) = serde_json::from_str::<Value>(candidate) {
        if let Ok(parsed) = parse_planned_response_value(value) {
            return Some(parsed);
        }
    }

    let repaired = repair_missing_commas(candidate);
    if repaired != candidate {
        if let Ok(parsed) = serde_json::from_str::<PlannedResponse>(&repaired) {
            return Some(parsed);
        }
        if let Ok(value) = serde_json::from_str::<Value>(&repaired) {
            if let Ok(parsed) = parse_planned_response_value(value) {
                return Some(parsed);
            }
        }
    }

    None
}

fn repair_missing_commas(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut output = String::with_capacity(input.len() + 12);
    let mut in_string = false;
    let mut escaped = false;
    let mut index = 0_usize;

    while index < chars.len() {
        let current = chars[index];
        output.push(current);

        if in_string {
            if escaped {
                escaped = false;
            } else if current == '\\' {
                escaped = true;
            } else if current == '"' {
                in_string = false;
                if should_insert_comma_after_string(&chars, index) {
                    output.push(',');
                }
            }

            index += 1;
            continue;
        }

        if current == '"' {
            in_string = true;
            index += 1;
            continue;
        }

        if is_non_string_value_terminator(current) && should_insert_comma(&chars, index) {
            output.push(',');
        }

        index += 1;
    }

    output
}

fn is_non_string_value_terminator(character: char) -> bool {
    character.is_ascii_digit() || matches!(character, '}' | ']' | 'e' | 'l')
}

fn should_insert_comma_after_string(chars: &[char], index: usize) -> bool {
    let next = next_significant_char(chars, index + 1);
    matches!(next, Some('"'))
}

fn should_insert_comma(chars: &[char], index: usize) -> bool {
    let next = next_significant_char(chars, index + 1);
    matches!(next, Some('"'))
}

fn next_significant_char(chars: &[char], mut index: usize) -> Option<char> {
    while index < chars.len() {
        if !chars[index].is_whitespace() {
            return Some(chars[index]);
        }
        index += 1;
    }
    None
}

fn strip_markdown_json_block(value: &str) -> Option<String> {
    let content = value.trim();
    if !content.starts_with("```") {
        return None;
    }

    let mut lines = content.lines();
    let _ = lines.next()?;
    let mut collected = Vec::new();
    for line in lines {
        if line.trim_start().starts_with("```") {
            break;
        }
        collected.push(line);
    }

    if collected.is_empty() {
        None
    } else {
        Some(collected.join("\n"))
    }
}

fn extract_first_json_object(value: &str) -> Option<&str> {
    let start = value.find('{')?;
    let slice = &value[start..];

    let mut depth = 0_i32;
    let mut in_string = false;
    let mut escaped = false;

    for (index, character) in slice.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_string = false;
            }
            continue;
        }

        match character {
            '"' => in_string = true,
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    let end = start + index + character.len_utf8();
                    return Some(&value[start..end]);
                }
            }
            _ => {}
        }
    }

    None
}

fn parse_planned_response_value(value: Value) -> Result<PlannedResponse, String> {
    let object = value
        .as_object()
        .ok_or_else(|| "Expected planning response object".to_string())?;

    let assistant_message = object
        .get("assistantMessage")
        .or_else(|| object.get("assistant_message"))
        .or_else(|| object.get("message"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim()
        .to_string();

    let mut actions = Vec::new();
    if let Some(actions_value) = object.get("actions") {
        match actions_value {
            Value::Array(items) => {
                for item in items {
                    if let Some(action) = normalize_action_value(item) {
                        if let Ok(parsed_action) = serde_json::from_value::<PlannedAction>(action) {
                            actions.push(parsed_action);
                        }
                    }
                }
            }
            Value::Object(_) => {
                if let Some(action) = normalize_action_value(actions_value) {
                    if let Ok(parsed_action) = serde_json::from_value::<PlannedAction>(action) {
                        actions.push(parsed_action);
                    }
                }
            }
            _ => {}
        }
    }
    if actions.is_empty() {
        if let Some(action_value) = object.get("action") {
            if let Some(action) = normalize_action_value(action_value) {
                if let Ok(parsed_action) = serde_json::from_value::<PlannedAction>(action) {
                    actions.push(parsed_action);
                }
            }
        }
    }

    if assistant_message.is_empty() && actions.is_empty() {
        return Err("No assistant message or executable actions were found".to_string());
    }

    Ok(PlannedResponse {
        assistant_message,
        actions,
    })
}

fn normalize_action_value(value: &Value) -> Option<Value> {
    let source = value.as_object()?;
    let mut normalized = Map::new();

    for (key, item) in source {
        normalized.insert(key.clone(), item.clone());
    }

    normalize_action_key(&mut normalized, "group_id", &["groupId"]);
    normalize_action_key(&mut normalized, "light_id", &["lightId"]);
    normalize_action_key(&mut normalized, "scene_id", &["sceneId"]);
    normalize_action_key(&mut normalized, "automation_id", &["automationId"]);
    normalize_action_key(
        &mut normalized,
        "entertainment_area_id",
        &["entertainmentAreaId"],
    );
    normalize_action_key(
        &mut normalized,
        "pipewire_target_object",
        &["pipewireTargetObject"],
    );
    normalize_action_key(&mut normalized, "speed_mode", &["speedMode"]);
    normalize_action_key(&mut normalized, "color_palette", &["colorPalette"]);
    normalize_action_key(&mut normalized, "base_color_hex", &["baseColorHex"]);
    normalize_action_key(
        &mut normalized,
        "brightness_ceiling",
        &["brightnessCeiling"],
    );
    normalize_action_key(&mut normalized, "transition_time", &["transitionTime"]);

    let raw_type = normalized
        .get("type")
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or_default();
    let normalized_type = match normalize_token(raw_type).as_str() {
        "setlightstate" => "setLightState",
        "setgroupstate" => "setGroupState",
        "activatescene" => "activateScene",
        "setautomationenabled" => "setAutomationEnabled",
        "startaudiosync" => "startAudioSync",
        "stopaudiosync" => "stopAudioSync",
        "updateaudiosync" => "updateAudioSync",
        _ => return None,
    };
    normalized.insert(
        "type".to_string(),
        Value::String(normalized_type.to_string()),
    );

    Some(Value::Object(normalized))
}

fn normalize_action_key(map: &mut Map<String, Value>, canonical: &str, aliases: &[&str]) {
    if map.contains_key(canonical) {
        return;
    }

    for alias in aliases {
        if let Some(value) = map.remove(*alias) {
            map.insert(canonical.to_string(), value);
            return;
        }
    }
}

fn validate_ollama_settings(settings: &OllamaSettings) -> Result<(), String> {
    if settings.base_url.trim().is_empty() {
        return Err("Ollama base URL is required.".to_string());
    }
    if settings.model.trim().is_empty() {
        return Err("Ollama model is required.".to_string());
    }
    Ok(())
}

fn build_device_context(lights: &[Light], sensors: &[Sensor]) -> Vec<DeviceContextItem> {
    let mut devices = Vec::with_capacity(lights.len() + sensors.len());
    for light in lights {
        devices.push(DeviceContextItem {
            id: light.id.clone(),
            name: light.name.clone(),
            source: "light".to_string(),
            device_type: light
                .light_type
                .clone()
                .unwrap_or_else(|| "Light".to_string()),
            reachable: light.reachable,
        });
    }
    for sensor in sensors {
        devices.push(DeviceContextItem {
            id: sensor.id.clone(),
            name: sensor.name.clone(),
            source: "sensor".to_string(),
            device_type: sensor
                .sensor_type
                .clone()
                .unwrap_or_else(|| "Sensor".to_string()),
            reachable: sensor.reachable,
        });
    }
    devices.sort_by(|left, right| left.name.cmp(&right.name).then(left.id.cmp(&right.id)));
    devices
}

fn build_light_state_update(
    mut on: Option<bool>,
    brightness: Option<u16>,
    saturation: Option<u16>,
    hue: Option<u32>,
    transition_time: Option<u16>,
) -> Result<LightStateUpdate, String> {
    if on.is_none() && (brightness.is_some() || saturation.is_some() || hue.is_some()) {
        on = Some(true);
    }

    let state = LightStateUpdate {
        on,
        brightness: brightness.map(|value| value.clamp(1, 254) as u8),
        saturation: saturation.map(|value| value.clamp(0, 254) as u8),
        hue: hue.map(|value| value.clamp(0, 65_535) as u16),
        transition_time,
    };

    if state.on.is_none()
        && state.brightness.is_none()
        && state.saturation.is_none()
        && state.hue.is_none()
        && state.transition_time.is_none()
    {
        return Err("At least one light-state field is required.".to_string());
    }

    Ok(state)
}

fn parse_audio_sync_speed_mode(
    value: Option<&str>,
    fallback: AudioSyncSpeedMode,
) -> AudioSyncSpeedMode {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return fallback;
    };

    match normalize_token(value).as_str() {
        "slow" => AudioSyncSpeedMode::Slow,
        "high" => AudioSyncSpeedMode::High,
        _ => AudioSyncSpeedMode::Medium,
    }
}

fn parse_audio_sync_color_palette(
    value: Option<&str>,
    fallback: AudioSyncColorPalette,
) -> AudioSyncColorPalette {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return fallback;
    };

    match normalize_token(value).as_str() {
        "sunset" => AudioSyncColorPalette::Sunset,
        "aurora" => AudioSyncColorPalette::Aurora,
        "ocean" => AudioSyncColorPalette::Ocean,
        "rose" => AudioSyncColorPalette::Rose,
        "mono" => AudioSyncColorPalette::Mono,
        "neonpulse" | "neonflux" => AudioSyncColorPalette::NeonPulse,
        "synthwave" | "retrowave" => AudioSyncColorPalette::Synthwave,
        "prism" => AudioSyncColorPalette::Prism,
        "vocalglow" | "vocal" => AudioSyncColorPalette::VocalGlow,
        "fireice" => AudioSyncColorPalette::FireIce,
        _ => AudioSyncColorPalette::CurrentRoom,
    }
}

fn normalize_token(value: &str) -> String {
    value
        .chars()
        .filter(|char| char.is_ascii_alphanumeric())
        .flat_map(|char| char.to_lowercase())
        .collect()
}

fn normalize_hex_color(value: Option<String>) -> Option<String> {
    let value = clean_optional_string(value)?;
    let prefixed = if value.starts_with('#') {
        value
    } else {
        format!("#{value}")
    };

    if prefixed.len() != 7 {
        return None;
    }

    if prefixed
        .chars()
        .skip(1)
        .all(|char| char.is_ascii_hexdigit())
    {
        Some(prefixed.to_ascii_lowercase())
    } else {
        None
    }
}

fn clean_optional_string(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn ollama_chat_endpoint(base_url: &str) -> Result<String, String> {
    let trimmed = base_url.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Err("Ollama base URL is required.".to_string());
    }

    if trimmed.ends_with("/api") {
        Ok(format!("{trimmed}/chat"))
    } else {
        Ok(format!("{trimmed}/api/chat"))
    }
}

fn ollama_tags_endpoint(base_url: &str) -> Result<String, String> {
    let trimmed = base_url.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Err("Ollama base URL is required.".to_string());
    }

    if trimmed.ends_with("/api") {
        Ok(format!("{trimmed}/tags"))
    } else {
        Ok(format!("{trimmed}/api/tags"))
    }
}

fn truncate_for_error(value: &str) -> String {
    const LIMIT: usize = 400;
    let trimmed = value.trim();
    if trimmed.len() <= LIMIT {
        trimmed.to_string()
    } else {
        format!("{}…", &trimmed[..LIMIT])
    }
}

fn pretty_json_for_log(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    match serde_json::from_str::<Value>(trimmed) {
        Ok(value) => serde_json::to_string_pretty(&value).unwrap_or_else(|_| trimmed.to_string()),
        Err(_) => trimmed.to_string(),
    }
}

fn action_ok(
    action: impl Into<String>,
    target: impl Into<String>,
    detail: impl Into<String>,
) -> OllamaActionOutcome {
    OllamaActionOutcome {
        action: action.into(),
        target: target.into(),
        status: STATUS_OK.to_string(),
        detail: detail.into(),
    }
}

fn action_partial(
    action: impl Into<String>,
    target: impl Into<String>,
    detail: impl Into<String>,
) -> OllamaActionOutcome {
    OllamaActionOutcome {
        action: action.into(),
        target: target.into(),
        status: STATUS_PARTIAL.to_string(),
        detail: detail.into(),
    }
}

fn action_error(
    action: impl Into<String>,
    target: impl Into<String>,
    detail: impl Into<String>,
) -> OllamaActionOutcome {
    OllamaActionOutcome {
        action: action.into(),
        target: target.into(),
        status: STATUS_ERROR.to_string(),
        detail: detail.into(),
    }
}

fn pluralize(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
}

fn normalize_for_similarity(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character.is_ascii_whitespace() {
                character.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn should_force_execution_retry(user_input: &str, planned: &PlannedResponse) -> bool {
    if !planned.actions.is_empty() {
        return false;
    }

    let assistant_message = planned.assistant_message.trim();
    if assistant_message.is_empty() {
        return true;
    }

    if !is_informative_no_action_message(assistant_message) {
        return true;
    }

    is_paraphrase_like(user_input, assistant_message)
}

fn is_paraphrase_like(input: &str, response: &str) -> bool {
    let normalized_input = normalize_for_similarity(input);
    let normalized_response = normalize_for_similarity(response);
    if normalized_input.is_empty() || normalized_response.is_empty() {
        return false;
    }

    if normalized_input == normalized_response
        || normalized_response.contains(&normalized_input)
        || normalized_input.contains(&normalized_response)
    {
        return true;
    }

    let input_tokens = normalized_input.split_whitespace().collect::<Vec<_>>();
    let response_tokens = normalized_response.split_whitespace().collect::<Vec<_>>();
    if input_tokens.is_empty() || response_tokens.is_empty() {
        return false;
    }

    let shared = response_tokens
        .iter()
        .filter(|token| input_tokens.contains(token))
        .count();
    let overlap = shared as f32 / input_tokens.len().max(response_tokens.len()) as f32;
    overlap >= 0.7
}

fn is_informative_no_action_message(message: &str) -> bool {
    let normalized = normalize_for_similarity(message);
    if normalized.is_empty() {
        return false;
    }

    let token_count = normalized.split_whitespace().count();
    if token_count < 4 || normalized.len() < 20 {
        return false;
    }

    let has_number = message.chars().any(|character| character.is_ascii_digit());
    let has_structure = message.contains(',') || message.contains(':') || message.contains('\n');
    has_number || has_structure || token_count >= 8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_single_action_object_in_actions_field() {
        let payload = json!({
            "assistantMessage": "ok",
            "actions": {
                "type": "setGroupState",
                "groupId": "81",
                "on": false
            }
        });

        let parsed = parse_planned_response_value(payload).expect("payload should parse");
        assert_eq!(parsed.actions.len(), 1);
        match &parsed.actions[0] {
            PlannedAction::SetGroupState { group_id, on, .. } => {
                assert_eq!(group_id, "81");
                assert_eq!(*on, Some(false));
            }
            _ => panic!("expected setGroupState action"),
        }
    }

    #[test]
    fn parses_single_action_from_action_field() {
        let payload = json!({
            "assistantMessage": "ok",
            "action": {
                "type": "setLightState",
                "lightId": "5",
                "on": true
            }
        });

        let parsed = parse_planned_response_value(payload).expect("payload should parse");
        assert_eq!(parsed.actions.len(), 1);
        match &parsed.actions[0] {
            PlannedAction::SetLightState { light_id, on, .. } => {
                assert_eq!(light_id, "5");
                assert_eq!(*on, Some(true));
            }
            _ => panic!("expected setLightState action"),
        }
    }

    #[test]
    fn force_retry_when_response_paraphrases_input() {
        let planned = PlannedResponse {
            assistant_message: "Turning off the light in the living room.".to_string(),
            actions: Vec::new(),
        };

        assert!(should_force_execution_retry(
            "Turning off the light in the living room.",
            &planned
        ));
    }

    #[test]
    fn no_force_retry_when_response_contains_actual_info() {
        let planned = PlannedResponse {
            assistant_message: "There are 12 lights and 4 rooms available on this bridge."
                .to_string(),
            actions: Vec::new(),
        };

        assert!(!should_force_execution_retry("list lights", &planned));
    }

    #[test]
    fn force_retry_for_placeholder_message() {
        let planned = PlannedResponse {
            assistant_message: "turnOnLightsResponse".to_string(),
            actions: Vec::new(),
        };

        assert!(should_force_execution_retry(
            "turn on the living room lights",
            &planned
        ));
    }
}
