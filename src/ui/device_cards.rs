use crate::hue::{Group, GroupKind, Light};
use leptos::prelude::*;
use std::collections::HashSet;

#[component]
pub fn DeviceGrid(
    lights: Vec<Light>,
    groups: Vec<Group>,
    pending_light_ids: ReadSignal<HashSet<String>>,
    on_toggle_light: Callback<String>,
    on_set_light_brightness: Callback<(String, u8)>,
) -> impl IntoView {
    view! {
        <div class="device-list">
            {lights
                .into_iter()
                .map(|light| {
                    let placement = derive_placement(&light.id, &groups);
                    let light_name = light.name.clone();
                    let light_type = light
                        .light_type
                        .clone()
                        .unwrap_or_else(|| "Hue light".to_string());
                    let reachable_text = if light.reachable.unwrap_or(true) {
                        "Reachable"
                    } else {
                        "Unavailable"
                    };
                    let light_title = light_name.clone();
                    let is_on = light.is_on.unwrap_or(false);
                    let brightness_text = brightness_label(light.brightness.unwrap_or(0));
                    let brightness_value = light.brightness.unwrap_or(1).max(1);
                    let light_style = light_accent_style(&light).unwrap_or_default();
                    let toggle_light_id = light.id.clone();
                    let set_light_brightness_id = light.id.clone();
                    let is_pending = pending_light_ids.get().contains(&light.id);
                    let light_icon_class = device_icon_class(&light);
                    let zone_text = if placement.zone_names.is_empty() {
                        "No zones".to_string()
                    } else {
                        placement.zone_names.join(", ")
                    };

                    view! {
                        <article class="light-card compact-light-card" style=light_style>
                            <div class="light-card-top">
                                <div class="light-card-identity">
                                    <span class="light-icon-shell">
                                        <span class=format!("{light_icon_class} fa-fw light-icon-glyph") aria-hidden="true"></span>
                                    </span>
                                    <div class="light-card-copy">
                                        <p class="light-eyebrow">{light_type}</p>
                                        <h3 title=light_title>{light_name}</h3>
                                        <p class="light-subcopy">{zone_text}</p>
                                    </div>
                                </div>
                                <button
                                    class="light-status-button"
                                    class:is-off=move || !is_on
                                    disabled=is_pending
                                    on:click=move |_| on_toggle_light.run(toggle_light_id.clone())
                                >
                                    <span class="status-dot"></span>
                                    <span>{if is_on { "On" } else { "Off" }}</span>
                                </button>
                            </div>

                            <div class="light-meta-cluster">
                                <span class="light-meta-chip">{brightness_text.clone()}</span>
                                <span class="light-meta-chip">{reachable_text}</span>
                            </div>

                            <div class="device-inline-control">
                                <div class="device-inline-copy">
                                    <span class="device-inline-label">"Brightness"</span>
                                    <strong>{brightness_text}</strong>
                                </div>
                                <input
                                    class="brightness-slider device-brightness-slider"
                                    type="range"
                                    min="1"
                                    max="254"
                                    value=brightness_value.to_string()
                                    disabled=is_pending || !light.reachable.unwrap_or(true)
                                    on:change=move |ev| {
                                        if let Ok(value) = event_target_value(&ev).parse::<u8>() {
                                            on_set_light_brightness.run((set_light_brightness_id.clone(), value));
                                        }
                                    }
                                />
                            </div>
                        </article>
                    }
                })
                .collect_view()}
        </div>
    }
}

#[derive(Default)]
struct LightPlacement {
    zone_names: Vec<String>,
}

fn derive_placement(light_id: &str, groups: &[Group]) -> LightPlacement {
    let mut placement = LightPlacement::default();

    for group in groups {
        if !group.light_ids.iter().any(|id| id == light_id) {
            continue;
        }

        if matches!(group.kind, GroupKind::Zone) {
            placement.zone_names.push(group.name.clone());
        }
    }

    placement.zone_names.sort();
    placement
}

fn brightness_label(value: u8) -> String {
    let percentage = (u16::from(value) * 100) / 254;
    format!("{percentage}%")
}

fn light_accent_style(light: &Light) -> Option<String> {
    light_accent_rgb(light)
        .map(|(red, green, blue)| format!("--bridge-accent: rgb({red} {green} {blue});"))
}

