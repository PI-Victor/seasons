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

use crate::hue::{self, Automation, AutomationConfigValue, AutomationDetail, BridgeConnection};
use leptos::{prelude::*, task::spawn_local};
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq, Eq)]
enum ConfigPathSegment {
    Key(String),
    Index(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TimePickerMode {
    Hour,
    Minute,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TimePickerTarget {
    StringValue,
    TimePointObject,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TimePickerState {
    label: String,
    path: Vec<ConfigPathSegment>,
    hour: u8,
    minute: u8,
    include_seconds: bool,
    mode: TimePickerMode,
    target: TimePickerTarget,
}

#[component]
pub fn AutomationPanel(
    active_connection: ReadSignal<Option<BridgeConnection>>,
    automations: ReadSignal<Vec<Automation>>,
    pending_automation_ids: ReadSignal<HashSet<String>>,
    on_toggle_automation: Callback<(String, bool)>,
    on_update_automation: Callback<(String, String, Option<bool>, Option<AutomationConfigValue>)>,
) -> impl IntoView {
    let (editor_automation_id, set_editor_automation_id) = signal(None::<String>);
    let (selected_automation_detail, set_selected_automation_detail) =
        signal(None::<AutomationDetail>);
    let (is_loading_detail, set_is_loading_detail) = signal(false);
    let (detail_error, set_detail_error) = signal(None::<String>);
    let (draft_name, set_draft_name) = signal(String::new());
    let (draft_enabled, set_draft_enabled) = signal(false);
    let (draft_configuration, set_draft_configuration) = signal(None::<AutomationConfigValue>);
    let (time_picker, set_time_picker) = signal(None::<TimePickerState>);
    let ordered_automations = Signal::derive(move || {
        let mut automations = automations.get();
        automations.sort_by(|left, right| left.name.cmp(&right.name));
        automations
    });
    let selected_automation = Signal::derive(move || {
        let selected_id = editor_automation_id.get();
        let automations = ordered_automations.get();

        selected_id
            .as_deref()
            .and_then(|id| automations.iter().find(|automation| automation.id == id))
            .cloned()
    });
    Effect::new(move |_| {
        let Some(connection) = active_connection.get() else {
            set_selected_automation_detail.set(None);
            set_is_loading_detail.set(false);
            set_detail_error.set(None);
            return;
        };
        let Some(selected) = selected_automation.get() else {
            set_selected_automation_detail.set(None);
            set_is_loading_detail.set(false);
            set_detail_error.set(None);
            return;
        };

        if pending_automation_ids.get().contains(&selected.id) {
            return;
        }

        let automation_id = selected.id.clone();
        set_is_loading_detail.set(true);
        set_detail_error.set(None);

        spawn_local(async move {
            let detail_result =
                hue::get_hue_automation_detail(connection, automation_id.clone()).await;
            let still_selected = selected_automation
                .get_untracked()
                .as_ref()
                .is_some_and(|current| current.id == automation_id);

            if !still_selected {
                return;
            }

            set_is_loading_detail.set(false);
            match detail_result {
                Ok(detail) => {
                    set_draft_name.set(detail.name.clone());
                    set_draft_enabled.set(detail.enabled.unwrap_or(false));
                    set_draft_configuration.set(detail.configuration.clone());
                    set_selected_automation_detail.set(Some(detail));
                    set_detail_error.set(None);
                }
                Err(error) => {
                    set_selected_automation_detail.set(None);
                    set_detail_error.set(Some(error));
                }
            }
        });
    });
    Effect::new(move |_| {
        let selected = selected_automation.get();
        let detail = selected_automation_detail.get();
        if let Some(detail) = detail {
            set_draft_name.set(detail.name.clone());
            set_draft_enabled.set(detail.enabled.unwrap_or(false));
            set_draft_configuration.set(detail.configuration.clone());
        } else if let Some(selected) = selected {
            set_draft_name.set(selected.name.clone());
            set_draft_enabled.set(selected.enabled.unwrap_or(false));
            set_draft_configuration.set(None);
        } else {
            set_draft_name.set(String::new());
            set_draft_enabled.set(false);
            set_draft_configuration.set(None);
        }
    });

    let open_time_picker = Callback::new(move |state: TimePickerState| {
        set_time_picker.set(Some(state));
    });

    view! {
        <section class="surface-panel automation-panel">
            {move || {
                let automations = ordered_automations.get();

                if automations.is_empty() {
                    view! {
                        <div class="empty-state compact-empty-state">
                            <h3>"No automations found"</h3>
                            <p>"This bridge did not expose any toggleable Hue automations."</p>
                        </div>
                    }
                        .into_any()
                } else if let Some(picker) = time_picker.get() {
                    let formatted = picker.formatted_display();
                    let is_hour_mode = picker.mode == TimePickerMode::Hour;
                    let hour_cards = (0_u8..24)
                        .map(|hour| {
                            let label = format!("{hour:02}");
                            let is_selected = picker.hour == hour;
                            view! {
                                <button
                                    class="automation-time-card"
                                    class:is-selected=is_selected
                                    on:click=move |_| {
                                        set_time_picker.update(|state| {
                                            if let Some(state) = state.as_mut() {
                                                state.hour = hour;
                                                state.mode = TimePickerMode::Minute;
                                            }
                                        });
                                    }
                                >
                                    {label}
                                </button>
                            }
                        })
                        .collect_view();
                    let minute_cards = (0_u8..12)
                        .map(|step| step * 5)
                        .map(|minute| {
                            let label = format!("{minute:02}");
                            let is_selected = picker.minute == minute;
                            view! {
                                <button
                                    class="automation-time-card"
                                    class:is-selected=is_selected
                                    on:click=move |_| {
                                        set_time_picker.update(|state| {
                                            if let Some(state) = state.as_mut() {
                                                state.minute = minute;
                                            }
                                        });
                                    }
                                >
                                    {label}
                                </button>
                            }
                        })
                        .collect_view();

                    view! {
                        <div class="automation-editor-page">
                            <div class="automation-page-header">
                                <button
                                    class="ghost-button automation-page-back"
                                    on:click=move |_| set_time_picker.set(None)
                                >
                                    <span class="fa-solid fa-chevron-left" aria-hidden="true"></span>
                                    <span>"Back"</span>
                                </button>
                                <div class="automation-page-copy">
                                    <p class="panel-kicker">"Time"</p>
                                    <h2>{format!("Set {}", picker.label)}</h2>
                                </div>
                                <button
                                    class="primary-button automation-page-save"
                                    on:click=move |_| {
                                        if let Some(picker) = time_picker.get_untracked() {
                                            match picker.target {
                                                TimePickerTarget::StringValue => set_config_string(
                                                    set_draft_configuration,
                                                    picker.path.clone(),
                                                    picker.formatted_value(),
                                                ),
                                                TimePickerTarget::TimePointObject => {
                                                    set_time_point_value(
                                                        set_draft_configuration,
                                                        picker.path.clone(),
                                                        picker.hour,
                                                        picker.minute,
                                                    );
                                                }
                                            }
                                        }
                                        set_time_picker.set(None);
                                    }
                                >
                                    "Save"
                                </button>
                            </div>

                            <div class="automation-time-picker">
                                <div class="automation-flip-clock">
                                    <button
                                        class="automation-flip-card"
                                        class:is-active=is_hour_mode
                                        on:click=move |_| {
                                            set_time_picker.update(|state| {
                                                if let Some(state) = state.as_mut() {
                                                    state.mode = TimePickerMode::Hour;
                                                }
                                            });
                                        }
                                    >
                                        <span class="automation-flip-card-top">{format!("{:02}", picker.hour)}</span>
                                        <span class="automation-flip-card-bottom">{format!("{:02}", picker.hour)}</span>
                                    </button>
                                    <span class="automation-flip-separator">":"</span>
                                    <button
                                        class="automation-flip-card"
                                        class:is-active=move || !is_hour_mode
                                        on:click=move |_| {
                                            set_time_picker.update(|state| {
                                                if let Some(state) = state.as_mut() {
                                                    state.mode = TimePickerMode::Minute;
                                                }
                                            });
                                        }
                                    >
                                        <span class="automation-flip-card-top">{format!("{:02}", picker.minute)}</span>
                                        <span class="automation-flip-card-bottom">{format!("{:02}", picker.minute)}</span>
                                    </button>
                                </div>
                                <p class="automation-time-display-copy">{formatted}</p>

                                <div class="automation-time-card-section">
                                    <p class="automation-config-section-title">
                                        "Hours"
                                    </p>
                                    <div class="automation-time-card-grid">
                                        {hour_cards}
                                    </div>
                                </div>

                                <div class="automation-time-card-section">
                                    <p class="automation-config-section-title">
                                        "Minutes"
                                    </p>
                                    <div class="automation-time-card-grid">
                                        {minute_cards}
                                    </div>
                                </div>
                            </div>
                        </div>
                    }.into_any()
                } else if let Some(automation) = selected_automation.get() {
                    let automation_id = automation.id.clone();
                    let icon_class = automation_icon_class(&automation);
                    let detail = selected_automation_detail.get();
                    let subtitle = detail
                        .as_ref()
                        .and_then(|detail| detail.script_name.clone())
                        .or_else(|| automation.script_id.clone())
                        .unwrap_or_else(|| "Hue automation".to_string());
                    let automation_type = detail
                        .as_ref()
                        .and_then(|detail| detail.automation_type.clone())
                        .or_else(|| automation.automation_type.clone());
                    let script_type = detail.as_ref().and_then(|detail| detail.script_type.clone());
                    let original_name = detail
                        .as_ref()
                        .map(|detail| detail.name.clone())
                        .unwrap_or_else(|| automation.name.clone());
                    let original_enabled = detail
                        .as_ref()
                        .and_then(|detail| detail.enabled)
                        .or(automation.enabled)
                        .unwrap_or(false);
                    let original_configuration =
                        detail.as_ref().and_then(|detail| detail.configuration.clone());
                    let is_pending = pending_automation_ids.get().contains(&automation.id);
                    let can_toggle = detail
                        .as_ref()
                        .map(|detail| detail.enabled.is_some())
                        .unwrap_or_else(|| automation.enabled.is_some());
                    let can_save = active_connection.get().is_some()
                        && !is_pending
                        && (!draft_name.get().trim().eq(original_name.trim())
                            || (can_toggle && draft_enabled.get() != original_enabled)
                            || draft_configuration.get() != original_configuration);

                    view! {
                        <div class="automation-editor-page">
                            <div class="automation-page-header">
                                <button
                                    class="ghost-button automation-page-back"
                                    on:click=move |_| {
                                        set_time_picker.set(None);
                                        set_editor_automation_id.set(None);
                                    }
                                >
                                    <span class="fa-solid fa-chevron-left" aria-hidden="true"></span>
                                    <span>"Automations"</span>
                                </button>
                                <div class="automation-page-copy">
                                    <p class="panel-kicker">"Edit automation"</p>
                                    <h2>{move || draft_name.get()}</h2>
                                    <p class="panel-copy">{subtitle.clone()}</p>
                                </div>
                                <button
                                    class="primary-button automation-page-save"
                                    disabled=move || !can_save
                                    on:click=move |_| {
                                        on_update_automation.run((
                                            automation_id.clone(),
                                            draft_name.get(),
                                            if can_toggle { Some(draft_enabled.get()) } else { None },
                                            draft_configuration.get(),
                                        ));
                                    }
                                >
                                    {move || if is_pending { "Saving..." } else { "Save" }}
                                </button>
                            </div>

                            <article class="automation-detail-card">
                                <div class="automation-card-top automation-detail-top">
                                    <div class="automation-card-identity">
                                        <span class="light-icon-shell automation-icon-shell">
                                            <span class=format!("{icon_class} fa-fw light-icon-glyph") aria-hidden="true"></span>
                                        </span>
                                        <div class="light-card-copy">
                                            <p class="light-eyebrow">"Automation"</p>
                                            <h3>{move || draft_name.get()}</h3>
                                        </div>
                                    </div>
                                </div>

                                <div class="automation-detail-meta">
                                    {automation_type
                                        .clone()
                                        .map(|automation_type| view! {
                                            <span class="light-meta-chip">{automation_type}</span>
                                        })}
                                    {script_type
                                        .clone()
                                        .map(|script_type| view! {
                                            <span class="light-meta-chip">{script_type}</span>
                                        })}
                                    {automation
                                        .script_id
                                        .clone()
                                        .map(|script_id| view! {
                                            <span class="light-meta-chip" title=script_id.clone()>{script_id.clone()}</span>
                                        })}
                                </div>

                                {move || {
                                    if is_loading_detail.get() {
                                        view! {
                                            <div class="automation-detail-block">
                                                <p class="light-subcopy">"Loading bridge automation payload…"</p>
                                            </div>
                                        }.into_any()
                                    } else if let Some(error) = detail_error.get() {
                                        view! {
                                            <div class="automation-detail-block">
                                                <p class="light-subcopy">{error}</p>
                                            </div>
                                        }.into_any()
                                    } else if let Some(detail) = selected_automation_detail.get() {
                                        let instance_json = detail.instance_json.clone();
                                        let script_json = detail.script_json.clone();
                                        let original_name_for_reset = original_name.clone();
                                        let original_enabled_for_reset = original_enabled;
                                        let original_configuration_for_reset = original_configuration.clone();
                                        view! {
                                            <>
                                                <div class="automation-detail-block">
                                                    <div class="field-grid automation-editor-grid">
                                                        <label class="field">
                                                            <span class="field-label">"Name"</span>
                                                            <input
                                                                type="text"
                                                                prop:value=draft_name
                                                                on:input=move |event| {
                                                                    set_draft_name.set(event_target_value(&event));
                                                                }
                                                                disabled=is_pending
                                                            />
                                                        </label>

                                                        <div class="field automation-toggle-field">
                                                            <span class="field-label">"Enabled"</span>
                                                            <button
                                                                class="automation-config-switch"
                                                                class:is-on=move || draft_enabled.get()
                                                                disabled=move || is_pending || !can_toggle
                                                                on:click=move |_| {
                                                                    if can_toggle {
                                                                        set_draft_enabled.update(|value| *value = !*value);
                                                                    }
                                                                }
                                                            >
                                                                <span class="automation-config-switch-track">
                                                                    <span class="automation-config-switch-thumb"></span>
                                                                </span>
                                                            </button>
                                                        </div>
                                                    </div>
                                                    <div class="panel-action-row automation-editor-actions">
                                                        <button
                                                            class="secondary-button"
                                                            disabled=is_pending
                                                            on:click=move |_| {
                                                                set_draft_name.set(original_name_for_reset.clone());
                                                                set_draft_enabled.set(original_enabled_for_reset);
                                                                set_draft_configuration.set(original_configuration_for_reset.clone());
                                                            }
                                                        >
                                                            "Reset"
                                                        </button>
                                                    </div>
                                                </div>

                                                <div class="automation-detail-block">
                                                    {move || {
                                                        if let Some(configuration) = draft_configuration.get() {
                                                            render_automation_config_value(
                                                                None,
                                                                configuration,
                                                                Vec::new(),
                                                                set_draft_configuration,
                                                                is_pending,
                                                                0,
                                                                open_time_picker,
                                                            )
                                                        } else {
                                                            view! {
                                                                <p class="light-subcopy">
                                                                    "This automation does not expose a configurable payload."
                                                                </p>
                                                            }.into_any()
                                                        }
                                                    }}
                                                </div>

                                                <details class="automation-detail-block automation-advanced">
                                                    <summary class="automation-advanced-summary">
                                                        <span class="light-eyebrow">"Advanced payload"</span>
                                                    </summary>
                                                    <pre class="automation-json">{instance_json}</pre>
                                                    {script_json.map(|script_json| view! {
                                                        <pre class="automation-json">{script_json}</pre>
                                                    })}
                                                </details>
                                            </>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <div class="automation-detail-block">
                                                <p class="light-subcopy">"No detailed bridge payload is available for this automation."</p>
                                            </div>
                                        }.into_any()
                                    }
                                }}
                            </article>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="automation-browser">
                            <div class="panel-header compact-panel-header devices-panel-header">
                                <div>
                                    <p class="panel-kicker">"Automations"</p>
                                    <h2>"Bridge automations"</h2>
                                </div>
                                <p class="panel-copy">
                                    "Open an automation to edit it on its own page."
                                </p>
                            </div>
                            <div class="automation-list">
                                {automations
                                    .into_iter()
                                    .map(|automation| {
                                        let automation_id = automation.id.clone();
                                        let automation_id_for_select = automation_id.clone();
                                        let automation_id_for_toggle = automation_id.clone();
                                        let enabled = automation.enabled.unwrap_or(false);
                                        let icon_class = automation_icon_class(&automation);
                                        let automation_type = automation.automation_type.clone();
                                        let subtitle = automation
                                            .script_id
                                            .clone()
                                            .unwrap_or_else(|| "Hue automation".to_string());
                                        let is_pending = pending_automation_ids
                                            .get()
                                            .contains(&automation.id);
                                        let can_toggle = automation.enabled.is_some();

                                        view! {
                                            <article
                                                class="automation-card automation-card-button"
                                                on:click=move |_| {
                                                    set_editor_automation_id
                                                        .set(Some(automation_id_for_select.clone()));
                                                    set_time_picker.set(None);
                                                }
                                            >
                                                <div class="automation-card-top">
                                                    <div class="automation-card-identity">
                                                        <span class="light-icon-shell automation-icon-shell">
                                                            <span class=format!("{icon_class} fa-fw light-icon-glyph") aria-hidden="true"></span>
                                                        </span>
                                                        <div class="light-card-copy">
                                                            <p class="light-eyebrow">"Automation"</p>
                                                            <h3>{automation.name}</h3>
                                                            <p class="light-subcopy" title=subtitle.clone()>{subtitle.clone()}</p>
                                                        </div>
                                                    </div>
                                                    <button
                                                        class="light-status-button"
                                                        class:is-off=move || !enabled
                                                        disabled=move || is_pending || !can_toggle
                                                        on:click=move |event| {
                                                            event.stop_propagation();
                                                            on_toggle_automation.run((automation_id_for_toggle.clone(), !enabled));
                                                        }
                                                    >
                                                        <span class="status-dot"></span>
                                                        <span>
                                                            {if can_toggle {
                                                                if enabled { "On" } else { "Off" }
                                                            } else {
                                                                "Read only"
                                                            }}
                                                        </span>
                                                    </button>
                                                </div>
                                                {automation_type
                                                    .clone()
                                                    .map(|automation_type| view! {
                                                        <div class="automation-detail-meta">
                                                            <span class="light-meta-chip">{automation_type}</span>
                                                        </div>
                                                    })}
                                                <div class="automation-card-footer">
                                                    <span class="automation-open-indicator">
                                                        <span class="fa-solid fa-sliders" aria-hidden="true"></span>
                                                        "Open editor"
                                                    </span>
                                                </div>
                                            </article>
                                        }
                                    })
                                    .collect_view()}
                            </div>
                        </div>
                    }
                        .into_any()
                }
            }}
        </section>
    }
}

fn render_automation_config_value(
    label: Option<String>,
    value: AutomationConfigValue,
    path: Vec<ConfigPathSegment>,
    set_root: WriteSignal<Option<AutomationConfigValue>>,
    disabled: bool,
    depth: usize,
    on_open_time_picker: Callback<TimePickerState>,
) -> AnyView {
    match value {
        AutomationConfigValue::Object(entries) => {
            let entries = visible_config_entries(entries);
            if entries.is_empty() {
                return ().into_any();
            }
            let title = label.unwrap_or_else(|| "Configuration".to_string());
            if let Some(time_point) = time_point_value(&entries) {
                return render_time_point_object(
                    title,
                    time_point,
                    path,
                    disabled,
                    depth,
                    on_open_time_picker,
                );
            }
            if let Some(nested_time_point) = nested_time_point_value(&entries) {
                let mut nested_path = path.clone();
                nested_path.extend(nested_time_point.path);
                return render_time_point_object(
                    nested_time_point.label,
                    nested_time_point.time_point,
                    nested_path,
                    disabled,
                    depth,
                    on_open_time_picker,
                );
            }
            if weekday_entries(&entries).is_some() {
                return render_weekday_object(title, entries, path, set_root, disabled, depth);
            }
            if entries.iter().all(|entry| is_scalar_value(&entry.value)) {
                return render_scalar_object_group(
                    title,
                    entries,
                    path,
                    set_root,
                    disabled,
                    depth,
                    on_open_time_picker,
                );
            }
            view! {
                <div class="automation-config-section" style=format!("--automation-config-depth:{depth};")>
                    <div class="automation-config-section-head">
                        <p class="automation-config-section-title">{humanize_config_label(&title)}</p>
                    </div>
                    <div class="automation-config-section-body">
                        {entries
                            .into_iter()
                            .map(|entry| {
                                let mut child_path = path.clone();
                                child_path.push(ConfigPathSegment::Key(entry.key.clone()));
                                render_config_entry(
                                    entry.key,
                                    entry.value,
                                    child_path,
                                    set_root,
                                    disabled,
                                    depth + 1,
                                    on_open_time_picker,
                                )
                            })
                            .collect_view()}
                    </div>
                </div>
            }
            .into_any()
        }
        AutomationConfigValue::Array(values) => {
            let title = label.unwrap_or_else(|| "Items".to_string());
            if weekday_values(&values).is_some() {
                return render_weekday_array(title, values, path, set_root, disabled, depth);
            }
            view! {
                <div class="automation-config-section automation-config-array" style=format!("--automation-config-depth:{depth};")>
                    <div class="automation-config-section-head">
                        <p class="automation-config-section-title">{humanize_config_label(&title)}</p>
                    </div>
                    <div class="automation-config-section-body">
                        {values
                            .into_iter()
                            .enumerate()
                            .map(|(index, item)| {
                                let mut child_path = path.clone();
                                child_path.push(ConfigPathSegment::Index(index));
                                render_automation_config_value(
                                    Some(format!("Item {}", index + 1)),
                                    item,
                                    child_path,
                                    set_root,
                                    disabled,
                                    depth + 1,
                                    on_open_time_picker,
                                )
                            })
                            .collect_view()}
                    </div>
                </div>
            }
            .into_any()
        }
        value => render_scalar_config_value(
            label,
            value,
            path,
            set_root,
            disabled,
            depth,
            on_open_time_picker,
        ),
    }
}

fn render_scalar_object_group(
    title: String,
    entries: Vec<crate::hue::AutomationConfigEntry>,
    path: Vec<ConfigPathSegment>,
    set_root: WriteSignal<Option<AutomationConfigValue>>,
    disabled: bool,
    depth: usize,
    on_open_time_picker: Callback<TimePickerState>,
) -> AnyView {
    if entries.is_empty() {
        return ().into_any();
    }

    view! {
        <div class="automation-config-section" style=format!("--automation-config-depth:{depth};")>
            <div class="automation-config-section-head">
                <p class="automation-config-section-title">{humanize_config_label(&title)}</p>
            </div>
            <div class="automation-config-section-body">
                {entries
                    .into_iter()
                    .map(|entry| {
                        let mut child_path = path.clone();
                        child_path.push(ConfigPathSegment::Key(entry.key.clone()));
                        render_config_entry(
                            entry.key,
                            entry.value,
                            child_path,
                            set_root,
                            disabled,
                            depth + 1,
                            on_open_time_picker,
                        )
                    })
                    .collect_view()}
            </div>
        </div>
    }
    .into_any()
}

fn render_config_entry(
    key: String,
    value: AutomationConfigValue,
    path: Vec<ConfigPathSegment>,
    set_root: WriteSignal<Option<AutomationConfigValue>>,
    disabled: bool,
    depth: usize,
    on_open_time_picker: Callback<TimePickerState>,
) -> AnyView {
    if let AutomationConfigValue::Object(entries) = &value {
        if let Some(time_point) = time_point_value(entries) {
            return render_time_point_object(
                key,
                time_point,
                path,
                disabled,
                depth,
                on_open_time_picker,
            );
        }

        if let Some(nested_time_point) = nested_time_point_value(entries) {
            let mut nested_path = path.clone();
            nested_path.extend(nested_time_point.path);
            return render_time_point_object(
                nested_time_point.label,
                nested_time_point.time_point,
                nested_path,
                disabled,
                depth,
                on_open_time_picker,
            );
        }
    }

    render_automation_config_value(
        Some(key),
        value,
        path,
        set_root,
        disabled,
        depth,
        on_open_time_picker,
    )
}

fn render_time_point_object(
    title: String,
    time_point: TimePointValue,
    path: Vec<ConfigPathSegment>,
    disabled: bool,
    depth: usize,
    on_open_time_picker: Callback<TimePickerState>,
) -> AnyView {
    let label = humanize_config_label(&title);
    let display = format!("{:02}:{:02}", time_point.hour, time_point.minute);

    view! {
        <div class="automation-config-row" style=format!("--automation-config-depth:{depth};")>
            <div class="automation-config-row-copy">
                <p class="automation-config-row-label">{label.clone()}</p>
                <p class="automation-config-row-subcopy">"Time"</p>
            </div>
            <div class="automation-config-row-control">
                <button
                    class="secondary-button automation-time-trigger"
                    disabled=disabled
                    on:click=move |_| {
                        on_open_time_picker.run(TimePickerState::from_time_point(
                            label.clone(),
                            path.clone(),
                            time_point.hour,
                            time_point.minute,
                        ));
                    }
                >
                    {display}
                </button>
            </div>
        </div>
    }
    .into_any()
}

fn render_weekday_object(
    title: String,
    entries: Vec<crate::hue::AutomationConfigEntry>,
    path: Vec<ConfigPathSegment>,
    set_root: WriteSignal<Option<AutomationConfigValue>>,
    disabled: bool,
    depth: usize,
) -> AnyView {
    let weekdays = weekday_entries(&entries).unwrap_or_default();
    if weekdays.is_empty() {
        return ().into_any();
    }

    view! {
        <div class="automation-config-section" style=format!("--automation-config-depth:{depth};")>
            <div class="automation-config-row automation-config-row-days">
                <div class="automation-config-row-copy">
                    <p class="automation-config-row-label">{humanize_config_label(&title)}</p>
                    <p class="automation-config-row-subcopy">"Choose which days this automation repeats."</p>
                </div>
                <div class="automation-config-weekdays">
                    {weekdays
                        .into_iter()
                        .map(|(index, key, enabled)| {
                            let mut child_path = path.clone();
                            child_path.push(ConfigPathSegment::Key(key));
                            view! {
                                <button
                                    class="automation-config-day"
                                    class:is-selected=enabled
                                    disabled=disabled
                                    on:click=move |_| {
                                        set_config_bool(set_root, child_path.clone(), !enabled);
                                    }
                                >
                                    {weekday_label(index)}
                                </button>
                            }
                        })
                        .collect_view()}
                </div>
            </div>
        </div>
    }
    .into_any()
}

fn render_weekday_array(
    title: String,
    values: Vec<AutomationConfigValue>,
    path: Vec<ConfigPathSegment>,
    set_root: WriteSignal<Option<AutomationConfigValue>>,
    disabled: bool,
    depth: usize,
) -> AnyView {
    let selected = weekday_values(&values).unwrap_or_default();

    view! {
        <div class="automation-config-section" style=format!("--automation-config-depth:{depth};")>
            <div class="automation-config-row automation-config-row-days">
                <div class="automation-config-row-copy">
                    <p class="automation-config-row-label">{humanize_config_label(&title)}</p>
                    <p class="automation-config-row-subcopy">"Choose which days this automation repeats."</p>
                </div>
                <div class="automation-config-weekdays">
                    {(0..7)
                        .map(|index| {
                            let is_selected = selected.contains(&index);
                            let values_for_toggle = values.clone();
                            let path_for_toggle = path.clone();
                            view! {
                                <button
                                    class="automation-config-day"
                                    class:is-selected=is_selected
                                    disabled=disabled
                                    on:click=move |_| {
                                        toggle_weekday_array_value(
                                            set_root,
                                            path_for_toggle.clone(),
                                            values_for_toggle.clone(),
                                            index,
                                        );
                                    }
                                >
                                    {weekday_label(index)}
                                </button>
                            }
                        })
                        .collect_view()}
                </div>
            </div>
        </div>
    }
    .into_any()
}

fn render_scalar_config_value(
    label: Option<String>,
    value: AutomationConfigValue,
    path: Vec<ConfigPathSegment>,
    set_root: WriteSignal<Option<AutomationConfigValue>>,
    disabled: bool,
    depth: usize,
    on_open_time_picker: Callback<TimePickerState>,
) -> AnyView {
    let label = label.unwrap_or_else(|| "Value".to_string());
    if is_read_only_config_key(&label) {
        return view! {
            <div class="automation-config-row" style=format!("--automation-config-depth:{depth};")>
                <div class="automation-config-row-copy">
                    <p class="automation-config-row-label">{humanize_config_label(&label)}</p>
                    <p class="automation-config-row-subcopy">"Read only"</p>
                </div>
                <div class="automation-config-row-control">
                    <p class="light-subcopy automation-config-readonly-value">{config_value_summary(&value)}</p>
                </div>
            </div>
        }
        .into_any();
    }
    if let AutomationConfigValue::Bool(current) = value {
        let path_for_value = path.clone();
        return view! {
            <div class="automation-config-row automation-config-row-switch" style=format!("--automation-config-depth:{depth};")>
                <div class="automation-config-row-copy">
                    <p class="automation-config-row-label">{humanize_config_label(&label)}</p>
                    <p class="automation-config-row-subcopy">"On or off"</p>
                </div>
                <button
                    class="automation-config-switch"
                    class:is-on=current
                    disabled=disabled
                    on:click=move |_| {
                        set_config_bool(set_root, path_for_value.clone(), !current);
                    }
                    aria-label=format!("Toggle {}", humanize_config_label(&label))
                >
                    <span class="automation-config-switch-track">
                        <span class="automation-config-switch-thumb"></span>
                    </span>
                </button>
            </div>
        }
        .into_any();
    }
    let kind = scalar_kind_name(&value).to_string();
    let path_for_kind = path.clone();
    let time_like = matches!(&value, AutomationConfigValue::String(current) if is_time_like_label(&label) && looks_like_time_value(current));
    let field = match value {
        AutomationConfigValue::String(current) => {
            let path_for_value = path.clone();
            if time_like {
                let current_value = current.clone();
                let label_for_picker = label.clone();
                view! {
                    <button
                        class="secondary-button automation-time-trigger"
                        disabled=disabled
                        on:click=move |_| {
                            if let Some(state) = TimePickerState::from_string_value(
                                humanize_config_label(&label_for_picker),
                                path_for_value.clone(),
                                &current_value,
                            ) {
                                on_open_time_picker.run(state);
                            }
                        }
                    >
                        {display_time_value(&current)}
                    </button>
                }
                .into_any()
            } else {
                view! {
                    <input
                        class="automation-config-inline-input"
                        type="text"
                        prop:value=current
                        disabled=disabled
                        on:input=move |event| {
                            set_config_string(
                                set_root,
                                path_for_value.clone(),
                                event_target_value(&event),
                            );
                        }
                    />
                }
                .into_any()
            }
        }
        AutomationConfigValue::Number(current) => {
            let path_for_value = path.clone();
            view! {
                <input
                    type="text"
                    inputmode="decimal"
                    prop:value=current
                    disabled=disabled
                    on:input=move |event| {
                        set_config_number(
                            set_root,
                            path_for_value.clone(),
                            event_target_value(&event),
                        );
                    }
                />
            }
            .into_any()
        }
        AutomationConfigValue::Bool(_) => unreachable!(),
        AutomationConfigValue::Null => view! {
            <p class="light-subcopy automation-config-null">
                "Null values can be converted to another scalar type with the type selector."
            </p>
        }
        .into_any(),
        AutomationConfigValue::Object(_) | AutomationConfigValue::Array(_) => unreachable!(),
    };

    view! {
        <div class="automation-config-row" style=format!("--automation-config-depth:{depth};")>
            <div class="automation-config-row-copy">
                <p class="automation-config-row-label">{humanize_config_label(&label)}</p>
                <p class="automation-config-row-subcopy">
                    {if time_like {
                        "Time"
                    } else {
                        match kind.as_str() {
                            "string" => "Text",
                            "number" => "Number",
                            "bool" => "Boolean",
                            "null" => "Null",
                            _ => "Value",
                        }
                    }}
                </p>
            </div>
            <div class="automation-config-row-control">
                <div class="automation-config-input">{field}</div>
                <select
                    class="automation-config-kind-select"
                    prop:value=kind
                    disabled=disabled
                    on:change=move |event| {
                        set_config_scalar_kind(
                            set_root,
                            path_for_kind.clone(),
                            event_target_value(&event),
                        );
                    }
                >
                    <option value="string">"Text"</option>
                    <option value="number">"Number"</option>
                    <option value="bool">"Boolean"</option>
                    <option value="null">"Null"</option>
                </select>
            </div>
        </div>
    }
    .into_any()
}

fn config_value_at_path_mut<'a>(
    root: &'a mut AutomationConfigValue,
    path: &[ConfigPathSegment],
) -> Option<&'a mut AutomationConfigValue> {
    let Some((segment, rest)) = path.split_first() else {
        return Some(root);
    };

    match (root, segment) {
        (AutomationConfigValue::Object(entries), ConfigPathSegment::Key(key)) => entries
            .iter_mut()
            .find(|entry| entry.key == *key)
            .and_then(|entry| config_value_at_path_mut(&mut entry.value, rest)),
        (AutomationConfigValue::Array(values), ConfigPathSegment::Index(index)) => values
            .get_mut(*index)
            .and_then(|value| config_value_at_path_mut(value, rest)),
        _ => None,
    }
}

fn update_config_at_path(
    set_root: WriteSignal<Option<AutomationConfigValue>>,
    path: Vec<ConfigPathSegment>,
    update: impl FnOnce(&mut AutomationConfigValue),
) {
    set_root.update(move |root| {
        let Some(root) = root.as_mut() else {
            return;
        };
        if let Some(value) = config_value_at_path_mut(root, &path) {
            update(value);
        }
    });
}

fn set_config_string(
    set_root: WriteSignal<Option<AutomationConfigValue>>,
    path: Vec<ConfigPathSegment>,
    next: String,
) {
    update_config_at_path(set_root, path, move |value| {
        *value = AutomationConfigValue::String(next);
    });
}

fn set_config_number(
    set_root: WriteSignal<Option<AutomationConfigValue>>,
    path: Vec<ConfigPathSegment>,
    next: String,
) {
    update_config_at_path(set_root, path, move |value| {
        *value = AutomationConfigValue::Number(next);
    });
}

fn set_time_point_value(
    set_root: WriteSignal<Option<AutomationConfigValue>>,
    path: Vec<ConfigPathSegment>,
    hour: u8,
    minute: u8,
) {
    update_config_at_path(set_root, path, move |value| {
        let AutomationConfigValue::Object(entries) = value else {
            return;
        };

        for entry in entries.iter_mut() {
            match entry.key.trim().to_ascii_lowercase().as_str() {
                "hour" => entry.value = AutomationConfigValue::Number(hour.to_string()),
                "minute" => entry.value = AutomationConfigValue::Number(minute.to_string()),
                _ => {}
            }
        }
    });
}

fn set_config_bool(
    set_root: WriteSignal<Option<AutomationConfigValue>>,
    path: Vec<ConfigPathSegment>,
    next: bool,
) {
    update_config_at_path(set_root, path, move |value| {
        *value = AutomationConfigValue::Bool(next);
    });
}

fn set_config_scalar_kind(
    set_root: WriteSignal<Option<AutomationConfigValue>>,
    path: Vec<ConfigPathSegment>,
    next_kind: String,
) {
    update_config_at_path(set_root, path, move |value| {
        let current = value.clone();
        *value = match next_kind.as_str() {
            "string" => AutomationConfigValue::String(match current {
                AutomationConfigValue::String(value) => value,
                AutomationConfigValue::Number(value) => value,
                AutomationConfigValue::Bool(value) => value.to_string(),
                AutomationConfigValue::Null => String::new(),
                AutomationConfigValue::Object(_) | AutomationConfigValue::Array(_) => return,
            }),
            "number" => AutomationConfigValue::Number(match current {
                AutomationConfigValue::String(value) => value,
                AutomationConfigValue::Number(value) => value,
                AutomationConfigValue::Bool(value) => {
                    if value {
                        "1".to_string()
                    } else {
                        "0".to_string()
                    }
                }
                AutomationConfigValue::Null => "0".to_string(),
                AutomationConfigValue::Object(_) | AutomationConfigValue::Array(_) => return,
            }),
            "bool" => AutomationConfigValue::Bool(match current {
                AutomationConfigValue::String(value) => matches!(
                    value.trim().to_ascii_lowercase().as_str(),
                    "true" | "1" | "yes" | "on"
                ),
                AutomationConfigValue::Number(value) => value.trim() != "0",
                AutomationConfigValue::Bool(value) => value,
                AutomationConfigValue::Null => false,
                AutomationConfigValue::Object(_) | AutomationConfigValue::Array(_) => return,
            }),
            "null" => AutomationConfigValue::Null,
            _ => current,
        };
    });
}

fn scalar_kind_name(value: &AutomationConfigValue) -> &'static str {
    match value {
        AutomationConfigValue::String(_) => "string",
        AutomationConfigValue::Number(_) => "number",
        AutomationConfigValue::Bool(_) => "bool",
        AutomationConfigValue::Null => "null",
        AutomationConfigValue::Object(_) | AutomationConfigValue::Array(_) => "string",
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TimePointValue {
    hour: u8,
    minute: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct NestedTimePointValue {
    label: String,
    path: Vec<ConfigPathSegment>,
    time_point: TimePointValue,
}

fn is_scalar_value(value: &AutomationConfigValue) -> bool {
    matches!(
        value,
        AutomationConfigValue::String(_)
            | AutomationConfigValue::Number(_)
            | AutomationConfigValue::Bool(_)
            | AutomationConfigValue::Null
    )
}

fn visible_config_entries(
    entries: Vec<crate::hue::AutomationConfigEntry>,
) -> Vec<crate::hue::AutomationConfigEntry> {
    entries
        .into_iter()
        .filter(|entry| should_render_config_key(&entry.key))
        .collect()
}

fn time_point_value(entries: &[crate::hue::AutomationConfigEntry]) -> Option<TimePointValue> {
    let mut hour = None;
    let mut minute = None;
    let mut has_type = false;

    for entry in entries {
        match entry.key.trim().to_ascii_lowercase().as_str() {
            "hour" => {
                let AutomationConfigValue::Number(value) = &entry.value else {
                    return None;
                };
                hour = value.trim().parse::<u8>().ok();
            }
            "minute" => {
                let AutomationConfigValue::Number(value) = &entry.value else {
                    return None;
                };
                minute = value.trim().parse::<u8>().ok();
            }
            "type" => {
                let AutomationConfigValue::String(value) = &entry.value else {
                    return None;
                };
                has_type = value.trim().eq_ignore_ascii_case("time");
            }
            _ => continue,
        }
    }

    if has_type {
        Some(TimePointValue {
            hour: hour?,
            minute: minute?,
        })
    } else {
        None
    }
}

fn nested_time_point_value(
    entries: &[crate::hue::AutomationConfigEntry],
) -> Option<NestedTimePointValue> {
    entries.iter().find_map(|entry| {
        let AutomationConfigValue::Object(children) = &entry.value else {
            return None;
        };

        Some(NestedTimePointValue {
            label: entry.key.clone(),
            path: vec![ConfigPathSegment::Key(entry.key.clone())],
            time_point: time_point_value(children)?,
        })
    })
}

fn should_render_config_key(key: &str) -> bool {
    !matches!(key.trim().to_ascii_lowercase().as_str(), "rid")
}

fn is_read_only_config_key(key: &str) -> bool {
    matches!(key.trim().to_ascii_lowercase().as_str(), "rtype")
}

fn config_value_summary(value: &AutomationConfigValue) -> String {
    match value {
        AutomationConfigValue::String(value) => value.clone(),
        AutomationConfigValue::Number(value) => value.clone(),
        AutomationConfigValue::Bool(value) => {
            if *value {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        AutomationConfigValue::Null => "null".to_string(),
        AutomationConfigValue::Object(_) => "object".to_string(),
        AutomationConfigValue::Array(values) => format!(
            "{} item{}",
            values.len(),
            if values.len() == 1 { "" } else { "s" }
        ),
    }
}

fn is_time_like_label(label: &str) -> bool {
    let normalized = label.to_ascii_lowercase();
    normalized.contains("time")
        || normalized.contains("start")
        || normalized.contains("end")
        || normalized.contains("sunrise")
        || normalized.contains("sunset")
}

fn looks_like_time_value(value: &str) -> bool {
    let trimmed = value.trim();
    let mut segments = trimmed.split(':');
    let Some(hours) = segments.next() else {
        return false;
    };
    let Some(minutes) = segments.next() else {
        return false;
    };
    if hours.len() != 2
        || minutes.len() != 2
        || !hours.chars().all(|ch| ch.is_ascii_digit())
        || !minutes.chars().all(|ch| ch.is_ascii_digit())
    {
        return false;
    }
    segments
        .next()
        .is_none_or(|seconds| seconds.len() == 2 && seconds.chars().all(|ch| ch.is_ascii_digit()))
}

fn display_time_value(value: &str) -> String {
    value.trim().chars().take(5).collect()
}

impl TimePickerState {
    fn from_string_value(label: String, path: Vec<ConfigPathSegment>, value: &str) -> Option<Self> {
        let trimmed = value.trim();
        let mut segments = trimmed.split(':');
        let hour = segments.next()?.parse::<u8>().ok()?;
        let minute = segments.next()?.parse::<u8>().ok()?;
        let include_seconds = segments.next().is_some();

        Some(Self {
            label,
            path,
            hour,
            minute,
            include_seconds,
            mode: TimePickerMode::Hour,
            target: TimePickerTarget::StringValue,
        })
    }

    fn from_time_point(label: String, path: Vec<ConfigPathSegment>, hour: u8, minute: u8) -> Self {
        Self {
            label,
            path,
            hour,
            minute,
            include_seconds: false,
            mode: TimePickerMode::Hour,
            target: TimePickerTarget::TimePointObject,
        }
    }

    fn formatted_display(&self) -> String {
        format!("{:02}:{:02}", self.hour, self.minute)
    }

    fn formatted_value(&self) -> String {
        if self.include_seconds {
            format!("{:02}:{:02}:00", self.hour, self.minute)
        } else {
            self.formatted_display()
        }
    }
}

fn weekday_entries(
    entries: &[crate::hue::AutomationConfigEntry],
) -> Option<Vec<(usize, String, bool)>> {
    if entries.is_empty() {
        return None;
    }

    let mut normalized = entries
        .iter()
        .filter_map(|entry| {
            let AutomationConfigValue::Bool(value) = entry.value else {
                return None;
            };
            weekday_index(&entry.key).map(|index| (index, entry.key.clone(), value))
        })
        .collect::<Vec<_>>();

    if normalized.len() != entries.len() {
        return None;
    }

    normalized.sort_by_key(|(index, _, _)| *index);
    Some(normalized)
}

fn weekday_values(values: &[AutomationConfigValue]) -> Option<Vec<usize>> {
    if values.is_empty() {
        return None;
    }

    let weekdays = values
        .iter()
        .map(|value| match value {
            AutomationConfigValue::String(value) => weekday_index(value),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()?;

    Some(weekdays)
}

fn weekday_index(value: &str) -> Option<usize> {
    match value.trim().to_ascii_lowercase().as_str() {
        "sun" | "sunday" => Some(0),
        "mon" | "monday" => Some(1),
        "tue" | "tues" | "tuesday" => Some(2),
        "wed" | "wednesday" => Some(3),
        "thu" | "thurs" | "thursday" => Some(4),
        "fri" | "friday" => Some(5),
        "sat" | "saturday" => Some(6),
        _ => None,
    }
}

fn weekday_label(index: usize) -> &'static str {
    match index {
        0 => "S",
        1 => "M",
        2 => "T",
        3 => "W",
        4 => "T",
        5 => "F",
        6 => "S",
        _ => "?",
    }
}

fn toggle_weekday_array_value(
    set_root: WriteSignal<Option<AutomationConfigValue>>,
    path: Vec<ConfigPathSegment>,
    current_values: Vec<AutomationConfigValue>,
    toggled_index: usize,
) {
    let style = infer_weekday_style(&current_values);
    let encoded = encode_weekday(toggled_index, style);
    let current_selection = weekday_values(&current_values).unwrap_or_default();

    update_config_at_path(set_root, path, move |value| {
        let AutomationConfigValue::Array(values) = value else {
            return;
        };

        if current_selection.contains(&toggled_index) {
            values.retain(|item| {
                !matches!(item, AutomationConfigValue::String(existing) if weekday_index(existing) == Some(toggled_index))
            });
        } else {
            values.push(AutomationConfigValue::String(encoded.clone()));
        }
    });
}

fn infer_weekday_style(values: &[AutomationConfigValue]) -> WeekdayStyle {
    values
        .iter()
        .find_map(|value| match value {
            AutomationConfigValue::String(value) => Some(match value.trim() {
                value if value.len() <= 3 => WeekdayStyle::ShortLower,
                _ => WeekdayStyle::LongLower,
            }),
            _ => None,
        })
        .unwrap_or(WeekdayStyle::LongLower)
}

fn encode_weekday(index: usize, style: WeekdayStyle) -> String {
    match style {
        WeekdayStyle::ShortLower => match index {
            0 => "sun",
            1 => "mon",
            2 => "tue",
            3 => "wed",
            4 => "thu",
            5 => "fri",
            6 => "sat",
            _ => "sun",
        }
        .to_string(),
        WeekdayStyle::LongLower => match index {
            0 => "sunday",
            1 => "monday",
            2 => "tuesday",
            3 => "wednesday",
            4 => "thursday",
            5 => "friday",
            6 => "saturday",
            _ => "sunday",
        }
        .to_string(),
    }
}

#[derive(Clone, Copy)]
enum WeekdayStyle {
    ShortLower,
    LongLower,
}

fn humanize_config_label(value: &str) -> String {
    value
        .split(['_', '-', '.'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn automation_icon_class(automation: &Automation) -> &'static str {
    let name = automation.name.to_ascii_lowercase();

    if name.contains("sleep") || name.contains("night") {
        "fa-solid fa-moon"
    } else if name.contains("wake") || name.contains("morning") || name.contains("sunrise") {
        "fa-solid fa-sun"
    } else if name.contains("timer") || name.contains("schedule") {
        "fa-solid fa-clock"
    } else if name.contains("away") || name.contains("security") {
        "fa-solid fa-shield-halved"
    } else {
        "fa-solid fa-wand-magic-sparkles"
    }
}
