use crate::app_state::{
    clear_bridge_connection as state_clear_bridge_connection,
    clear_room_order as state_clear_room_order,
    load_bridge_connection as state_load_bridge_connection,
    load_room_order as state_load_room_order, load_theme_preference as state_load_theme_preference,
    save_bridge_connection as state_save_bridge_connection,
    save_room_order as state_save_room_order, save_theme_preference as state_save_theme_preference,
    SaveRoomOrderRequest,
};
use crate::hue::{
    ActivateSceneRequest, BridgeConnection, CreateSceneRequest, CreateUserRequest,
    DeleteSceneRequest, DiscoveredBridge, Group, HueBridgeClient, HueBridgeConfig, Light,
    RegisteredApp, Scene, SetLightStateRequest,
};
use crate::theme::ThemePreference;
use tauri::AppHandle;

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
pub async fn list_hue_groups(connection: BridgeConnection) -> Result<Vec<Group>, String> {
    let BridgeConnection {
        bridge_ip,
        username,
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
