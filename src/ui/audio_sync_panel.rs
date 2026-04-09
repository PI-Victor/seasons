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

use crate::hue::{
    AudioSyncColorPalette, AudioSyncSpeedMode, BridgeConnection, EntertainmentArea,
    PipeWireOutputTarget,
};
use leptos::prelude::*;

#[component]
pub fn AudioSyncPanel(
    active_connection: ReadSignal<Option<BridgeConnection>>,
    entertainment_areas: ReadSignal<Vec<EntertainmentArea>>,
    selected_entertainment_area_id: ReadSignal<String>,
    pipewire_targets: ReadSignal<Vec<PipeWireOutputTarget>>,
    selected_pipewire_target_object: ReadSignal<String>,
    selected_sync_speed_mode: ReadSignal<AudioSyncSpeedMode>,
    selected_sync_color_palette: ReadSignal<AudioSyncColorPalette>,
    is_loading_areas: ReadSignal<bool>,
    is_loading_pipewire_targets: ReadSignal<bool>,
    is_audio_syncing: ReadSignal<bool>,
    is_audio_sync_starting: ReadSignal<bool>,
    on_select_area: Callback<String>,
    on_select_pipewire_target: Callback<String>,
    on_select_sync_speed_mode: Callback<AudioSyncSpeedMode>,
    on_select_sync_color_palette: Callback<AudioSyncColorPalette>,
    on_start: Callback<()>,
    on_stop: Callback<()>,
) -> impl IntoView {
    view! {
        <details class="audio-sync-panel surface-panel">
            <summary class="audio-sync-summary">
                <div class="settings-header">
                    <div>
                        <p class="panel-kicker">"Entertainment"</p>
                        <h2>"Audio sync"</h2>
                    </div>
                    <div class="audio-sync-summary-meta">
                        <div class="connection-pulse">
                            <span class="connection-pulse-dot"></span>
                            <span>
                                {move || {
                                    if is_audio_syncing.get() {
                                        "Live streaming to Hue".to_string()
                                    } else if is_audio_sync_starting.get() {
                                        "Starting stream...".to_string()
                                    } else {
                                        "Idle".to_string()
                                    }
                                }}
                            </span>
                        </div>
                        <span class="audio-sync-summary-toggle">"Open"</span>
                    </div>
                </div>
            </summary>

            <div class="audio-sync-body">
                <p class="panel-copy">
                    "This uses the Hue Entertainment stream, not the normal bridge REST light path. Pick an entertainment area from the Hue app, then start sync."
                </p>

                <div class="field-grid">
                    <label class="theme-field">
                        <span class="field-label">"Entertainment area"</span>
                        <select
                            class="theme-select"
                            prop:value=selected_entertainment_area_id
                            on:change=move |event| on_select_area.run(event_target_value(&event))
                            disabled=move || is_loading_areas.get() || entertainment_areas.get().is_empty()
                        >
                            {move || {
                                let areas = entertainment_areas.get();
                                if areas.is_empty() {
                                    view! {
                                        <option value="">
                                            {if is_loading_areas.get() {
                                                "Loading areas..."
                                            } else {
                                                "No entertainment areas on this bridge"
                                            }}
                                        </option>
                                    }.into_any()
                                } else {
                                    areas.into_iter().map(|area| {
                                        let detail = area.configuration_type.clone().unwrap_or_else(|| "screen".to_string());
                                        view! {
                                            <option value=area.id.clone()>
                                                {format!("{} ({detail}, {} channels)", area.name, area.channels.len())}
                                            </option>
                                        }
                                    }).collect_view().into_any()
                                }
                            }}
                        </select>
                    </label>

                    <label class="theme-field">
                        <span class="field-label">"Audio source"</span>
                        <select
                            class="theme-select"
                            prop:value=selected_pipewire_target_object
                            on:change=move |event| on_select_pipewire_target.run(event_target_value(&event))
                            disabled=move || is_loading_pipewire_targets.get() || pipewire_targets.get().is_empty()
                        >
                            {move || {
                                let targets = pipewire_targets.get();
                                if targets.is_empty() {
                                    view! {
                                        <option value="">
                                            {if is_loading_pipewire_targets.get() {
                                                "Loading outputs..."
                                            } else {
                                                "No audio sources found"
                                            }}
                                        </option>
                                    }.into_any()
                                } else {
                                    targets.into_iter().map(|target| {
                                        view! {
                                            <option value=target.target_object.clone()>
                                                {target.description}
                                            </option>
                                        }
                                    }).collect_view().into_any()
                                }
                            }}
                        </select>
                    </label>

                    <label class="theme-field">
                        <span class="field-label">"Sync speed"</span>
                        <select
                            class="theme-select"
                            prop:value=move || audio_sync_speed_mode_value(selected_sync_speed_mode.get())
                            on:change=move |event| {
                                on_select_sync_speed_mode.run(parse_audio_sync_speed_mode(&event_target_value(&event)))
                            }
                        >
                            <option value="slow">"Slow"</option>
                            <option value="medium">"Medium"</option>
                            <option value="high">"High"</option>
                        </select>
                    </label>

                    <label class="theme-field">
                        <span class="field-label">"Color palette"</span>
                        <select
                            class="theme-select"
                            prop:value=move || audio_sync_color_palette_value(selected_sync_color_palette.get())
                            on:change=move |event| {
                                on_select_sync_color_palette.run(parse_audio_sync_color_palette(&event_target_value(&event)))
                            }
                        >
                            <option value="currentRoom">"Current room"</option>
                            <option value="sunset">"Sunset"</option>
                            <option value="aurora">"Aurora"</option>
                            <option value="ocean">"Ocean"</option>
                            <option value="rose">"Rose"</option>
                            <option value="mono">"Mono"</option>
                            <option value="neonPulse">"Neon pulse"</option>
                            <option value="prism">"Prism"</option>
                            <option value="vocalGlow">"Vocal glow"</option>
                            <option value="fireIce">"Fire and ice"</option>
                        </select>
                    </label>
                </div>

                <div class="panel-action-row">
                    <button
                        class="primary-button"
                        on:click=move |_| {
                            if is_audio_syncing.get() {
                                on_stop.run(());
                            } else if !is_audio_sync_starting.get() {
                                on_start.run(());
                            }
                        }
                        disabled=move || {
                            active_connection.get().is_none()
                                || (!is_audio_syncing.get() && selected_entertainment_area_id.get().trim().is_empty())
                        }
                    >
                        {move || {
                            if is_audio_syncing.get() {
                                "Stop audio sync"
                            } else if is_audio_sync_starting.get() {
                                "Starting audio sync..."
                            } else {
                                "Start audio sync"
                            }
                        }}
                    </button>
                </div>
            </div>
        </details>
    }
}

