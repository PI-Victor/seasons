use super::{DeviceGrid, SceneComposer, SceneComposerRequest};
use crate::hue::{
    ActivateSceneRequest, BridgeConnection, DeleteSceneRequest, Group, GroupKind, Light, Scene,
};
use leptos::prelude::*;
use std::collections::{HashMap, HashSet};

#[component]
pub fn LightGrid(
    lights: ReadSignal<Vec<Light>>,
    groups: ReadSignal<Vec<Group>>,
    scenes: ReadSignal<Vec<Scene>>,
    room_order: ReadSignal<Vec<String>>,
    pending_scene_id: ReadSignal<Option<String>>,
    pending_room_ids: ReadSignal<HashSet<String>>,
    pending_room_control_ids: ReadSignal<HashSet<String>>,
    pending_light_ids: ReadSignal<HashSet<String>>,
    active_scene_by_group: ReadSignal<HashMap<String, String>>,
    active_connection: ReadSignal<Option<BridgeConnection>>,
    is_refreshing: ReadSignal<bool>,
    on_open_settings: Callback<()>,
    on_toggle_all_lights: Callback<()>,
    on_toggle_room: Callback<String>,
    on_set_room_brightness: Callback<(String, u8)>,
    on_toggle_light: Callback<String>,
    on_set_light_brightness: Callback<(String, u8)>,
    on_activate_scene: Callback<ActivateSceneRequest>,
    on_delete_scene: Callback<DeleteSceneRequest>,
    on_create_curated_scenes: Callback<String>,
    on_create_custom_scene: Callback<SceneComposerRequest>,
    on_reorder_rooms: Callback<Vec<String>>,
) -> impl IntoView {
    let (dragged_room_id, set_dragged_room_id) = signal(None::<String>);
    let (drop_target_room_id, set_drop_target_room_id) = signal(None::<String>);
    let home_light_count = Signal::derive(move || lights.get().len());
    let home_active_count = Signal::derive(move || {
        lights
            .get()
            .iter()
            .filter(|light| light.is_on.unwrap_or(false))
            .count()
    });
    let is_updating_home =
        Signal::derive(move || pending_room_control_ids.get().contains("__all__"));

    view! {
        <section
            class="light-panel"
            on:pointerup=move |_| {
                set_dragged_room_id.set(None);
                set_drop_target_room_id.set(None);
            }
            on:pointercancel=move |_| {
                set_dragged_room_id.set(None);
                set_drop_target_room_id.set(None);
            }
        >
            <div class="panel-header compact-panel-header light-panel-header">
                <div>
                    <p class="panel-kicker">"Rooms"</p>
                    <h2>"Main residence"</h2>
                </div>
                <div class="light-panel-actions">
                    <button
                        class=move || {
                            if home_active_count.get() > 0 {
                                "home-master-switch is-on"
                            } else {
                                "home-master-switch"
                            }
                        }
                        disabled=move || is_refreshing.get() || is_updating_home.get() || home_light_count.get() == 0
                        on:click=move |_| on_toggle_all_lights.run(())
                    >
                        <span class="home-master-switch-label">
                            {move || if home_active_count.get() > 0 { "All on" } else { "All off" }}
                        </span>
                        <span class="home-master-switch-track">
                            <span class="home-master-switch-thumb"></span>
                        </span>
                    </button>
                </div>
            </div>

            {move || {
                if active_connection.get().is_none() {
                    view! {
                        <div class="empty-state">
                            <h3>"No active bridge session"</h3>
                            <p>"This app now keeps bridge setup behind settings. Open settings to pair or reconnect once."</p>
                            <button class="secondary-button inline-settings-button" on:click=move |_| on_open_settings.run(())>
                                "Open settings"
                            </button>
                        </div>
                    }
                    .into_any()
                } else if is_refreshing.get() && lights.get().is_empty() {
                    view! {
                        <div class="empty-state">
                            <h3>"Loading rooms"</h3>
                            <p>"Fetching the latest bridge snapshot."</p>
                        </div>
                    }
                    .into_any()
                } else if lights.get().is_empty() {
                    view! {
                        <div class="empty-state">
                            <h3>"No devices found"</h3>
                            <p>"The bridge is connected, but it did not return any lights in the current snapshot."</p>
                        </div>
                    }
                    .into_any()
                } else {
                    let room_sections = apply_room_order(
                        build_room_sections(&lights.get(), &groups.get(), &scenes.get()),
                        &room_order.get(),
                    );
                    let ordered_room_ids = room_sections
                        .iter()
                        .map(|room| room.id.clone())
                        .collect::<Vec<_>>();
                    let connection = active_connection.get();
                    view! {
                        <div class="room-grid">
                            {room_sections
                                .into_iter()
                                .map(|room| {
                                    let room_name = room.name.clone();
                                    let room_id = room.id.clone();
                                    let can_craft_scenes = room.can_craft_scenes;
                                    let room_scene_count = room.scenes.len();
                                    let room_light_count = room.lights.len();
                                    let room_active_count = room.active_light_count;
                                    let room_is_on = room_active_count > 0;
                                    let room_average_brightness = room.average_brightness;
                                    let room_brightness_label = brightness_label(room_average_brightness);
                                    let room_slider_value = room_average_brightness.max(1);
                                    let room_card_class = if room_active_count > 0 {
                                        "room-card is-active"
                                    } else {
                                        "room-card"
                                    };
                                    let room_style = room_accent_style(&room.lights)
                                        .unwrap_or_default();
                                    let room_drag_class = if drop_target_room_id.get().as_deref()
                                        == Some(room_id.as_str())
                                    {
                                        format!("{room_card_class} is-drop-target")
                                    } else {
                                        room_card_class.to_string()
                                    };
                                    let room_order_snapshot = ordered_room_ids.clone();
                                    let pointerenter_room_id = room_id.clone();
                                    let pointermove_room_id = room_id.clone();
                                    let dragleave_room_id = room_id.clone();
                                    let drop_room_id = room_id.clone();
                                    let dragstart_room_id = room_id.clone();
                                    let dragstart_drop_room_id = room_id.clone();
                                    let craft_room_id = room_id.clone();
                                    let custom_scene_room_id = room_id.clone();
                                    let toggle_room_id = room_id.clone();
                                    let slider_room_brightness_id = room_id.clone();
                                    let is_creating_scenes = pending_room_ids.get().contains(&room_id);
                                    let is_updating_room = pending_room_control_ids.get().contains(&room_id);
                                    let (show_composer, set_show_composer) = signal(false);
                                    let scene_strip = if room.scenes.is_empty() {
                                        view! {
                                            <div class="room-scene-empty">"No saved presets for this room"</div>
                                        }.into_any()
                                    } else {
                                        room
                                            .scenes
                                            .into_iter()
                                            .map(|scene| {
                                                let scene_name = scene.name.clone();
                                                let is_pending = pending_scene_id.get().as_deref() == Some(scene.id.as_str());
                                                let request = connection.clone().map(|connection| ActivateSceneRequest {
                                                    bridge_ip: connection.bridge_ip,
                                                    username: connection.username,
                                                    scene_id: scene.id.clone(),
                                                    group_id: scene.group_id.clone(),
                                                });
                                                let preview_style = scene_preview_style(&scene);
                                                let scene_type = scene
                                                    .scene_type
                                                    .clone()
                                                    .unwrap_or_else(|| "Scene".to_string());
                                                let delete_request = connection.clone().map(|connection| DeleteSceneRequest {
                                                    bridge_ip: connection.bridge_ip,
                                                    username: connection.username,
                                                    scene_id: scene.id.clone(),
                                                });
                                                let is_active_scene = scene
                                                    .group_id
                                                    .as_deref()
                                                    .and_then(|group_id| {
                                                        active_scene_by_group
                                                            .get()
                                                            .get(group_id)
                                                            .cloned()
                                                    })
                                                    .as_deref()
                                                    == Some(scene.id.as_str());
                                                let scene_class = if is_active_scene {
                                                    "scene-thumb is-active"
                                                } else {
                                                    "scene-thumb"
                                                };

                                                view! {
                                                    <div class=scene_class style=preview_style>
                                                        <button
                                                            class="scene-thumb-main"
                                                            disabled=is_pending
                                                            on:click=move |_| {
                                                                if let Some(request) = request.clone() {
                                                                    on_activate_scene.run(request);
                                                                }
                                                            }
                                                        >
                                                            <span class="scene-thumb-art-shell">
                                                                <span class="scene-thumb-art"></span>
                                                            </span>
                                                            <span class="scene-thumb-copy">
                                                                <strong>{scene_name.clone()}</strong>
                                                                <small>{scene_type}</small>
                                                            </span>
                                                        </button>
                                                        <button
                                                            class="scene-thumb-delete"
                                                            title="Delete scene"
                                                            aria-label=format!("Delete {scene_name}")
                                                            disabled=is_pending
                                                            on:click=move |ev| {
                                                                ev.stop_propagation();
                                                                if let Some(request) = delete_request.clone() {
                                                                    on_delete_scene.run(request);
                                                                }
                                                            }
                                                        >
                                                            "×"
                                                        </button>
                                                    </div>
                                                }
                                            })
                                            .collect_view()
                                            .into_any()
                                    };

                                    view! {
                                        <details class=room_drag_class style=room_style>
                                            <summary
                                                class="room-card-summary"
                                                on:pointerenter=move |_| {
                                                    if dragged_room_id.get().is_some() {
                                                        set_drop_target_room_id
                                                            .set(Some(pointerenter_room_id.clone()));
                                                    }
                                                }
                                                on:pointermove=move |_| {
                                                    if dragged_room_id.get().is_some() {
                                                        set_drop_target_room_id
                                                            .set(Some(pointermove_room_id.clone()));
                                                    }
                                                }
                                                on:pointerleave=move |_| {
                                                    if drop_target_room_id.get().as_deref()
                                                        == Some(dragleave_room_id.as_str())
                                                    {
                                                        set_drop_target_room_id.set(None);
                                                    }
                                                }
                                                on:pointerup=move |ev| {
                                                    let Some(source_room_id) = dragged_room_id.get() else {
                                                        return;
                                                    };

                                                    ev.prevent_default();
                                                    ev.stop_propagation();
                                                    set_drop_target_room_id.set(None);
                                                    set_dragged_room_id.set(None);

                                                    if source_room_id == drop_room_id {
                                                        return;
                                                    }

                                                    let reordered = reorder_room_ids(
                                                        &room_order_snapshot,
                                                        &source_room_id,
                                                        &drop_room_id,
                                                    );

                                                    if reordered != room_order_snapshot {
                                                        on_reorder_rooms.run(reordered);
                                                    }
                                                }
                                            >
                                                <div class="room-card-header">
                                                    <div class="room-summary-main">
                                                        <span
                                                            class="room-drag-handle"
                                                            title="Drag to reorder rooms"
                                                            on:pointerdown=move |ev| {
                                                                ev.prevent_default();
                                                                ev.stop_propagation();
                                                                set_dragged_room_id
                                                                    .set(Some(dragstart_room_id.clone()));
                                                                set_drop_target_room_id
                                                                    .set(Some(dragstart_drop_room_id.clone()));
                                                            }
                                                            on:pointerup=move |ev| {
                                                                ev.prevent_default();
                                                                ev.stop_propagation();
                                                            }
                                                            on:click=move |ev| {
                                                                ev.prevent_default();
                                                                ev.stop_propagation();
                                                            }
                                                        >
                                                            "⋮⋮"
                                                        </span>
                                                        <span class="room-summary-dot"></span>
                                                        <div class="room-summary-copy">
                                                            <h3>{room_name}</h3>
                                                            <p>
                                                                {format!("{room_scene_count} scenes · {room_light_count} device{}", if room_light_count == 1 { "" } else { "s" })}
                                                            </p>
                                                        </div>
                                                    </div>
                                                    <div class="room-card-tools">
                                                        <div class="room-meta">
                                                            <span>{format!("{room_active_count} on")}</span>
                                                            <span>{room_brightness_label.clone()}</span>
                                                        </div>
                                                        <button
                                                            class=move || {
                                                                if room_is_on {
                                                                    "room-summary-switch is-on"
                                                                } else {
                                                                    "room-summary-switch"
                                                                }
                                                            }
                                                            disabled=is_updating_room
                                                            on:click=move |ev| {
                                                                ev.prevent_default();
                                                                ev.stop_propagation();
                                                                on_toggle_room.run(toggle_room_id.clone());
                                                            }
                                                        >
                                                            <span class="room-summary-switch-track">
                                                                <span class="room-summary-switch-thumb"></span>
                                                            </span>
                                                        </button>
                                                    </div>
                                                </div>
                                                <div
                                                    class="room-level-slider-shell summary-level-bar"
                                                    on:mousedown=move |ev| {
                                                        ev.stop_propagation();
                                                    }
                                                    on:click=move |ev| {
                                                        ev.stop_propagation();
                                                    }
                                                >
                                                    <span class="room-level-bar">
                                                        <span
                                                            class="room-level-fill"
                                                            style=format!("width: {};", room_brightness_label)
                                                        ></span>
                                                    </span>
                                                    <input
                                                        class="room-level-slider room-level-slider-overlay"
                                                        type="range"
                                                        min="1"
                                                        max="254"
                                                        value=room_slider_value.to_string()
                                                        disabled=is_updating_room
                                                        on:change=move |ev| {
                                                            ev.stop_propagation();
                                                            if let Ok(value) = event_target_value(&ev).parse::<u8>() {
                                                                on_set_room_brightness.run((slider_room_brightness_id.clone(), value));
                                                            }
                                                        }
                                                    />
                                                </div>
                                            </summary>

                                            <div class="room-card-body">
                                                <div class="room-strip-header">
                                                    <span class="room-strip-label">"Scenes"</span>
                                                    <div class="room-strip-actions">
                                                        <button
                                                            class="secondary-button room-action-button"
                                                            disabled=is_creating_scenes
                                                            on:click=move |_| set_show_composer.update(|show| *show = !*show)
                                                        >
                                                            {move || if show_composer.get() { "Hide composer" } else { "New scene" }}
                                                        </button>
                                                        {if can_craft_scenes {
                                                            view! {
                                                                <button
                                                                    class="secondary-button room-action-button"
                                                                    disabled=is_creating_scenes
                                                                    on:click=move |_| on_create_curated_scenes.run(craft_room_id.clone())
                                                                >
                                                                    {if is_creating_scenes {
                                                                        "Crafting..."
                                                                    } else {
                                                                        "Craft curated scenes"
                                                                    }}
                                                                </button>
                                                            }
                                                                .into_any()
                                                        } else {
                                                            view! { <span class="room-strip-muted">"Bridge room required"</span> }.into_any()
                                                        }}
                                                    </div>
                                                </div>
                                                {move || {
                                                    if show_composer.get() {
                                                        view! {
                                                            <SceneComposer
                                                                room_id=custom_scene_room_id.clone()
                                                                pending=is_creating_scenes
                                                                on_submit=Callback::new(move |request: SceneComposerRequest| {
                                                                    on_create_custom_scene.run(request);
                                                                    set_show_composer.set(false);
                                                                })
                                                            />
                                                        }.into_any()
                                                    } else {
                                                        ().into_any()
                                                    }
                                                }}
                                                <div class="room-scene-strip">
                                                    {scene_strip}
                                                </div>

                                                <span class="room-strip-label">"Lights"</span>
                                                <DeviceGrid
                                                    lights=room.lights
                                                    groups=groups.get()
                                                    pending_light_ids=pending_light_ids
                                                    on_toggle_light=on_toggle_light
                                                    on_set_light_brightness=on_set_light_brightness
                                                />
                                            </div>
                                        </details>
                                    }
                                })
                                .collect_view()}
                        </div>
                    }
                    .into_any()
                }
            }}
        </section>
    }
}

