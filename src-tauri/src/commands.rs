use crate::app_state::{
    clear_bridge_connection as state_clear_bridge_connection,
    clear_room_order as state_clear_room_order,
    load_audio_sync_preferences as state_load_audio_sync_preferences,
    load_bridge_connection as state_load_bridge_connection,
    load_room_order as state_load_room_order, load_theme_preference as state_load_theme_preference,
    save_audio_sync_preferences as state_save_audio_sync_preferences,
    save_bridge_connection as state_save_bridge_connection,
    save_room_order as state_save_room_order, save_theme_preference as state_save_theme_preference,
    AudioSyncPreferences, SaveRoomOrderRequest,
};
use crate::audio::{capture, AudioSyncManager};
use crate::hue::{
    ActivateSceneRequest, AudioSyncStartRequest, AudioSyncStartResult, AudioSyncUpdateRequest,
    Automation, AutomationDetail, BridgeConnection, CreateSceneRequest, CreateUserRequest,
    DeleteSceneRequest, DiscoveredBridge, EntertainmentArea, Group, HueBridgeClient,
    HueBridgeConfig, Light, PipeWireOutputTarget, RegisteredApp, Scene, Sensor,
    SetAutomationEnabledRequest, SetLightStateRequest, UpdateAutomationRequest,
};
use crate::theme::ThemePreference;
use tauri::{AppHandle, State};
use tokio::time::{timeout, Duration};