fn light_accent_rgb(light: &Light) -> Option<(u8, u8, u8)> {
    let brightness = light.brightness?;
    if brightness == 0 {
        return None;
    }

    if let Some([x, y]) = light.xy {
        if let Some(rgb) = xy_to_ui_rgb(x, y, brightness) {
            return Some(rgb);
        }
    }

    let hue = light.hue.unwrap_or(8_000);
    let saturation = light.saturation.unwrap_or(40);
    Some(hsv_to_ui_rgb(hue, saturation, brightness))
}

fn device_icon_class(light: &Light) -> &'static str {
    let light_type = light
        .light_type
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();

    if light_type.contains("strip") {
        "fa-solid fa-grip-lines-vertical"
    } else if light_type.contains("plug") {
        "fa-solid fa-plug"
    } else {
        "fa-solid fa-lightbulb"
    }
}

fn hsv_to_ui_rgb(hue: u16, saturation: u8, brightness: u8) -> (u8, u8, u8) {
    let hue = f32::from(hue) * 360.0 / 65_535.0;
    let saturation = (f32::from(saturation) / 254.0).clamp(0.38, 0.96);
    let brightness_ratio = f32::from(brightness) / 254.0;
    let value = (0.86 + brightness_ratio * 0.12).clamp(0.86, 0.98);

    hsv_to_rgb_float(hue, saturation, value)
}

fn xy_to_ui_rgb(x: f32, y: f32, brightness: u8) -> Option<(u8, u8, u8)> {
    if !(0.0..=1.0).contains(&x) || !(0.0..=1.0).contains(&y) || y <= f32::EPSILON {
        return None;
    }

    let z = 1.0 - x - y;
    if z < 0.0 {
        return None;
    }

    let luminance = 0.82 + (f32::from(brightness) / 254.0) * 0.16;
    let x_component = (luminance / y) * x;
    let z_component = (luminance / y) * z;

    let mut red = x_component * 1.612 - luminance * 0.203 - z_component * 0.302;
    let mut green = -x_component * 0.509 + luminance * 1.412 + z_component * 0.066;
    let mut blue = x_component * 0.026 - luminance * 0.072 + z_component * 0.962;

    red = if red <= 0.003_130_8 {
        12.92 * red
    } else {
        1.055 * red.powf(1.0 / 2.4) - 0.055
    };
    green = if green <= 0.003_130_8 {
        12.92 * green
    } else {
        1.055 * green.powf(1.0 / 2.4) - 0.055
    };
    blue = if blue <= 0.003_130_8 {
        12.92 * blue
    } else {
        1.055 * blue.powf(1.0 / 2.4) - 0.055
    };

    let max_channel = red.max(green).max(blue);
    if max_channel <= f32::EPSILON {
        return None;
    }

    Some((
        ((red / max_channel).clamp(0.0, 1.0) * 255.0).round() as u8,
        ((green / max_channel).clamp(0.0, 1.0) * 255.0).round() as u8,
        ((blue / max_channel).clamp(0.0, 1.0) * 255.0).round() as u8,
    ))
}

fn hsv_to_rgb_float(hue: f32, saturation: f32, value: f32) -> (u8, u8, u8) {
    let chroma = value * saturation;
    let scaled_hue = hue / 60.0;
    let secondary = chroma * (1.0 - ((scaled_hue % 2.0) - 1.0).abs());
    let match_value = value - chroma;

    let (red, green, blue) = if (0.0..1.0).contains(&scaled_hue) {
        (chroma, secondary, 0.0)
    } else if (1.0..2.0).contains(&scaled_hue) {
        (secondary, chroma, 0.0)
    } else if (2.0..3.0).contains(&scaled_hue) {
        (0.0, chroma, secondary)
    } else if (3.0..4.0).contains(&scaled_hue) {
        (0.0, secondary, chroma)
    } else if (4.0..5.0).contains(&scaled_hue) {
        (secondary, 0.0, chroma)
    } else {
        (chroma, 0.0, secondary)
    };

    (
        ((red + match_value) * 255.0).round() as u8,
        ((green + match_value) * 255.0).round() as u8,
        ((blue + match_value) * 255.0).round() as u8,
    )
}
