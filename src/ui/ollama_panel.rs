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

use crate::ollama::OllamaSettings;
use leptos::prelude::*;

#[component]
pub fn OllamaPanel(
    settings: ReadSignal<OllamaSettings>,
    is_saving: ReadSignal<bool>,
    on_base_url_input: Callback<String>,
    on_model_input: Callback<String>,
    on_api_key_input: Callback<String>,
    on_timeout_input: Callback<String>,
    on_save: Callback<()>,
) -> impl IntoView {
    view! {
        <section class="bridge-panel surface-panel">
            <div class="panel-header">
                <div>
                    <p class="panel-kicker">"Settings"</p>
                    <h2>"Ollama command routing"</h2>
                </div>
            </div>

            <p class="panel-copy">
                "Connect a local or remote Ollama server. The app sends a structured bridge snapshot so the model can plan exact light, scene, automation, and audio-sync actions."
            </p>

            <div class="field-grid">
                <label class="field">
                    <span class="field-label">"Base URL"</span>
                    <input
                        type="text"
                        placeholder="http://localhost:11434"
                        prop:value=move || settings.get().base_url
                        on:input=move |event| on_base_url_input.run(event_target_value(&event))
                    />
                </label>

                <label class="field">
                    <span class="field-label">"Model"</span>
                    <input
                        type="text"
                        placeholder="llama3.2:3b"
                        prop:value=move || settings.get().model
                        on:input=move |event| on_model_input.run(event_target_value(&event))
                    />
                </label>

                <label class="field">
                    <span class="field-label">"Bearer token (optional)"</span>
                    <input
                        type="password"
                        placeholder="Leave empty for local unauthenticated Ollama"
                        prop:value=move || settings.get().api_key.unwrap_or_default()
                        on:input=move |event| on_api_key_input.run(event_target_value(&event))
                    />
                </label>

                <label class="field">
                    <span class="field-label">"Timeout seconds"</span>
                    <input
                        type="number"
                        min="5"
                        max="120"
                        step="1"
                        prop:value=move || settings.get().request_timeout_seconds.to_string()
                        on:input=move |event| on_timeout_input.run(event_target_value(&event))
                    />
                </label>
            </div>

            <div class="panel-action-row">
                <button
                    class="primary-button"
                    on:click=move |_| on_save.run(())
                    disabled=move || is_saving.get()
                >
                    {move || {
                        if is_saving.get() {
                            "Saving..."
                        } else {
                            "Save Ollama settings"
                        }
                    }}
                </button>
            </div>
        </section>
    }
}