#[derive(Clone, PartialEq)]
struct RoomSection {
    id: String,
    can_craft_scenes: bool,
    name: String,
    active_light_count: usize,
    average_brightness: u8,
    lights: Vec<Light>,
    scenes: Vec<Scene>,
}

fn build_room_sections(lights: &[Light], groups: &[Group], scenes: &[Scene]) -> Vec<RoomSection> {
    let mut rooms: Vec<Group> = groups
        .iter()
        .filter(|group| matches!(group.kind, GroupKind::Room))
        .cloned()
        .collect();
    rooms.sort_by(|left, right| left.name.cmp(&right.name));

    let mut sections: Vec<RoomSection> = rooms
        .into_iter()
        .map(|room| {
            let mut room_lights: Vec<Light> = lights
                .iter()
                .filter(|light| room.light_ids.iter().any(|id| id == &light.id))
                .cloned()
                .collect();
            room_lights.sort_by(|left, right| left.name.cmp(&right.name));

            let mut room_scenes: Vec<Scene> = scenes
                .iter()
                .filter(|scene| scene.group_id.as_deref() == Some(room.id.as_str()))
                .cloned()
                .collect();
            room_scenes.sort_by(|left, right| left.name.cmp(&right.name));

            let active_light_count = room_lights
                .iter()
                .filter(|light| light.is_on.unwrap_or(false))
                .count();
            let average_brightness = if room_lights.is_empty() {
                0
            } else {
                let total_brightness: u32 = room_lights
                    .iter()
                    .map(|light| u32::from(light.brightness.unwrap_or(0)))
                    .sum();
                (total_brightness / room_lights.len() as u32) as u8
            };

            RoomSection {
                id: room.id,
                can_craft_scenes: true,
                name: room.name,
                active_light_count,
                average_brightness,
                lights: room_lights,
                scenes: room_scenes,
            }
        })
        .filter(|section| !section.lights.is_empty() || !section.scenes.is_empty())
        .collect();

    let assigned_ids: HashSet<String> = sections
        .iter()
        .flat_map(|section| section.lights.iter().map(|light| light.id.clone()))
        .collect();

    let mut orphan_lights: Vec<Light> = lights
        .iter()
        .filter(|light| !assigned_ids.contains(&light.id))
        .cloned()
        .collect();
    orphan_lights.sort_by(|left, right| left.name.cmp(&right.name));

    if !orphan_lights.is_empty() {
        sections.push(RoomSection {
            id: "unassigned".to_string(),
            can_craft_scenes: false,
            name: "Unassigned".to_string(),
            active_light_count: 0,
            average_brightness: 0,
            lights: orphan_lights,
            scenes: Vec::new(),
        });
    }

    sections
}

