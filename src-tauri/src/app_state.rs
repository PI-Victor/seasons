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

use crate::hue::{AudioSyncColorPalette, AudioSyncSpeedMode, BridgeConnection};
use crate::ollama::OllamaSettings;
use crate::theme::ThemePreference;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const APP_DIR_NAME: &str = "seasons";
const CONFIG_FILE_NAME: &str = "config.json";
const DATA_FILE_NAME: &str = "state.json";

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SaveRoomOrderRequest {
    pub connection: BridgeConnection,
    pub room_ids: Vec<String>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
struct AppConfigFile {
    last_connection: Option<BridgeConnection>,
    theme_preference: ThemePreference,
    audio_sync: AudioSyncPreferences,
    ollama: OllamaSettings,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
struct AppDataFile {
    room_orders: BTreeMap<String, Vec<String>>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)]
#[serde(rename_all = "camelCase")]
pub struct AudioSyncPreferences {
    pub selected_entertainment_area_id: Option<String>,
    pub selected_pipewire_target_object: Option<String>,
    pub selected_sync_speed_mode: AudioSyncSpeedMode,
    pub selected_sync_color_palette: AudioSyncColorPalette,
}

pub fn load_bridge_connection() -> Result<Option<BridgeConnection>, String> {
    let config = read_json::<AppConfigFile>(&config_file_path()?)?;
    Ok(config.last_connection)
}

pub fn save_bridge_connection(connection: &BridgeConnection) -> Result<(), String> {
    let mut config = read_json::<AppConfigFile>(&config_file_path()?)?;
    config.last_connection = Some(connection.clone());
    write_json(&config_file_path()?, &config)
}

pub fn clear_bridge_connection() -> Result<(), String> {
    let mut config = read_json::<AppConfigFile>(&config_file_path()?)?;
    config.last_connection = None;
    write_json(&config_file_path()?, &config)
}

pub fn load_theme_preference() -> Result<ThemePreference, String> {
    let config = read_json::<AppConfigFile>(&config_file_path()?)?;
    Ok(config.theme_preference)
}

pub fn save_theme_preference(preference: &ThemePreference) -> Result<(), String> {
    let mut config = read_json::<AppConfigFile>(&config_file_path()?)?;
    config.theme_preference = preference.clone();
    write_json(&config_file_path()?, &config)
}

pub fn load_audio_sync_preferences() -> Result<AudioSyncPreferences, String> {
    let config = read_json::<AppConfigFile>(&config_file_path()?)?;
    Ok(config.audio_sync)
}

pub fn save_audio_sync_preferences(preferences: &AudioSyncPreferences) -> Result<(), String> {
    let mut config = read_json::<AppConfigFile>(&config_file_path()?)?;
    config.audio_sync = preferences.clone();
    write_json(&config_file_path()?, &config)
}

pub fn load_ollama_settings() -> Result<OllamaSettings, String> {
    let config = read_json::<AppConfigFile>(&config_file_path()?)?;
    Ok(config.ollama)
}

pub fn save_ollama_settings(settings: &OllamaSettings) -> Result<(), String> {
    let mut config = read_json::<AppConfigFile>(&config_file_path()?)?;
    config.ollama = settings.clone();
    write_json(&config_file_path()?, &config)
}

pub fn load_room_order(connection: &BridgeConnection) -> Result<Vec<String>, String> {
    let data = read_json::<AppDataFile>(&data_file_path()?)?;
    Ok(data
        .room_orders
        .get(&room_order_key(connection))
        .cloned()
        .unwrap_or_default())
}

pub fn save_room_order(request: &SaveRoomOrderRequest) -> Result<(), String> {
    let mut data = read_json::<AppDataFile>(&data_file_path()?)?;
    data.room_orders.insert(
        room_order_key(&request.connection),
        request.room_ids.clone(),
    );
    write_json(&data_file_path()?, &data)
}

pub fn clear_room_order(connection: &BridgeConnection) -> Result<(), String> {
    let mut data = read_json::<AppDataFile>(&data_file_path()?)?;
    data.room_orders.remove(&room_order_key(connection));
    write_json(&data_file_path()?, &data)
}

fn config_file_path() -> Result<PathBuf, String> {
    Ok(config_dir()?.join(CONFIG_FILE_NAME))
}

fn data_file_path() -> Result<PathBuf, String> {
    Ok(data_dir()?.join(DATA_FILE_NAME))
}

fn config_dir() -> Result<PathBuf, String> {
    resolve_xdg_dir("XDG_CONFIG_HOME", &[".config", APP_DIR_NAME])
}

fn data_dir() -> Result<PathBuf, String> {
    resolve_xdg_dir("XDG_DATA_HOME", &[".local", "share", APP_DIR_NAME])
}

fn resolve_xdg_dir(env_key: &str, fallback_segments: &[&str]) -> Result<PathBuf, String> {
    if let Some(path) = std::env::var_os(env_key).filter(|value| !value.is_empty()) {
        return Ok(PathBuf::from(path).join(APP_DIR_NAME));
    }

    let Some(home) = std::env::var_os("HOME").filter(|value| !value.is_empty()) else {
        return Err(format!("{env_key} is not set and HOME is unavailable"));
    };

    let mut path = PathBuf::from(home);
    for segment in fallback_segments {
        path.push(segment);
    }
    Ok(path)
}

fn read_json<T>(path: &Path) -> Result<T, String>
where
    T: for<'de> Deserialize<'de> + Default,
{
    if !path.exists() {
        return Ok(T::default());
    }

    let contents = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;

    serde_json::from_str(&contents)
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))
}