#[tauri::command]
pub async fn discover_hue_bridges() -> Result<Vec<DiscoveredBridge>, String> {
    HueBridgeClient::discover_bridges()
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn create_hue_user(request: CreateUserRequest) -> Result<RegisteredApp, String> {
    let CreateUserRequest {
        bridge_ip,
        device_type,
    } = request;

    let config = HueBridgeConfig::new(bridge_ip, None).map_err(|error| error.to_string())?;
    let client = HueBridgeClient::new(config).map_err(|error| error.to_string())?;

    client
        .create_user(&device_type)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn list_hue_lights(connection: BridgeConnection) -> Result<Vec<Light>, String> {
    let BridgeConnection {
        bridge_ip,
        username,
        ..
    } = connection;

    let config =
        HueBridgeConfig::authenticated(bridge_ip, username).map_err(|error| error.to_string())?;
    let client = HueBridgeClient::new(config).map_err(|error| error.to_string())?;

    client
        .list_lights()
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn list_hue_scenes(connection: BridgeConnection) -> Result<Vec<Scene>, String> {
    let BridgeConnection {
        bridge_ip,
        username,
        ..
    } = connection;

    let config =
        HueBridgeConfig::authenticated(bridge_ip, username).map_err(|error| error.to_string())?;
    let client = HueBridgeClient::new(config).map_err(|error| error.to_string())?;

    client
        .list_scenes()
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn list_hue_sensors(connection: BridgeConnection) -> Result<Vec<Sensor>, String> {
    let BridgeConnection {
        bridge_ip,
        username,
        ..
    } = connection;

    let config =
        HueBridgeConfig::authenticated(bridge_ip, username).map_err(|error| error.to_string())?;
    let client = HueBridgeClient::new(config).map_err(|error| error.to_string())?;

    client
        .list_sensors()
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn list_hue_groups(connection: BridgeConnection) -> Result<Vec<Group>, String> {
    let BridgeConnection {
        bridge_ip,
        username,
        ..
    } = connection;

    let config =
        HueBridgeConfig::authenticated(bridge_ip, username).map_err(|error| error.to_string())?;
    let client = HueBridgeClient::new(config).map_err(|error| error.to_string())?;

    client
        .list_groups()
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn list_hue_entertainment_areas(
    connection: BridgeConnection,
) -> Result<Vec<EntertainmentArea>, String> {
    let BridgeConnection {
        bridge_ip,
        username,
        ..
    } = connection;

    let config =
        HueBridgeConfig::authenticated(bridge_ip, username).map_err(|error| error.to_string())?;
    let client = HueBridgeClient::new(config).map_err(|error| error.to_string())?;

    client
        .list_entertainment_areas()
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn list_hue_automations(connection: BridgeConnection) -> Result<Vec<Automation>, String> {
    let BridgeConnection {
        bridge_ip,
        username,
        ..
    } = connection;

    let config =
        HueBridgeConfig::authenticated(bridge_ip, username).map_err(|error| error.to_string())?;
    let client = HueBridgeClient::new(config).map_err(|error| error.to_string())?;

    client
        .list_automations()
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn get_hue_automation_detail(
    connection: BridgeConnection,
    automation_id: String,
) -> Result<AutomationDetail, String> {
    let BridgeConnection {
        bridge_ip,
        username,
        ..
    } = connection;

    let config =
        HueBridgeConfig::authenticated(bridge_ip, username).map_err(|error| error.to_string())?;
    let client = HueBridgeClient::new(config).map_err(|error| error.to_string())?;

    client
        .get_automation_detail(&automation_id)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_pipewire_output_targets() -> Result<Vec<PipeWireOutputTarget>, String> {
    capture::list_output_targets().map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn set_hue_light_state(request: SetLightStateRequest) -> Result<(), String> {
    let SetLightStateRequest {
        bridge_ip,
        username,
        light_id,
        state,
    } = request;

    let config =
        HueBridgeConfig::authenticated(bridge_ip, username).map_err(|error| error.to_string())?;
    let client = HueBridgeClient::new(config).map_err(|error| error.to_string())?;

    client
        .set_light_state(&light_id, &state)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn activate_hue_scene(request: ActivateSceneRequest) -> Result<(), String> {
    let ActivateSceneRequest {
        bridge_ip,
        username,
        scene_id,
        group_id,
    } = request;

    let config =
        HueBridgeConfig::authenticated(bridge_ip, username).map_err(|error| error.to_string())?;
    let client = HueBridgeClient::new(config).map_err(|error| error.to_string())?;

    client
        .activate_scene(&scene_id, group_id.as_deref())
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn create_hue_scene(request: CreateSceneRequest) -> Result<Scene, String> {
    let CreateSceneRequest {
        bridge_ip,
        username,
        group_id,
        scene_name,
        light_ids,
    } = request;

    let config =
        HueBridgeConfig::authenticated(bridge_ip, username).map_err(|error| error.to_string())?;
    let client = HueBridgeClient::new(config).map_err(|error| error.to_string())?;

    client
        .create_scene(&group_id, &scene_name, &light_ids)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn delete_hue_scene(request: DeleteSceneRequest) -> Result<(), String> {
    let DeleteSceneRequest {
        bridge_ip,
        username,
        scene_id,
    } = request;

    let config =
        HueBridgeConfig::authenticated(bridge_ip, username).map_err(|error| error.to_string())?;
    let client = HueBridgeClient::new(config).map_err(|error| error.to_string())?;

    client
        .delete_scene(&scene_id)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn set_hue_automation_enabled(
    request: SetAutomationEnabledRequest,
) -> Result<(), String> {
    let SetAutomationEnabledRequest {
        connection,
        automation_id,
        enabled,
    } = request;

    let config = HueBridgeConfig::authenticated(connection.bridge_ip, connection.username)
        .map_err(|error| error.to_string())?;
    let client = HueBridgeClient::new(config).map_err(|error| error.to_string())?;

    client
        .set_automation_enabled(&automation_id, enabled)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn update_hue_automation(request: UpdateAutomationRequest) -> Result<(), String> {
    let UpdateAutomationRequest {
        connection,
        automation_id,
        name,
        enabled,
        configuration,
    } = request;

    let config = HueBridgeConfig::authenticated(connection.bridge_ip, connection.username)
        .map_err(|error| error.to_string())?;
    let client = HueBridgeClient::new(config).map_err(|error| error.to_string())?;

    client
        .update_automation(&automation_id, &name, enabled, configuration.as_ref())
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn start_hue_audio_sync(
    request: AudioSyncStartRequest,
    audio_sync: State<'_, AudioSyncManager>,
) -> Result<AudioSyncStartResult, String> {
    let AudioSyncStartRequest {
        connection,
        entertainment_area_id,
        pipewire_target_object,
        speed_mode,
        color_palette,
        base_color_hex,
        brightness_ceiling,
    } = request;

    let config =
        HueBridgeConfig::authenticated(connection.bridge_ip.clone(), connection.username.clone())
            .map_err(|error| error.to_string())?;
    let client = HueBridgeClient::new(config).map_err(|error| error.to_string())?;
    let area = client
        .list_entertainment_areas()
        .await
        .map_err(|error| error.to_string())?
        .into_iter()
        .find(|area| area.id == entertainment_area_id)
        .ok_or_else(|| "The selected entertainment area is no longer available.".to_string())?;

    timeout(
        Duration::from_secs(6),
        audio_sync.start(
            connection,
            area,
            pipewire_target_object,
            speed_mode,
            color_palette,
            base_color_hex,
            brightness_ceiling,
        ),
    )
    .await
    .map_err(|_| {
        "Hue audio sync start timed out while waiting for the bridge or audio pipeline.".to_string()
    })?
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn stop_hue_audio_sync(audio_sync: State<'_, AudioSyncManager>) -> Result<(), String> {
    audio_sync.stop().await.map_err(|error| error.to_string())
}

#[tauri::command]
pub fn update_hue_audio_sync(
    request: AudioSyncUpdateRequest,
    audio_sync: State<'_, AudioSyncManager>,
) -> Result<(), String> {
    let AudioSyncUpdateRequest {
        speed_mode,
        color_palette,
        base_color_hex,
        brightness_ceiling,
    } = request;

    audio_sync
        .update(
            speed_mode,
            color_palette,
            base_color_hex,
            brightness_ceiling,
        )
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn load_persisted_bridge_connection() -> Result<Option<BridgeConnection>, String> {
    state_load_bridge_connection()
}

#[tauri::command]
pub fn save_persisted_bridge_connection(connection: BridgeConnection) -> Result<(), String> {
    state_save_bridge_connection(&connection)
}

#[tauri::command]
pub fn clear_persisted_bridge_connection() -> Result<(), String> {
    state_clear_bridge_connection()
}

#[tauri::command]
pub fn load_persisted_room_order(connection: BridgeConnection) -> Result<Vec<String>, String> {
    state_load_room_order(&connection)
}

#[tauri::command]
pub fn save_persisted_room_order(request: SaveRoomOrderRequest) -> Result<(), String> {
    state_save_room_order(&request)
}

#[tauri::command]
pub fn clear_persisted_room_order(connection: BridgeConnection) -> Result<(), String> {
    state_clear_room_order(&connection)
}

#[tauri::command]
pub fn load_audio_sync_preferences() -> Result<AudioSyncPreferences, String> {
    state_load_audio_sync_preferences()
}

#[tauri::command]
pub fn save_audio_sync_preferences(preferences: AudioSyncPreferences) -> Result<(), String> {
    state_save_audio_sync_preferences(&preferences)
}

#[tauri::command]
pub fn quit_app(app: AppHandle) {
    app.exit(0);
}

#[tauri::command]
pub fn load_theme_preference() -> Result<ThemePreference, String> {
    state_load_theme_preference()
}

#[tauri::command]
pub fn save_theme_preference(preference: ThemePreference) -> Result<(), String> {
    state_save_theme_preference(&preference)
}