fn apply_room_order(mut sections: Vec<RoomSection>, saved_order: &[String]) -> Vec<RoomSection> {
    let saved_positions = saved_order
        .iter()
        .enumerate()
        .map(|(index, room_id)| (room_id.as_str(), index))
        .collect::<std::collections::HashMap<_, _>>();

    sections.sort_by(|left, right| {
        let left_index = saved_positions.get(left.id.as_str()).copied();
        let right_index = saved_positions.get(right.id.as_str()).copied();

        match (left_index, right_index) {
            (Some(left_index), Some(right_index)) => left_index
                .cmp(&right_index)
                .then(left.name.cmp(&right.name)),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => left.name.cmp(&right.name),
        }
    });

    sections
}

fn reorder_room_ids(current_order: &[String], source_id: &str, target_id: &str) -> Vec<String> {
    let mut reordered = current_order.to_vec();
    let Some(source_index) = reordered.iter().position(|room_id| room_id == source_id) else {
        return reordered;
    };
    let Some(target_index) = reordered.iter().position(|room_id| room_id == target_id) else {
        return reordered;
    };

    let source_room_id = reordered.remove(source_index);
    let insert_index = target_index.min(reordered.len());
    reordered.insert(insert_index, source_room_id);
    reordered
}

fn brightness_label(value: u8) -> String {
    let percentage = (u16::from(value) * 100) / 254;
    format!("{percentage}%")
}