fn write_json<T>(path: &Path, value: &T) -> Result<(), String>
where
    T: Serialize,
{
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
    }

    let contents = serde_json::to_string_pretty(value)
        .map_err(|error| format!("failed to serialize {}: {error}", path.display()))?;
    fs::write(path, contents)
        .map_err(|error| format!("failed to write {}: {error}", path.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let permissions = fs::Permissions::from_mode(0o600);
        fs::set_permissions(path, permissions).map_err(|error| {
            format!(
                "failed to update permissions for {}: {error}",
                path.display()
            )
        })?;
    }

    Ok(())
}

fn room_order_key(connection: &BridgeConnection) -> String {
    format!(
        "{}::{}",
        connection.bridge_ip.trim(),
        connection.username.trim()
    )
}

#[cfg(test)]
mod tests {
    use super::{room_order_key, AppConfigFile, AppDataFile, AudioSyncPreferences};
    use crate::hue::{AudioSyncColorPalette, AudioSyncSpeedMode, BridgeConnection};
    use crate::ollama::OllamaSettings;
    use crate::theme::{ThemeMode, ThemePalette, ThemePreference};

    #[test]
    fn room_order_key_uses_bridge_and_username() {
        let connection = BridgeConnection {
            bridge_ip: "172.16.0.10".to_string(),
            username: "user-token".to_string(),
            client_key: None,
            application_id: None,
        };

        assert_eq!(room_order_key(&connection), "172.16.0.10::user-token");
    }

    #[test]
    fn state_files_default_to_empty() {
        let config = AppConfigFile::default();
        let data = AppDataFile::default();

        assert!(config.last_connection.is_none());
        assert_eq!(
            config.theme_preference,
            ThemePreference {
                palette: ThemePalette::RosePine,
                mode: ThemeMode::System,
            }
        );
        assert_eq!(
            config.audio_sync,
            AudioSyncPreferences {
                selected_entertainment_area_id: None,
                selected_pipewire_target_object: None,
                selected_sync_speed_mode: AudioSyncSpeedMode::Medium,
                selected_sync_color_palette: AudioSyncColorPalette::CurrentRoom,
            }
        );
        assert_eq!(config.ollama, OllamaSettings::default());
        assert!(data.room_orders.is_empty());
    }
}
