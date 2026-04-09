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

use crate::hue::BridgeConnection;
use serde::{Deserialize, Serialize};

const DEFAULT_OLLAMA_BASE_URL: &str = "http://localhost:11434";
const DEFAULT_OLLAMA_MODEL: &str = "qwen2.5:14b-instruct";

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)]
#[serde(rename_all = "camelCase")]
pub struct OllamaSettings {
    pub base_url: String,
    pub model: String,
    #[serde(default)]
    pub api_key: Option<String>,
    pub request_timeout_seconds: u64,
}

impl Default for OllamaSettings {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_OLLAMA_BASE_URL.to_string(),
            model: DEFAULT_OLLAMA_MODEL.to_string(),
            api_key: None,
            request_timeout_seconds: 30,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteOllamaCommandRequest {
    pub input: String,
    pub connection: BridgeConnection,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OllamaActionOutcome {
    pub action: String,
    pub target: String,
    pub status: String,
    pub detail: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteOllamaCommandResult {
    pub assistant_message: String,
    pub actions: Vec<OllamaActionOutcome>,
    pub bridge_state_changed: bool,
    #[serde(default)]
    pub updated_connection: Option<BridgeConnection>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OllamaChatResponse {
    pub message: OllamaChatMessage,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OllamaChatMessage {
    #[serde(default)]
    pub content: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PlannedResponse {
    pub assistant_message: String,
    pub actions: Vec<PlannedAction>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub(crate) enum PlannedAction {
    SetLightState {
        light_id: String,
        #[serde(default)]
        on: Option<bool>,
        #[serde(default)]
        brightness: Option<u16>,
        #[serde(default)]
        saturation: Option<u16>,
        #[serde(default)]
        hue: Option<u32>,
        #[serde(default)]
        transition_time: Option<u16>,
    },
    SetGroupState {
        group_id: String,
        #[serde(default)]
        on: Option<bool>,
        #[serde(default)]
        brightness: Option<u16>,
        #[serde(default)]
        saturation: Option<u16>,
        #[serde(default)]
        hue: Option<u32>,
        #[serde(default)]
        transition_time: Option<u16>,
    },
    ActivateScene {
        scene_id: String,
        #[serde(default)]
        group_id: Option<String>,
    },
    SetAutomationEnabled {
        automation_id: String,
        enabled: bool,
    },
    StartAudioSync {
        #[serde(default)]
        entertainment_area_id: Option<String>,
        #[serde(default)]
        pipewire_target_object: Option<String>,
        #[serde(default)]
        speed_mode: Option<String>,
        #[serde(default)]
        color_palette: Option<String>,
        #[serde(default)]
        base_color_hex: Option<String>,
        #[serde(default)]
        brightness_ceiling: Option<u16>,
    },
    StopAudioSync {},
    UpdateAudioSync {
        #[serde(default)]
        speed_mode: Option<String>,
        #[serde(default)]
        color_palette: Option<String>,
        #[serde(default)]
        base_color_hex: Option<String>,
        #[serde(default)]
        brightness_ceiling: Option<u16>,
    },
}
