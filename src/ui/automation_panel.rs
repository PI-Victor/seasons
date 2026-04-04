use crate::hue::{self, Automation, AutomationDetail, BridgeConnection};
use leptos::{prelude::*, task::spawn_local};
use std::collections::HashSet;

#[component]
pub fn AutomationPanel(
    active_connection: ReadSignal<Option<BridgeConnection>>,
    automations: ReadSignal<Vec<Automation>>,
    pending_automation_ids: ReadSignal<HashSet<String>>,
    on_toggle_automation: Callback<(String, bool)>,
) -> impl IntoView {
    let (selected_automation_id, set_selected_automation_id) = signal(None::<String>);
    let (selected_automation_detail, set_selected_automation_detail) =
        signal(None::<AutomationDetail>);
    let (is_loading_detail, set_is_loading_detail) = signal(false);
    let (detail_error, set_detail_error) = signal(None::<String>);
    let ordered_automations = Signal::derive(move || {
        let mut automations = automations.get();
        automations.sort_by(|left, right| left.name.cmp(&right.name));
        automations
    });
    let selected_automation = Signal::derive(move || {
        let selected_id = selected_automation_id.get();
        let automations = ordered_automations.get();

        selected_id
            .as_deref()
            .and_then(|id| automations.iter().find(|automation| automation.id == id))
            .cloned()
            .or_else(|| automations.into_iter().next())
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

    view! {
        <section class="surface-panel automation-panel">
            <div class="panel-header compact-panel-header devices-panel-header">
                <div>
                    <p class="panel-kicker">"Automations"</p>
                    <h2>"Bridge automations"</h2>
                </div>
                <p class="panel-copy">
                    "Schedules and routines exposed by the bridge. Toggle them without leaving the app."
                </p>
            </div>

            {move || {
                let automations = ordered_automations.get();

                if automations.is_empty() {
                    view! {
                        <div class="empty-state compact-empty-state">
                            <h3>"No automations found"</h3>
                            <p>"This bridge did not expose any toggleable Hue automations."</p>
                        </div>
                    }.into_any()
                } else {
                    let detail = selected_automation.get();
                    view! {
                        <div class="automation-stack">
                            {detail.map(|automation| {
                                let automation_id = automation.id.clone();
                                let icon_class = automation_icon_class(&automation);
                                let subtitle = automation
                                    .script_id
                                    .clone()
                                    .unwrap_or_else(|| "Hue automation".to_string());
                                let detail = selected_automation_detail.get();
                                let enabled = detail
                                    .as_ref()
                                    .and_then(|detail| detail.enabled)
                                    .or(automation.enabled)
                                    .unwrap_or(false);
                                let is_pending = pending_automation_ids.get().contains(&automation.id);
                                let can_toggle = detail
                                    .as_ref()
                                    .map(|detail| detail.enabled.is_some())
                                    .unwrap_or_else(|| automation.enabled.is_some());

                                view! {
                                    <article class="automation-detail-card">
                                        <div class="automation-card-top automation-detail-top">
                                            <div class="automation-card-identity">
                                                <span class="light-icon-shell automation-icon-shell">
                                                    <span class=format!("{icon_class} fa-fw light-icon-glyph") aria-hidden="true"></span>
                                                </span>
                                                <div class="light-card-copy">
                                                    <p class="light-eyebrow">"Selected automation"</p>
                                                    <h3>{automation.name}</h3>
                                                    <p class="light-subcopy" title=subtitle.clone()>{subtitle.clone()}</p>
                                                </div>
                                            </div>
                                            <button
                                                class="light-status-button"
                                                class:is-off=move || !enabled
                                                disabled=move || is_pending || !can_toggle
                                                on:click=move |_| {
                                                    on_toggle_automation.run((automation_id.clone(), !enabled));
                                                }
                                            >
                                                <span class="status-dot"></span>
                                                <span>
                                                    {if can_toggle {
                                                        if enabled { "Turn off" } else { "Turn on" }
                                                    } else {
                                                        "Read only"
                                                    }}
                                                </span>
                                            </button>
                                        </div>

                                        <div class="automation-detail-meta">
                                            <span class="light-meta-chip">{format!("ID {}", automation.id)}</span>
                                            <span class="light-meta-chip">
                                                {if enabled { "Enabled" } else { "Disabled" }}
                                            </span>
                                            {automation
                                                .script_id
                                                .clone()
                                                .map(|script_id| view! {
                                                    <span class="light-meta-chip" title=script_id.clone()>{script_id.clone()}</span>
                                                })}
                                        </div>

                                        <div class="automation-detail-sections">
                                            {move || {
                                                if is_loading_detail.get() {
                                                    view! {
                                                        <div class="automation-detail-block">
                                                            <p class="light-eyebrow">"Rules"</p>
                                                            <p class="light-subcopy">"Loading bridge automation payload…"</p>
                                                        </div>
                                                    }.into_any()
                                                } else if let Some(error) = detail_error.get() {
                                                    view! {
                                                        <div class="automation-detail-block">
                                                            <p class="light-eyebrow">"Rules"</p>
                                                            <p class="light-subcopy">{error}</p>
                                                        </div>
                                                    }.into_any()
                                                } else if let Some(detail) = selected_automation_detail.get() {
                                                    let instance_json = detail.instance_json.clone();
                                                    let script_json = detail.script_json.clone();
                                                    view! {
                                                        <>
                                                            <div class="automation-detail-block">
                                                                <p class="light-eyebrow">"Behavior instance"</p>
                                                                <pre class="automation-json">{instance_json}</pre>
                                                            </div>
                                                            {script_json.map(|script_json| view! {
                                                                <div class="automation-detail-block">
                                                                    <p class="light-eyebrow">"Behavior script"</p>
                                                                    <pre class="automation-json">{script_json}</pre>
                                                                </div>
                                                            })}
                                                        </>
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <div class="automation-detail-block">
                                                            <p class="light-eyebrow">"Rules"</p>
                                                            <p class="light-subcopy">"No detailed bridge payload is available for this automation."</p>
                                                        </div>
                                                    }.into_any()
                                                }
                                            }}
                                        </div>
                                    </article>
                                }
                            })}

                            <div class="automation-list">
                                {automations
                                    .into_iter()
                                    .map(|automation| {
                                        let automation_id = automation.id.clone();
                                        let automation_id_for_class = automation_id.clone();
                                        let automation_id_for_select = automation_id.clone();
                                        let automation_id_for_toggle = automation_id.clone();
                                        let selected_id = selected_automation_id.get();
                                        let enabled = automation.enabled.unwrap_or(false);
                                        let icon_class = automation_icon_class(&automation);
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
                                                class=move || {
                                                    if selected_id.as_deref()
                                                        == Some(automation_id_for_class.as_str())
                                                    {
                                                        "automation-card automation-card-button is-selected"
                                                    } else {
                                                        "automation-card automation-card-button"
                                                    }
                                                }
                                                on:click=move |_| {
                                                    set_selected_automation_id
                                                        .set(Some(automation_id_for_select.clone()))
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
                                                            on_toggle_automation.run((
                                                                automation_id_for_toggle.clone(),
                                                                !enabled,
                                                            ));
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
                                            </article>
                                        }
                                    })
                                    .collect_view()}
                            </div>
                        </div>
                    }.into_any()
                }
            }}
        </section>
    }
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
