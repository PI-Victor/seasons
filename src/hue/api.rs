use crate::hue::models::{
    ActivateSceneRequest, BridgeConnection, CreateSceneRequest, CreateUserRequest,
    DiscoveredBridge, Group, Light, RegisteredApp, Scene, SetLightStateRequest,
};
use serde::Serialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

pub async fn discover_hue_bridges() -> Result<Vec<DiscoveredBridge>, String> {
    invoke_without_args("discover_hue_bridges").await
}

pub async fn create_hue_user(request: CreateUserRequest) -> Result<RegisteredApp, String> {
    invoke_with_named_args("create_hue_user", &[("request", &request)]).await
}

pub async fn list_hue_lights(connection: BridgeConnection) -> Result<Vec<Light>, String> {
    invoke_with_named_args("list_hue_lights", &[("connection", &connection)]).await
}

pub async fn list_hue_scenes(connection: BridgeConnection) -> Result<Vec<Scene>, String> {
    invoke_with_named_args("list_hue_scenes", &[("connection", &connection)]).await
}

pub async fn list_hue_groups(connection: BridgeConnection) -> Result<Vec<Group>, String> {
    invoke_with_named_args("list_hue_groups", &[("connection", &connection)]).await
}

pub async fn set_hue_light_state(request: SetLightStateRequest) -> Result<(), String> {
    invoke_with_named_args("set_hue_light_state", &[("request", &request)]).await
}

pub async fn activate_hue_scene(request: ActivateSceneRequest) -> Result<(), String> {
    invoke_with_named_args("activate_hue_scene", &[("request", &request)]).await
}

pub async fn create_hue_scene(request: CreateSceneRequest) -> Result<Scene, String> {
    invoke_with_named_args("create_hue_scene", &[("request", &request)]).await
}

async fn invoke_without_args<T>(cmd: &str) -> Result<T, String>
where
    T: serde::de::DeserializeOwned,
{
    let response = invoke(cmd, JsValue::NULL).await?;

    serde_wasm_bindgen::from_value(response)
        .map_err(|error| format!("failed to decode response for `{cmd}`: {error}"))
}

async fn invoke_with_named_args<TRequest, TResponse>(
    cmd: &str,
    args: &[(&str, &TRequest)],
) -> Result<TResponse, String>
where
    TRequest: Serialize,
    TResponse: serde::de::DeserializeOwned,
{
    let payload = js_sys::Object::new();
    for (key, value) in args {
        let encoded = serde_wasm_bindgen::to_value(value)
            .map_err(|error| format!("failed to encode `{cmd}` arguments: {error}"))?;
        js_sys::Reflect::set(&payload, &JsValue::from_str(key), &encoded)
            .map_err(js_error_to_string)?;
    }

    let response = invoke(cmd, payload.into()).await?;
    serde_wasm_bindgen::from_value(response)
        .map_err(|error| format!("failed to decode response for `{cmd}`: {error}"))
}

async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, String> {
    let global = js_sys::global();
    let window =
        js_sys::Reflect::get(&global, &JsValue::from_str("window")).map_err(js_error_to_string)?;
    let tauri = js_sys::Reflect::get(&window, &JsValue::from_str("__TAURI__"))
        .map_err(js_error_to_string)?;
    let core =
        js_sys::Reflect::get(&tauri, &JsValue::from_str("core")).map_err(js_error_to_string)?;
    let invoke =
        js_sys::Reflect::get(&core, &JsValue::from_str("invoke")).map_err(js_error_to_string)?;
    let function = invoke
        .dyn_into::<js_sys::Function>()
        .map_err(|_| "Tauri invoke bridge is not available on window.__TAURI__.core".to_string())?;

    let promise = function
        .call2(&core, &JsValue::from_str(cmd), &args)
        .map_err(js_error_to_string)?
        .dyn_into::<js_sys::Promise>()
        .map_err(|_| "Tauri invoke did not return a Promise".to_string())?;

    JsFuture::from(promise).await.map_err(js_error_to_string)
}

fn js_error_to_string(error: JsValue) -> String {
    if let Some(message) = error.as_string() {
        return message;
    }

    js_sys::JSON::stringify(&error)
        .ok()
        .and_then(|value| value.as_string())
        .unwrap_or_else(|| "unknown JavaScript bridge error".to_string())
}
