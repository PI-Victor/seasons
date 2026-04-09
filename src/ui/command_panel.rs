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
use crate::ollama::ExecuteOllamaCommandResult;
use leptos::prelude::*;

#[component]
pub fn CommandPanel(
    active_connection: ReadSignal<Option<BridgeConnection>>,
    ollama_connection_ok: ReadSignal<Option<bool>>,
    is_checking_connection: ReadSignal<bool>,
    command_input: ReadSignal<String>,
    is_executing: ReadSignal<bool>,
    last_result: ReadSignal<Option<ExecuteOllamaCommandResult>>,
    on_input: Callback<String>,
    on_execute: Callback<()>,
) -> impl IntoView {
    view! {
        <details class="command-panel surface-panel">
            <summary class="command-summary">
                <div class="settings-header">
                    <div>
                        <p class="panel-kicker">"Command"</p>
                        <h2>"AI control"</h2>
                    </div>
                    <div class="audio-sync-summary-meta">
                        <div class="connection-pulse">
                            <span
                                class=move || {
                                    if is_checking_connection.get() {
                                        "connection-pulse-dot is-checking"
                                    } else if ollama_connection_ok.get().unwrap_or(false) {
                                        "connection-pulse-dot"
                                    } else {
                                        "connection-pulse-dot is-offline"
                                    }
                                }
                            ></span>
                            <span>
                                {move || {
                                    if is_checking_connection.get() {
                                        "Checking AI".to_string()
                                    } else if ollama_connection_ok.get().unwrap_or(false) {
                                        "AI online".to_string()
                                    } else {
                                        "AI offline".to_string()
                                    }
                                }}
                            </span>
                        </div>
                        <span class="audio-sync-summary-toggle">"Open"</span>
                    </div>
                </div>
            </summary>

            <div class="command-body">
                <p class="panel-copy">
                    "Write a natural-language command and execute it against the active bridge using your configured Ollama model."
                </p>

                <label class="field">
                    <span class="field-label">"Instruction"</span>
                    <textarea
                        placeholder="kill the lights, set living room to amber red, and start audio sync in lounge"
                        prop:value=command_input
                        on:input=move |event| on_input.run(event_target_value(&event))
                        rows="4"
                    ></textarea>
                </label>

                <div class="panel-action-row">
                    <button
                        class="primary-button"
                        on:click=move |_| on_execute.run(())
                        disabled=move || is_executing.get() || active_connection.get().is_none()
                    >
                        {move || {
                            if is_executing.get() {
                                "Executing..."
                            } else {
                                "Run command"
                            }
                        }}
                    </button>
                </div>

                <Show when=move || last_result.get().is_some()>
                    {move || {
                        last_result.get().map(|result| {
                            view! {
                                <div class="bridge-subsection">
                                    <div class="pairing-copy">
                                        <p class="panel-kicker">"Assistant response"</p>
                                        <p>{result.assistant_message.clone()}</p>
                                    </div>

                                    <div class="bridge-chip-list">
                                        {if result.actions.is_empty() {
                                            view! {
                                                <div class="empty-chip-state">
                                                    "No actions were executed."
                                                </div>
                                            }.into_any()
                                        } else {
                                            result.actions
                                                .into_iter()
                                                .map(|action| {
                                                    view! {
                                                        <article class="surface-panel">
                                                            <p class="panel-kicker">{format!("{} • {}", action.action, action.status)}</p>
                                                            <p><strong>{action.target}</strong></p>
                                                            <p>{action.detail}</p>
                                                        </article>
                                                    }
                                                })
                                                .collect_view()
                                                .into_any()
                                        }}
                                    </div>
                                </div>
                            }
                        })
                    }}
                </Show>
            </div>
        </details>
    }
}