fn scene_preview_style(scene: &Scene) -> String {
    if let (Some(soft), Some(main), Some(deep)) = (
        scene.preview_color_soft.as_deref(),
        scene.preview_color_main.as_deref(),
        scene.preview_color_deep.as_deref(),
    ) {
        return format!(
            "--scene-color-soft: {soft}; --scene-color-main: {main}; --scene-color-deep: {deep};"
        );
    }

    let (mut hue, mut saturation, mut value) = fallback_scene_palette_seed(&scene.name);
    let lower = scene.name.to_ascii_lowercase();
    let tone = classify_scene_tone(&lower);
    let variation = hashed_scene_variation(&scene.name);

    hue = wrap_hue(hue + tone.hue_shift + variation.hue_shift);
    saturation =
        (saturation + tone.saturation_shift + variation.saturation_shift).clamp(0.44, 0.88);
    value = (value + tone.value_shift + variation.value_shift).clamp(0.82, 0.99);

    let soft = hsv_to_rgb_float(
        hue,
        (saturation * 0.58).clamp(0.26, 0.62),
        (value + 0.06).clamp(0.9, 1.0),
    );
    let main = hsv_to_rgb_float(hue, saturation, value);
    let deep = hsv_to_rgb_float(
        wrap_hue(hue - 12.0),
        (saturation * 0.96).clamp(0.42, 0.9),
        (value - 0.2).clamp(0.52, 0.84),
    );

    format!(
        "--scene-color-soft: rgb({} {} {}); --scene-color-main: rgb({} {} {}); --scene-color-deep: rgb({} {} {});",
        soft.0, soft.1, soft.2, main.0, main.1, main.2, deep.0, deep.1, deep.2
    )
}

