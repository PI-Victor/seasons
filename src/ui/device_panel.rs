use crate::hue::{Group, Light, Sensor};
use leptos::prelude::*;
use std::collections::HashSet;

use super::DeviceGrid;

#[component]
pub fn DevicePanel(
    lights: ReadSignal<Vec<Light>>,
    sensors: ReadSignal<Vec<Sensor>>,
    groups: ReadSignal<Vec<Group>>,
    pending_light_ids: ReadSignal<HashSet<String>>,
    on_toggle_light: Callback<String>,
    on_set_light_brightness: Callback<(String, u8)>,
) -> impl IntoView {
    let ordered_lights = Signal::derive(move || {
        let mut lights = lights.get();
        lights.sort_by(|left, right| left.name.cmp(&right.name));
        lights
    });

    view! {
        <section class="surface-panel device-panel">
            <div class="panel-header compact-panel-header devices-panel-header">
                <div>
                    <p class="panel-kicker">"Devices"</p>
                    <h2>"All devices"</h2>
                </div>
                <p class="panel-copy">"Every connected light in one place, without room collapse getting in the way."</p>
            </div>

            {move || {
                let lights = ordered_lights.get();
                let mut sensors = sensors.get();
                sensors.sort_by(|left, right| left.name.cmp(&right.name));

                if lights.is_empty() && sensors.is_empty() {
                    view! {
                        <div class="empty-state compact-empty-state">
                            <h3>"No devices found"</h3>
                            <p>"The current bridge snapshot did not include any controllable devices."</p>
                        </div>
                    }.into_any()
                } else {
                    let (plug_lights, regular_lights): (Vec<_>, Vec<_>) = lights
                        .into_iter()
                        .partition(is_plug_device);
                    let (switch_sensors, other_sensors): (Vec<_>, Vec<_>) = sensors
                        .into_iter()
                        .partition(is_switch_sensor);

                    view! {
                        <div class="device-type-stack">
                            {if !regular_lights.is_empty() {
                                view! {
                                    <section class="device-type-section">
                                        <div class="device-type-header">
                                            <p class="panel-kicker">"Lights"</p>
                                            <span class="device-type-count">{format!("{} device{}", regular_lights.len(), if regular_lights.len() == 1 { "" } else { "s" })}</span>
                                        </div>
                                        <DeviceGrid
                                            lights=regular_lights
                                            groups=groups.get()
                                            pending_light_ids=pending_light_ids
                                            on_toggle_light=on_toggle_light
                                            on_set_light_brightness=on_set_light_brightness
                                        />
                                    </section>
                                }.into_any()
                            } else {
                                ().into_any()
                            }}

                            {if !plug_lights.is_empty() {
                                view! {
                                    <section class="device-type-section">
                                        <div class="device-type-header">
                                            <p class="panel-kicker">"Plugs"</p>
                                            <span class="device-type-count">{format!("{} device{}", plug_lights.len(), if plug_lights.len() == 1 { "" } else { "s" })}</span>
                                        </div>
                                        <DeviceGrid
                                            lights=plug_lights
                                            groups=groups.get()
                                            pending_light_ids=pending_light_ids
                                            on_toggle_light=on_toggle_light
                                            on_set_light_brightness=on_set_light_brightness
                                        />
                                    </section>
                                }.into_any()
                            } else {
                                ().into_any()
                            }}

                            {if !switch_sensors.is_empty() {
                                view! {
                                    <section class="device-type-section">
                                        <div class="device-type-header">
                                            <p class="panel-kicker">"Switches"</p>
                                            <span class="device-type-count">{format!("{} switch{}", switch_sensors.len(), if switch_sensors.len() == 1 { "" } else { "es" })}</span>
                                        </div>
                                        <div class="sensor-list">
                                            {switch_sensors
                                                .into_iter()
                                                .map(|sensor| {
                                                    let sensor_type = sensor
                                                        .sensor_type
                                                        .clone()
                                                        .unwrap_or_else(|| "Hue switch".to_string());
                                                    let reachable_text = if sensor.reachable.unwrap_or(true) {
                                                        "Reachable".to_string()
                                                    } else {
                                                        "Unavailable".to_string()
                                                    };
                                                    let battery_text = sensor
                                                        .battery
                                                        .map(|battery| format!("Battery {battery}%"))
                                                        .unwrap_or_else(|| reachable_text.clone());
                                                    let summary = sensor
                                                        .summary
                                                        .clone()
                                                        .or(sensor.last_updated.clone())
                                                        .unwrap_or_else(|| "No recent activity".to_string());
                                                    let icon_class = sensor_icon_class(&sensor);

                                                    view! {
                                                        <article class="light-card compact-light-card sensor-card">
                                                            <div class="light-card-top">
                                                                <div class="light-card-identity">
                                                                    <span class="light-icon-shell sensor-icon-shell">
                                                                        <span class=format!("{icon_class} fa-fw light-icon-glyph") aria-hidden="true"></span>
                                                                    </span>
                                                                    <div class="light-card-copy">
                                                                        <p class="light-eyebrow">{sensor_type}</p>
                                                                        <h3>{sensor.name}</h3>
                                                                        <p class="light-subcopy">{summary}</p>
                                                                    </div>
                                                                </div>
                                                            </div>

                                                            <div class="light-meta-cluster">
                                                                <span class="light-meta-chip">{battery_text}</span>
                                                                <span class="light-meta-chip">{reachable_text}</span>
                                                            </div>
                                                        </article>
                                                    }
                                                })
                                                .collect_view()}
                                        </div>
                                    </section>
                                }.into_any()
                            } else {
                                ().into_any()
                            }}

                            {if !other_sensors.is_empty() {
                                view! {
                                    <section class="device-type-section">
                                        <div class="device-type-header">
                                            <p class="panel-kicker">"Sensors"</p>
                                            <span class="device-type-count">{format!("{} sensor{}", other_sensors.len(), if other_sensors.len() == 1 { "" } else { "s" })}</span>
                                        </div>
                                        <div class="sensor-list">
                                            {other_sensors
                                                .into_iter()
                                                .map(|sensor| {
                                                    let sensor_type = sensor
                                                        .sensor_type
                                                        .clone()
                                                        .unwrap_or_else(|| "Hue sensor".to_string());
                                                    let reachable_text = if sensor.reachable.unwrap_or(true) {
                                                        "Reachable".to_string()
                                                    } else {
                                                        "Unavailable".to_string()
                                                    };
                                                    let battery_text = sensor
                                                        .battery
                                                        .map(|battery| format!("Battery {battery}%"))
                                                        .unwrap_or_else(|| reachable_text.clone());
                                                    let summary = sensor
                                                        .summary
                                                        .clone()
                                                        .or(sensor.last_updated.clone())
                                                        .unwrap_or_else(|| "No recent activity".to_string());
                                                    let icon_class = sensor_icon_class(&sensor);

                                                    view! {
                                                        <article class="light-card compact-light-card sensor-card">
                                                            <div class="light-card-top">
                                                                <div class="light-card-identity">
                                                                    <span class="light-icon-shell sensor-icon-shell">
                                                                        <span class=format!("{icon_class} fa-fw light-icon-glyph") aria-hidden="true"></span>
                                                                    </span>
                                                                    <div class="light-card-copy">
                                                                        <p class="light-eyebrow">{sensor_type}</p>
                                                                        <h3>{sensor.name}</h3>
                                                                        <p class="light-subcopy">{summary}</p>
                                                                    </div>
                                                                </div>
                                                            </div>

                                                            <div class="light-meta-cluster">
                                                                <span class="light-meta-chip">{battery_text}</span>
                                                                <span class="light-meta-chip">{reachable_text}</span>
                                                            </div>
                                                        </article>
                                                    }
                                                })
                                                .collect_view()}
                                        </div>
                                    </section>
                                }.into_any()
                            } else {
                                ().into_any()
                            }}
                        </div>
                    }.into_any()
                }
            }}
        </section>
    }
}

fn is_plug_device(light: &Light) -> bool {
    light
        .light_type
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase()
        .contains("plug")
}

fn is_switch_sensor(sensor: &Sensor) -> bool {
    let sensor_type = sensor
        .sensor_type
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();

    sensor_type.contains("switch") || sensor_type.contains("tap") || sensor_type.contains("button")
}

fn sensor_icon_class(sensor: &Sensor) -> &'static str {
    let sensor_type = sensor
        .sensor_type
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();

    if sensor_type.contains("motion") {
        "fa-solid fa-person-rays"
    } else if sensor_type.contains("temperature") {
        "fa-solid fa-temperature-half"
    } else if sensor_type.contains("switch") || sensor_type.contains("tap") {
        "fa-solid fa-toggle-on"
    } else if sensor_type.contains("lightlevel") || sensor_type.contains("daylight") {
        "fa-solid fa-sun"
    } else {
        "fa-solid fa-wave-square"
    }
}
