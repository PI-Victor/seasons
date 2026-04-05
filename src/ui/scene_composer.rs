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

use js_sys::Reflect;
use leptos::ev::MouseEvent;
use leptos::prelude::*;
use wasm_bindgen::{JsCast, JsValue};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SceneComposerRequest {
    pub room_id: String,
    pub scene_name: String,
    pub hue_degrees: u16,
    pub saturation: u8,
    pub brightness: u8,
}

#[component]
pub fn SceneComposer(
    room_id: String,
    pending: bool,
    on_submit: Callback<SceneComposerRequest>,
) -> impl IntoView {
    let (scene_name, set_scene_name) = signal(String::new());
    let (hue_degrees, set_hue_degrees) = signal(32_u16);
    let (saturation, set_saturation) = signal(196_u8);
    let (brightness, set_brightness) = signal(188_u8);

    let wheel_style = move || {
        let angle = hue_degrees.get();
        let strength = f32::from(saturation.get()) / 254.0;
        let thumb_radius = 34.0 * strength;
        let radians = f32::from(angle).to_radians();
        let x = 42.0 + thumb_radius * radians.cos();
        let y = 42.0 + thumb_radius * radians.sin();
        let preview = composer_preview_color(angle, saturation.get(), brightness.get());

        format!(
            "--composer-thumb-x: {x:.2}px; --composer-thumb-y: {y:.2}px; --composer-preview: {preview};"
        )
    };

    let save_scene = move |_| {
        let trimmed_name = scene_name.get_untracked().trim().to_string();
        if trimmed_name.is_empty() {
            return;
        }

        on_submit.run(SceneComposerRequest {
            room_id: room_id.clone(),
            scene_name: trimmed_name,
            hue_degrees: hue_degrees.get_untracked(),
            saturation: saturation.get_untracked(),
            brightness: brightness.get_untracked(),
        });
    };

    view! {
        <div class="scene-composer">
            <div class="scene-composer-wheel-block">
                <button
                    class="scene-composer-wheel"
                    style=wheel_style
                    type="button"
                    on:click=move |event| {
                        if let Some((angle, next_saturation)) = read_wheel_selection(&event) {
                            set_hue_degrees.set(angle);
                            set_saturation.set(next_saturation.max(18));
                        }
                    }
                >
                    <span class="scene-composer-wheel-core"></span>
                    <span class="scene-composer-wheel-thumb"></span>
                </button>
                <div class="scene-composer-preview">
                    <span class="scene-composer-preview-dot" style=move || {
                        let preview = composer_preview_color(
                            hue_degrees.get(),
                            saturation.get(),
                            brightness.get(),
                        );
                        format!("background: {preview};")
                    }></span>
                    <div class="scene-composer-preview-copy">
                        <strong>{move || format!("{}%", brightness_percent(brightness.get()))}</strong>
                        <small>{move || format!("sat {}%", saturation_percent(saturation.get()))}</small>
                    </div>
                </div>
            </div>

            <div class="scene-composer-fields">
                <label class="field scene-composer-field">
                    <span class="field-label">"Scene name"</span>
                    <input
                        type="text"
                        placeholder="Velvet Sunset"
                        prop:value=scene_name
                        on:input=move |event| set_scene_name.set(event_target_value(&event))
                    />
                </label>

                <label class="field scene-composer-field">
                    <span class="field-label">"Brightness"</span>
                    <input
                        class="brightness-slider"
                        type="range"
                        min="1"
                        max="254"
                        value=move || brightness.get().to_string()
                        on:input=move |event| {
                            if let Ok(value) = event_target_value(&event).parse::<u8>() {
                                set_brightness.set(value.max(1));
                            }
                        }
                    />
                </label>
            </div>

            <div class="scene-composer-actions">
                <button
                    class="secondary-button compact-button"
                    type="button"
                    disabled=move || pending || scene_name.get().trim().is_empty()
                    on:click=save_scene
                >
                    {move || if pending { "Saving..." } else { "Save scene" }}
                </button>
            </div>
        </div>
    }
}

fn read_wheel_selection(event: &MouseEvent) -> Option<(u16, u8)> {
    let target = event.target()?;
    let target: JsValue = target.into();
    let rect_fn = Reflect::get(&target, &JsValue::from_str("getBoundingClientRect"))
        .ok()?
        .dyn_into::<js_sys::Function>()
        .ok()?;
    let rect = rect_fn.call0(&target).ok()?;

    let left = Reflect::get(&rect, &JsValue::from_str("left"))
        .ok()?
        .as_f64()?;
    let top = Reflect::get(&rect, &JsValue::from_str("top"))
        .ok()?
        .as_f64()?;
    let width = Reflect::get(&rect, &JsValue::from_str("width"))
        .ok()?
        .as_f64()?;
    let height = Reflect::get(&rect, &JsValue::from_str("height"))
        .ok()?
        .as_f64()?;

    let radius = width.min(height) / 2.0;
    if radius <= f64::EPSILON {
        return None;
    }

    let dx = f64::from(event.client_x()) - left - radius;
    let dy = f64::from(event.client_y()) - top - radius;
    let distance = (dx * dx + dy * dy).sqrt();
    let normalized = (distance / radius).clamp(0.0, 1.0);

    let mut angle = dy.atan2(dx).to_degrees();
    if angle < 0.0 {
        angle += 360.0;
    }

    Some((angle.round() as u16, (normalized * 254.0).round() as u8))
}

fn brightness_percent(value: u8) -> u8 {
    ((f32::from(value) / 254.0) * 100.0).round() as u8
}

fn saturation_percent(value: u8) -> u8 {
    ((f32::from(value) / 254.0) * 100.0).round() as u8
}

fn composer_preview_color(hue_degrees: u16, saturation: u8, brightness: u8) -> String {
    let (red, green, blue) = hsv_to_rgb_float(
        f32::from(hue_degrees),
        f32::from(saturation) / 254.0,
        f32::from(brightness.max(1)) / 254.0,
    );
    format!("rgb({red} {green} {blue})")
}

fn hsv_to_rgb_float(hue: f32, saturation: f32, value: f32) -> (u8, u8, u8) {
    let hue = wrap_hue(hue);

    if saturation <= f32::EPSILON {
        let channel = (value * 255.0).round() as u8;
        return (channel, channel, channel);
    }

    let chroma = value * saturation;
    let hue_sector = hue / 60.0;
    let secondary = chroma * (1.0 - ((hue_sector % 2.0) - 1.0).abs());
    let match_value = value - chroma;

    let (red, green, blue) = match hue_sector as u8 {
        0 => (chroma, secondary, 0.0),
        1 => (secondary, chroma, 0.0),
        2 => (0.0, chroma, secondary),
        3 => (0.0, secondary, chroma),
        4 => (secondary, 0.0, chroma),
        _ => (chroma, 0.0, secondary),
    };

    (
        ((red + match_value) * 255.0).round() as u8,
        ((green + match_value) * 255.0).round() as u8,
        ((blue + match_value) * 255.0).round() as u8,
    )
}

fn wrap_hue(hue: f32) -> f32 {
    let wrapped = hue % 360.0;
    if wrapped < 0.0 {
        wrapped + 360.0
    } else {
        wrapped
    }
}
