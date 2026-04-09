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

pub mod api;
pub mod models;
pub mod presets;

pub use api::{
    activate_hue_scene, create_hue_scene, create_hue_user, delete_hue_scene, discover_hue_bridges,
    get_hue_audio_sync_preview, get_hue_automation_detail, list_hue_automations,
    list_hue_entertainment_areas, list_hue_groups, list_hue_lights, list_hue_scenes,
    list_hue_sensors, list_pipewire_output_targets, set_hue_automation_enabled,
    set_hue_light_state, start_hue_audio_sync, stop_hue_audio_sync, update_hue_audio_sync,
    update_hue_automation,
};
pub use models::{
    ActivateSceneRequest, AudioSyncColorPalette, AudioSyncPreview, AudioSyncSpeedMode,
    AudioSyncStartRequest, AudioSyncStartResult, AudioSyncUpdateRequest, Automation,
    AutomationConfigEntry, AutomationConfigValue, AutomationDetail, BridgeConnection,
    CreateSceneRequest, CreateUserRequest, DeleteSceneRequest, DiscoveredBridge, EntertainmentArea,
    Group, GroupKind, Light, LightStateUpdate, PipeWireOutputTarget, Scene, Sensor,
    SetAutomationEnabledRequest, SetLightStateRequest, UpdateAutomationRequest,
};
pub use presets::{CuratedScenePreset, curated_room_scenes, preset_light_state};
