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

use crate::ollama::models::{
    ExecuteOllamaCommandRequest, ExecuteOllamaCommandResult, OllamaSettings,
};
use serde::Serialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

pub async fn load_ollama_settings() -> Result<OllamaSettings, String> {
    invoke_without_args("load_ollama_settings").await
}

pub async fn save_ollama_settings(settings: &OllamaSettings) -> Result<(), String> {
    invoke_with_named_args("save_ollama_settings", &[("settings", settings)]).await
}

pub async fn execute_ollama_command(
    request: ExecuteOllamaCommandRequest,
) -> Result<ExecuteOllamaCommandResult, String> {
    invoke_with_named_args("execute_ollama_command", &[("request", &request)]).await
}

pub async fn probe_ollama_connection(settings: &OllamaSettings) -> Result<(), String> {
    invoke_with_named_args("probe_ollama_connection", &[("settings", settings)]).await
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
