use crate::hue::{ActivateSceneRequest, BridgeConnection, Group, GroupKind, Light, Scene};
use leptos::prelude::*;
use std::collections::HashSet;
use wasm_bindgen::{JsCast, JsValue};

#[component]
pub fn LightGrid(
    lights: ReadSignal<Vec<Light>>,
    groups: ReadSignal<Vec<Group>>,
    scenes: ReadSignal<Vec<Scene>>,
    room_order: ReadSignal<Vec<String>>,
    pending_light_ids: ReadSignal<HashSet<String>>,
    pending_scene_id: ReadSignal<Option<String>>,
    pending_room_ids: ReadSignal<HashSet<String>>,
    pending_room_control_ids: ReadSignal<HashSet<String>>,
    active_connection: ReadSignal<Option<BridgeConnection>>,
    is_refreshing: ReadSignal<bool>,
    on_open_settings: Callback<()>,
    on_toggle_light: Callback<String>,
    on_set_brightness: Callback<(String, u8)>,
    on_toggle_room: Callback<String>,
    on_set_room_brightness: Callback<(String, u8)>,
    on_activate_scene: Callback<ActivateSceneRequest>,
    on_create_curated_scenes: Callback<String>,
    on_reorder_rooms: Callback<Vec<String>>,
) -> impl IntoView {
    let (dragged_room_id, set_dragged_room_id) = signal(None::<String>);
    let (drop_target_room_id, set_drop_target_room_id) = signal(None::<String>);

    view! {
        <section class="panel surface-panel light-panel">
            <div class="panel-header compact-panel-header">
                <div>
                    <p class="panel-kicker">"Rooms"</p>
                    <h2>"Main residence"</h2>
                </div>
                <div class="panel-badge">
                    {move || {
                        active_connection
                            .get()
                            .map(|connection| connection.bridge_ip)
                            .unwrap_or_else(|| "Not connected".to_string())
                    }}
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
                                    let room_average_brightness = room.average_brightness;
                                    let room_brightness_label = brightness_label(room_average_brightness);
                                    let room_slider_value = room_average_brightness.max(1);
                                    let room_card_class = if room_active_count > 0 {
                                        "room-card is-active"
                                    } else {
                                        "room-card"
                                    };
                                    let room_drag_class = if drop_target_room_id.get().as_deref()
                                        == Some(room_id.as_str())
                                    {
                                        format!("{room_card_class} is-drop-target")
                                    } else {
                                        room_card_class.to_string()
                                    };
                                    let room_order_snapshot = ordered_room_ids.clone();
                                    let dragover_room_id = room_id.clone();
                                    let dragleave_room_id = room_id.clone();
                                    let drop_room_id = room_id.clone();
                                    let dragstart_room_id = room_id.clone();
                                    let dragstart_drop_room_id = room_id.clone();
                                    let craft_room_id = room_id.clone();
                                    let toggle_room_id = room_id.clone();
                                    let set_room_brightness_id = room_id.clone();
                                    let is_creating_scenes = pending_room_ids.get().contains(&room_id);
                                    let is_updating_room = pending_room_control_ids.get().contains(&room_id);
                                    let scene_strip = if room.scenes.is_empty() {
                                        view! {
                                            <div class="room-scene-empty">"No saved presets for this room"</div>
                                        }.into_any()
                                    } else {
                                        room
                                            .scenes
                                            .into_iter()
                                            .map(|scene| {
                                                let is_pending = pending_scene_id.get().as_deref() == Some(scene.id.as_str());
                                                let request = connection.clone().map(|connection| ActivateSceneRequest {
                                                    bridge_ip: connection.bridge_ip,
                                                    username: connection.username,
                                                    scene_id: scene.id.clone(),
                                                    group_id: scene.group_id.clone(),
                                                });
                                                let preview_class = scene_preview_class(&scene.name);
                                                let scene_type = scene
                                                    .scene_type
                                                    .clone()
                                                    .unwrap_or_else(|| "Scene".to_string());

                                                view! {
                                                    <button
                                                        class="scene-thumb"
                                                        disabled=is_pending
                                                        on:click=move |_| {
                                                            if let Some(request) = request.clone() {
                                                                on_activate_scene.run(request);
                                                            }
                                                        }
                                                    >
                                                        <span class=format!("scene-thumb-art {preview_class}")></span>
                                                        <span class="scene-thumb-copy">
                                                            <strong>{scene.name}</strong>
                                                            <small>{scene_type}</small>
                                                        </span>
                                                    </button>
                                                }
                                            })
                                            .collect_view()
                                            .into_any()
                                    };

                                    view! {
                                        <details class=room_drag_class>
                                            <summary
                                                class="room-card-summary"
                                                on:dragover=move |ev| {
                                                    ev.prevent_default();
                                                    set_drop_effect(&ev, "move");
                                                    set_drop_target_room_id.set(Some(dragover_room_id.clone()));
                                                }
                                                on:dragleave=move |_| {
                                                    if drop_target_room_id.get().as_deref()
                                                        == Some(dragleave_room_id.as_str())
                                                    {
                                                        set_drop_target_room_id.set(None);
                                                    }
                                                }
                                                on:drop=move |ev| {
                                                    ev.prevent_default();
                                                    let source_room_id = read_drag_data(&ev)
                                                        .filter(|room_id| !room_id.is_empty())
                                                        .or_else(|| dragged_room_id.get());

                                                    let Some(source_room_id) = source_room_id else {
                                                        return;
                                                    };

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
                                                        <button
                                                            class="room-drag-handle"
                                                            draggable="true"
                                                            on:dragstart=move |ev| {
                                                                ev.stop_propagation();
                                                                write_drag_data(&ev, &dragstart_room_id);
                                                                set_effect_allowed(&ev, "move");
                                                                set_dragged_room_id
                                                                    .set(Some(dragstart_room_id.clone()));
                                                                set_drop_target_room_id
                                                                    .set(Some(dragstart_drop_room_id.clone()));
                                                            }
                                                            on:dragend=move |_| {
                                                                set_dragged_room_id.set(None);
                                                                set_drop_target_room_id.set(None);
                                                            }
                                                        >
                                                            "⋮⋮"
                                                        </button>
                                                        <button
                                                            class="room-summary-dot-button"
                                                            disabled=is_updating_room
                                                            on:click=move |ev| {
                                                                ev.prevent_default();
                                                                ev.stop_propagation();
                                                                on_toggle_room.run(toggle_room_id.clone());
                                                            }
                                                        >
                                                            <span class="room-summary-dot"></span>
                                                        </button>
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
                                                        <span class="room-collapse-hint">"Collapse"</span>
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
                                                                on_set_room_brightness.run((set_room_brightness_id.clone(), value));
                                                            }
                                                        }
                                                    />
                                                </div>
                                            </summary>

                                            <div class="room-card-body">
                                                <div class="room-strip-block">
                                                    <div class="room-strip-header">
                                                        <span class="room-strip-label">"Scenes"</span>
                                                        <div class="room-strip-actions">
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
                                                    <div class="room-scene-strip">
                                                        {scene_strip}
                                                    </div>
                                                </div>

                                                <div class="room-strip-block">
                                                    <span class="room-strip-label">"Lights"</span>
                                                    <div class="device-list">
                                                        {room
                                                            .lights
                                                            .into_iter()
                                                            .map(|light| {
                                                        let light_id = light.id.clone();
                                                        let toggle_light_id = light.id.clone();
                                                        let brightness_light_id = light.id.clone();
                                                        let slider_id = format!("brightness-{}", light.id);
                                                        let is_pending = pending_light_ids.get().contains(&light_id);
                                                        let placement = derive_placement(&light.id, &groups.get());
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
                                                        let brightness_value = light.brightness.unwrap_or(127);
                                                        let is_on = light.is_on.unwrap_or(false);

                                                        view! {
                                                            <article class="light-card compact-light-card">
                                                                <div class="light-card-top">
                                                                    <div>
                                                                        <p class="light-eyebrow">{light_type}</p>
                                                                        <h3>{light_name}</h3>
                                                                    </div>
                                                                    <div class="light-status" class:is-off=move || !is_on>
                                                                        <span class="status-dot"></span>
                                                                        <span>{if is_on { "On" } else { "Off" }}</span>
                                                                    </div>
                                                                </div>

                                                                <div class="light-meta-cluster">
                                                                    <span class="light-meta-chip">
                                                                        {if placement.zone_names.is_empty() {
                                                                            "No zones".to_string()
                                                                        } else {
                                                                            placement.zone_names.join(", ")
                                                                        }}
                                                                    </span>
                                                                    <span class="light-meta-chip">{format!("ID {}", light.id)}</span>
                                                                    <span class="light-meta-chip">{reachable_text}</span>
                                                                </div>

                                                                <div class="device-actions">
                                                                    <button
                                                                        class="toggle-button compact-toggle-button"
                                                                        class:is-active=move || is_on
                                                                        disabled=is_pending
                                                                        on:click=move |_| on_toggle_light.run(toggle_light_id.clone())
                                                                    >
                                                                        {move || {
                                                                            if is_pending {
                                                                                "Updating..."
                                                                            } else if is_on {
                                                                                "Turn off"
                                                                            } else {
                                                                                "Turn on"
                                                                            }
                                                                        }}
                                                                    </button>

                                                                    <label class="brightness-block compact-brightness-block" for=slider_id.clone()>
                                                                        <div class="brightness-header">
                                                                            <span>"Brightness"</span>
                                                                            <strong>{brightness_label(brightness_value)}</strong>
                                                                        </div>
                                                                        <input
                                                                            id=slider_id.clone()
                                                                            class="brightness-slider"
                                                                            type="range"
                                                                            min="1"
                                                                            max="254"
                                                                            value=brightness_value.to_string()
                                                                            disabled=is_pending
                                                                            on:change=move |ev| {
                                                                                if let Ok(value) = event_target_value(&ev).parse::<u8>() {
                                                                                    on_set_brightness.run((brightness_light_id.clone(), value));
                                                                                }
                                                                            }
                                                                        />
                                                                    </label>
                                                                </div>
                                                            </article>
                                                        }
                                                            })
                                                            .collect_view()}
                                                    </div>
                                                </div>
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

#[derive(Clone, PartialEq, Eq)]
struct RoomSection {
    id: String,
    can_craft_scenes: bool,
    name: String,
    active_light_count: usize,
    average_brightness: u8,
    lights: Vec<Light>,
    scenes: Vec<Scene>,
}

#[derive(Default)]
struct LightPlacement {
    zone_names: Vec<String>,
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
    let insert_index = if source_index < target_index {
        target_index.saturating_sub(1)
    } else {
        target_index
    };
    reordered.insert(insert_index, source_room_id);
    reordered
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

fn scene_preview_class(name: &str) -> &'static str {
    let lower = name.to_ascii_lowercase();
    if lower.contains("sunset") || lower.contains("gold") || lower.contains("amber") {
        "is-sunset"
    } else if lower.contains("ocean") || lower.contains("blue") || lower.contains("arctic") {
        "is-ocean"
    } else if lower.contains("forest") || lower.contains("spring") || lower.contains("green") {
        "is-forest"
    } else if lower.contains("night") || lower.contains("focus") || lower.contains("dim") {
        "is-night"
    } else {
        "is-default"
    }
}

fn write_drag_data<E>(event: &E, room_id: &str)
where
    E: AsRef<JsValue>,
{
    let Some(data_transfer) = get_data_transfer(event) else {
        return;
    };

    let Ok(set_data) = js_sys::Reflect::get(&data_transfer, &JsValue::from_str("setData"))
        .and_then(|value| value.dyn_into::<js_sys::Function>())
    else {
        return;
    };

    let _ = set_data.call2(
        &data_transfer,
        &JsValue::from_str("text/plain"),
        &JsValue::from_str(room_id),
    );
}

fn read_drag_data<E>(event: &E) -> Option<String>
where
    E: AsRef<JsValue>,
{
    let data_transfer = get_data_transfer(event)?;
    let get_data = js_sys::Reflect::get(&data_transfer, &JsValue::from_str("getData"))
        .ok()?
        .dyn_into::<js_sys::Function>()
        .ok()?;

    get_data
        .call1(&data_transfer, &JsValue::from_str("text/plain"))
        .ok()?
        .as_string()
}

fn set_effect_allowed<E>(event: &E, value: &str)
where
    E: AsRef<JsValue>,
{
    let Some(data_transfer) = get_data_transfer(event) else {
        return;
    };

    let _ = js_sys::Reflect::set(
        &data_transfer,
        &JsValue::from_str("effectAllowed"),
        &JsValue::from_str(value),
    );
}

fn set_drop_effect<E>(event: &E, value: &str)
where
    E: AsRef<JsValue>,
{
    let Some(data_transfer) = get_data_transfer(event) else {
        return;
    };

    let _ = js_sys::Reflect::set(
        &data_transfer,
        &JsValue::from_str("dropEffect"),
        &JsValue::from_str(value),
    );
}

fn get_data_transfer<E>(event: &E) -> Option<JsValue>
where
    E: AsRef<JsValue>,
{
    let value = js_sys::Reflect::get(event.as_ref(), &JsValue::from_str("dataTransfer")).ok()?;
    if value.is_null() || value.is_undefined() {
        None
    } else {
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::{apply_room_order, reorder_room_ids, RoomSection};

    #[test]
    fn reorder_room_ids_moves_source_before_target() {
        let current = vec![
            "desk".to_string(),
            "hallway".to_string(),
            "kitchen".to_string(),
        ];

        let reordered = reorder_room_ids(&current, "kitchen", "desk");
        assert_eq!(reordered, vec!["kitchen", "desk", "hallway"]);
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