fn audio_sync_speed_mode_value(mode: AudioSyncSpeedMode) -> &'static str {
    match mode {
        AudioSyncSpeedMode::Slow => "slow",
        AudioSyncSpeedMode::Medium => "medium",
        AudioSyncSpeedMode::High => "high",
    }
}

fn parse_audio_sync_speed_mode(value: &str) -> AudioSyncSpeedMode {
    match value {
        "slow" => AudioSyncSpeedMode::Slow,
        "high" => AudioSyncSpeedMode::High,
        _ => AudioSyncSpeedMode::Medium,
    }
}

fn audio_sync_color_palette_value(palette: AudioSyncColorPalette) -> &'static str {
    match palette {
        AudioSyncColorPalette::CurrentRoom => "currentRoom",
        AudioSyncColorPalette::Sunset => "sunset",
        AudioSyncColorPalette::Aurora => "aurora",
        AudioSyncColorPalette::Ocean => "ocean",
        AudioSyncColorPalette::Rose => "rose",
        AudioSyncColorPalette::Mono => "mono",
        AudioSyncColorPalette::NeonPulse => "neonPulse",
        AudioSyncColorPalette::Prism => "prism",
        AudioSyncColorPalette::VocalGlow => "vocalGlow",
        AudioSyncColorPalette::FireIce => "fireIce",
    }
}

fn parse_audio_sync_color_palette(value: &str) -> AudioSyncColorPalette {
    match value {
        "sunset" => AudioSyncColorPalette::Sunset,
        "aurora" => AudioSyncColorPalette::Aurora,
        "ocean" => AudioSyncColorPalette::Ocean,
        "rose" => AudioSyncColorPalette::Rose,
        "mono" => AudioSyncColorPalette::Mono,
        "neonPulse" => AudioSyncColorPalette::NeonPulse,
        "prism" => AudioSyncColorPalette::Prism,
        "vocalGlow" => AudioSyncColorPalette::VocalGlow,
        "fireIce" => AudioSyncColorPalette::FireIce,
        _ => AudioSyncColorPalette::CurrentRoom,
    }
}
