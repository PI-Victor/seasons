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
        <section class="audio-sync-panel surface-panel">
            <div class="settings-header">
                <div>
                    <p class="panel-kicker">"Entertainment"</p>
                    <h2>"Audio sync"</h2>
                </div>
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
            </div>

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
                    <span class="field-label">"PipeWire output"</span>
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
                                            "No PipeWire outputs found"
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
                        } else {
                            return;
                        }
                    }
                    disabled=move || {
                        is_audio_sync_starting.get()
                            ||
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
        </section>
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
    }
}

fn parse_audio_sync_color_palette(value: &str) -> AudioSyncColorPalette {
    match value {
        "sunset" => AudioSyncColorPalette::Sunset,
        "aurora" => AudioSyncColorPalette::Aurora,
        "ocean" => AudioSyncColorPalette::Ocean,
        "rose" => AudioSyncColorPalette::Rose,
        "mono" => AudioSyncColorPalette::Mono,
        _ => AudioSyncColorPalette::CurrentRoom,
    }
}
