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

pub mod about_panel;
pub mod audio_sync_panel;
pub mod automation_panel;
pub mod bridge_panel;
pub mod command_panel;
pub mod device_cards;
pub mod device_panel;
pub mod light_grid;
pub mod ollama_panel;
pub mod scene_composer;
pub mod status_banner;
pub mod theme_panel;

pub use about_panel::AboutPanel;
pub use audio_sync_panel::AudioSyncPanel;
pub use automation_panel::AutomationPanel;
pub use bridge_panel::BridgePanel;
pub use command_panel::CommandPanel;
pub use device_cards::DeviceGrid;
pub use device_panel::DevicePanel;
pub use light_grid::LightGrid;
pub use ollama_panel::OllamaPanel;
pub use scene_composer::{SceneComposer, SceneComposerRequest};
pub use status_banner::{NoticeTone, NotificationsPanel, UiNotice, UiToast};
pub use theme_panel::ThemePanel;