fn room_accent_style(lights: &[Light]) -> Option<String> {
    average_accent_color(lights).map(|color| format!("--bridge-accent: {color};"))
}

fn average_accent_color(lights: &[Light]) -> Option<String> {
    let preferred_lights: Vec<&Light> = lights
        .iter()
        .filter(|light| light.is_on.unwrap_or(false))
        .collect();
    let source_lights = if preferred_lights.is_empty() {
        lights.iter().collect::<Vec<_>>()
    } else {
        preferred_lights
    };

    let colors: Vec<(u8, u8, u8)> = source_lights
        .into_iter()
        .filter_map(light_accent_rgb)
        .collect();

    if colors.is_empty() {
        return None;
    }

    let count = colors.len() as u32;
    let (red_sum, green_sum, blue_sum) = colors.into_iter().fold(
        (0_u32, 0_u32, 0_u32),
        |(red_sum, green_sum, blue_sum), (red, green, blue)| {
            (
                red_sum + u32::from(red),
                green_sum + u32::from(green),
                blue_sum + u32::from(blue),
            )
        },
    );

    Some(format!(
        "rgb({} {} {})",
        red_sum / count,
        green_sum / count,
        blue_sum / count
    ))
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
    let x_xyz = (luminance / y) * x;
    let z_xyz = (luminance / y) * z;

    let mut red = x_xyz * 1.656492 - luminance * 0.354851 - z_xyz * 0.255038;
    let mut green = -x_xyz * 0.707196 + luminance * 1.655397 + z_xyz * 0.036152;
    let mut blue = x_xyz * 0.051713 - luminance * 0.121364 + z_xyz * 1.01153;

    red = red.max(0.0);
    green = green.max(0.0);
    blue = blue.max(0.0);

    let max_channel = red.max(green).max(blue);
    if max_channel > 1.0 {
        red /= max_channel;
        green /= max_channel;
        blue /= max_channel;
    }

    Some((
        gamma_correct(red),
        gamma_correct(green),
        gamma_correct(blue),
    ))
}

fn gamma_correct(value: f32) -> u8 {
    let corrected = if value <= 0.0031308 {
        12.92 * value
    } else {
        1.055 * value.powf(1.0 / 2.4) - 0.055
    };

    (corrected.clamp(0.0, 1.0) * 255.0).round() as u8
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

fn fallback_scene_palette_seed(name: &str) -> (f32, f32, f32) {
    let hash = name.bytes().fold(0_u32, |hash, byte| {
        hash.wrapping_mul(37).wrapping_add(u32::from(byte))
    });

    let hue = (hash % 360) as f32;
    let saturation = 0.66 + (((hash >> 8) % 17) as f32 / 100.0);
    let value = 0.9 + (((hash >> 16) % 7) as f32 / 100.0);

    (hue, saturation.clamp(0.66, 0.82), value.clamp(0.9, 0.97))
}

#[derive(Clone, Copy)]
struct SceneTone {
    hue_shift: f32,
    saturation_shift: f32,
    value_shift: f32,
}

fn classify_scene_tone(name: &str) -> SceneTone {
    if name.contains("sunset") || name.contains("gold") || name.contains("amber") {
        SceneTone {
            hue_shift: 0.0,
            saturation_shift: 0.06,
            value_shift: 0.05,
        }
    } else if name.contains("ocean") || name.contains("blue") || name.contains("arctic") {
        SceneTone {
            hue_shift: 200.0,
            saturation_shift: 0.02,
            value_shift: -0.04,
        }
    } else if name.contains("forest") || name.contains("spring") || name.contains("green") {
        SceneTone {
            hue_shift: 120.0,
            saturation_shift: 0.02,
            value_shift: 0.0,
        }
    } else if name.contains("night") || name.contains("focus") || name.contains("dim") {
        SceneTone {
            hue_shift: 280.0,
            saturation_shift: -0.02,
            value_shift: -0.1,
        }
    } else if name.contains("read") || name.contains("relax") || name.contains("soft") {
        SceneTone {
            hue_shift: 36.0,
            saturation_shift: -0.1,
            value_shift: 0.04,
        }
    } else {
        SceneTone {
            hue_shift: 0.0,
            saturation_shift: 0.0,
            value_shift: 0.02,
        }
    }
}

fn hashed_scene_variation(name: &str) -> SceneTone {
    let hash = name.bytes().fold(0_u32, |hash, byte| {
        hash.wrapping_mul(33).wrapping_add(u32::from(byte))
    });
    let hue_shift = ((hash % 31) as f32 - 15.0) * 2.8;
    let saturation_shift = (((hash >> 8) % 11) as f32 - 5.0) * 0.016;
    let value_shift = (((hash >> 16) % 9) as f32 - 4.0) * 0.012;

    SceneTone {
        hue_shift,
        saturation_shift,
        value_shift,
    }
}

#[cfg(test)]
mod tests {
    use super::{apply_room_order, reorder_room_ids, RoomSection};

    #[test]
    fn reorder_room_ids_moves_source_into_target_position_when_moving_up() {
        let current = vec![
            "desk".to_string(),
            "hallway".to_string(),
            "kitchen".to_string(),
        ];

        let reordered = reorder_room_ids(&current, "kitchen", "desk");
        assert_eq!(reordered, vec!["kitchen", "desk", "hallway"]);
    }

    #[test]
    fn reorder_room_ids_moves_source_into_target_position_when_moving_down() {
        let current = vec![
            "desk".to_string(),
            "hallway".to_string(),
            "kitchen".to_string(),
        ];

        let reordered = reorder_room_ids(&current, "desk", "hallway");
        assert_eq!(reordered, vec!["hallway", "desk", "kitchen"]);
    }

    #[test]
    fn apply_room_order_prefers_saved_sequence() {
        let sections = vec![
            RoomSection {
                id: "hallway".to_string(),
                can_craft_scenes: true,
                name: "Hallway".to_string(),
                active_light_count: 0,
                average_brightness: 0,
                lights: Vec::new(),
                scenes: Vec::new(),
            },
            RoomSection {
                id: "desk".to_string(),
                can_craft_scenes: true,
                name: "Desk".to_string(),
                active_light_count: 0,
                average_brightness: 0,
                lights: Vec::new(),
                scenes: Vec::new(),
            },
        ];

        let ordered = apply_room_order(sections, &["desk".to_string()]);
        assert_eq!(ordered[0].id, "desk");
        assert_eq!(ordered[1].id, "hallway");
    }
}
