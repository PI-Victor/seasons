use crate::hue::{
    self, curated_room_scenes, preset_light_state, ActivateSceneRequest, BridgeConnection,
    CreateSceneRequest, CreateUserRequest, Group, GroupKind, Light, LightStateUpdate, Scene,
    SetLightStateRequest,
};
use crate::storage;
use crate::ui::{BridgePanel, LightGrid, NoticeTone, StatusBanner, UiNotice};
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AppPage {
    Home,
    Settings,
}

#[component]
pub fn App() -> impl IntoView {
    let (page, set_page) = signal(AppPage::Home);
    let (discovered_bridges, set_discovered_bridges) = signal(Vec::new());
    let (selected_bridge_ip, set_selected_bridge_ip) = signal(String::new());
    let (username, set_username) = signal(String::new());
    let (device_type, set_device_type) = signal("seasons#desktop".to_string());
    let (scenes, set_scenes) = signal(Vec::<Scene>::new());
    let (groups, set_groups) = signal(Vec::<Group>::new());
    let (lights, set_lights) = signal(Vec::<Light>::new());
    let (active_connection, set_active_connection) = signal(None::<BridgeConnection>);
    let (notice, set_notice) = signal(Some(UiNotice::info(
        "Local-first Hue control",
        "Bridge setup is stored locally after a successful connection. Use settings only when you need to change it.",
    )));
    let (is_discovering, set_is_discovering) = signal(false);
    let (is_registering, set_is_registering) = signal(false);
    let (is_connecting, set_is_connecting) = signal(false);
    let (is_refreshing, set_is_refreshing) = signal(false);
    let (did_restore_session, set_did_restore_session) = signal(false);
    let (pending_light_ids, set_pending_light_ids) = signal(HashSet::<String>::new());
    let (pending_scene_id, set_pending_scene_id) = signal(None::<String>);
    let (pending_room_ids, set_pending_room_ids) = signal(HashSet::<String>::new());
    let (pending_room_control_ids, set_pending_room_control_ids) = signal(HashSet::<String>::new());
    let (room_order, set_room_order) = signal(Vec::<String>::new());

    let refresh_bridge_state = Callback::new({
        move |connection: BridgeConnection| {
            set_is_refreshing.set(true);
            spawn_local({
                let connection = connection.clone();
                async move {
                    match fetch_bridge_snapshot(connection.clone()).await {
                        Ok((fetched_lights, fetched_scenes, fetched_groups)) => {
                            let light_count = fetched_lights.len();
                            let scene_count = fetched_scenes.len();
                            let saved_room_order = storage::load_room_order(&connection)
                                .await
                                .unwrap_or_else(|_| Vec::new());
                            set_lights.set(fetched_lights);
                            set_scenes.set(fetched_scenes);
                            set_groups.set(fetched_groups);
                            set_room_order.set(saved_room_order);
                            set_active_connection.set(Some(connection.clone()));
                            if let Err(error) = storage::save_bridge_connection(&connection).await {
                                set_notice.set(Some(UiNotice::warning(
                                    "Session active, but not persisted",
                                    error,
                                )));
                            } else {
                                set_notice.set(Some(UiNotice::success(
                                    "Bridge connected",
                                    format!(
                                        "Loaded {light_count} devices and {scene_count} scenes from the bridge."
                                    ),
                                )));
                            }
                        }
                        Err(error) => {
                            set_active_connection.set(None);
                            set_lights.set(Vec::new());
                            set_scenes.set(Vec::new());
                            set_groups.set(Vec::new());
                            set_room_order.set(Vec::new());
                            set_notice
                                .set(Some(UiNotice::error("Could not load bridge data", error)));
                        }
                    }

                    set_is_refreshing.set(false);
                }
            });
        }
    });

    Effect::new(move |_| {
        if did_restore_session.get() {
            return;
        }

        set_did_restore_session.set(true);
        spawn_local(async move {
            match storage::load_bridge_connection().await {
                Ok(Some(connection)) => {
                    set_selected_bridge_ip.set(connection.bridge_ip.clone());
                    set_username.set(connection.username.clone());
                    set_active_connection.set(Some(connection.clone()));
                    set_notice.set(Some(UiNotice::info(
                        "Restoring last bridge",
                        "Loading your previously connected Hue bridge.",
                    )));
                    refresh_bridge_state.run(connection);
                }
                Ok(None) => {}
                Err(error) => {
                    set_notice.set(Some(UiNotice::warning(
                        "Could not restore saved bridge",
                        error,
                    )));
                }
            }
        });
    });

    let discover_bridges = Callback::new({
        move |()| {
            set_is_discovering.set(true);
            spawn_local(async move {
                match hue::discover_hue_bridges().await {
                    Ok(bridges) => {
                        let bridge_count = bridges.len();
                        let first_bridge = bridges
                            .first()
                            .map(|bridge| bridge.internal_ip_address.clone());
                        set_discovered_bridges.set(bridges);

                        if selected_bridge_ip.get_untracked().trim().is_empty() {
                            if let Some(first_bridge) = first_bridge {
                                set_selected_bridge_ip.set(first_bridge);
                            }
                        }

                        if bridge_count == 0 {
                            set_notice.set(Some(UiNotice::warning(
                                "No bridge discovered",
                                "No Hue bridge responded on the local network. Verify power, network access, and that the bridge is on the same LAN.",
                            )));
                        } else {
                            set_notice.set(Some(UiNotice::success(
                                "Bridge discovery complete",
                                format!("Found {bridge_count} bridge{}", pluralize(bridge_count)),
                            )));
                        }
                    }
                    Err(error) => {
                        set_notice.set(Some(UiNotice::error("Bridge discovery failed", error)));
                    }
                }

                set_is_discovering.set(false);
            });
        }
    });

    let connect_bridge = Callback::new({
        move |()| {
            let bridge_ip = selected_bridge_ip.get_untracked().trim().to_string();
            let username_value = username.get_untracked().trim().to_string();

            if bridge_ip.is_empty() {
                set_notice.set(Some(UiNotice::warning(
                    "Bridge IP required",
                    "Choose a discovered bridge or enter the bridge IP before loading rooms.",
                )));
                return;
            }

            if username_value.is_empty() {
                set_notice.set(Some(UiNotice::warning(
                    "Username required",
                    "Enter an existing Hue app username or pair a new one with the bridge button.",
                )));
                return;
            }

            set_is_connecting.set(true);
            spawn_local(async move {
                let connection = BridgeConnection {
                    bridge_ip,
                    username: username_value,
                };

                set_notice.set(Some(UiNotice::info(
                    "Loading bridge state",
                    "Connecting to the bridge and fetching devices, scenes, and room data.",
                )));
                refresh_bridge_state.run(connection);
                set_is_connecting.set(false);
                set_page.set(AppPage::Home);
            });
        }
    });

    let pair_new_app = Callback::new({
        move |()| {
            let bridge_ip = selected_bridge_ip.get_untracked().trim().to_string();
            let device_type_value = device_type.get_untracked().trim().to_string();

            if bridge_ip.is_empty() {
                set_notice.set(Some(UiNotice::warning(
                    "Bridge IP required",
                    "Choose a discovered bridge or enter the bridge IP before pairing.",
                )));
                return;
            }

            if device_type_value.is_empty() {
                set_notice.set(Some(UiNotice::warning(
                    "Device label required",
                    "Use a short device label so the bridge can identify this app.",
                )));
                return;
            }

            set_is_registering.set(true);
            set_notice.set(Some(UiNotice::info(
                "Ready to pair",
                "Press the button on the Hue bridge, then confirm pairing here within about 30 seconds.",
            )));

            spawn_local(async move {
                let request = CreateUserRequest {
                    bridge_ip: bridge_ip.clone(),
                    device_type: device_type_value,
                };

                match hue::create_hue_user(request).await {
                    Ok(registered) => {
                        let connection = BridgeConnection {
                            bridge_ip,
                            username: registered.username.clone(),
                        };

                        set_username.set(registered.username);
                        set_notice.set(Some(UiNotice::success(
                            "Pairing complete",
                            "The bridge accepted this app. The connection is being stored locally now.",
                        )));
                        refresh_bridge_state.run(connection);
                        set_page.set(AppPage::Home);
                    }
                    Err(error) => {
                        set_notice.set(Some(UiNotice::error("Pairing failed", error)));
                    }
                }

                set_is_registering.set(false);
            });
        }
    });

    let forget_bridge = Callback::new({
        move |()| {
            let active_connection = active_connection.get_untracked();
            spawn_local(async move {
                if let Some(connection) = active_connection {
                    let _ = storage::clear_room_order(&connection).await;
                }

                if let Err(error) = storage::clear_bridge_connection().await {
                    set_notice.set(Some(UiNotice::error("Could not clear saved bridge", error)));
                    return;
                }

                set_active_connection.set(None);
                set_selected_bridge_ip.set(String::new());
                set_username.set(String::new());
                set_lights.set(Vec::new());
                set_scenes.set(Vec::new());
                set_groups.set(Vec::new());
                set_room_order.set(Vec::new());
                set_notice.set(Some(UiNotice::success(
                    "Saved bridge removed",
                    "The local bridge session was cleared. Use settings to connect again.",
                )));
                set_page.set(AppPage::Settings);
            });
        }
    });

    let toggle_light = Callback::new({
        move |light_id: String| {
            let connection = match active_connection.get_untracked() {
                Some(connection) => connection,
                None => {
                    set_notice.set(Some(UiNotice::warning(
                        "No active bridge connection",
                        "Connect to a bridge before changing device state.",
                    )));
                    return;
                }
            };

            let maybe_light = lights
                .get_untracked()
                .into_iter()
                .find(|light| light.id == light_id);

            let light = match maybe_light {
                Some(light) => light,
                None => {
                    set_notice.set(Some(UiNotice::error(
                        "Device not found",
                        "The selected device no longer exists in the current bridge snapshot.",
                    )));
                    return;
                }
            };

            let request = SetLightStateRequest {
                bridge_ip: connection.bridge_ip,
                username: connection.username,
                light_id: light.id.clone(),
                state: LightStateUpdate {
                    on: Some(!light.is_on.unwrap_or(false)),
                    brightness: None,
                    saturation: None,
                    hue: None,
                    transition_time: Some(3),
                },
            };

            run_light_update(
                request,
                set_pending_light_ids,
                set_lights,
                set_notice,
                "Device updated",
            );
        }
    });

    let set_brightness = Callback::new({
        move |(light_id, brightness): (String, u8)| {
            let connection = match active_connection.get_untracked() {
                Some(connection) => connection,
                None => {
                    set_notice.set(Some(UiNotice::warning(
                        "No active bridge connection",
                        "Connect to a bridge before changing brightness.",
                    )));
                    return;
                }
            };

            let request = SetLightStateRequest {
                bridge_ip: connection.bridge_ip,
                username: connection.username,
                light_id,
                state: LightStateUpdate {
                    on: None,
                    brightness: Some(brightness.max(1)),
                    saturation: None,
                    hue: None,
                    transition_time: Some(4),
                },
            };

            run_light_update(
                request,
                set_pending_light_ids,
                set_lights,
                set_notice,
                "Brightness updated",
            );
        }
    });

    let toggle_room = Callback::new({
        move |room_id: String| {
            let connection = match active_connection.get_untracked() {
                Some(connection) => connection,
                None => {
                    set_notice.set(Some(UiNotice::warning(
                        "No active bridge connection",
                        "Connect to a bridge before changing room state.",
                    )));
                    return;
                }
            };

            let Some((_, room_lights)) =
                collect_room_context(&groups.get_untracked(), &lights.get_untracked(), &room_id)
            else {
                set_notice.set(Some(UiNotice::warning(
                    "Room not available",
                    "The selected room is not available in the current bridge snapshot.",
                )));
                return;
            };

            if room_lights.is_empty() {
                set_notice.set(Some(UiNotice::warning(
                    "No room devices found",
                    "This room does not currently expose any lights to control.",
                )));
                return;
            }

            let should_turn_on = room_lights
                .iter()
                .all(|light| !light.is_on.unwrap_or(false));
            let requests = room_lights
                .iter()
                .map(|light| SetLightStateRequest {
                    bridge_ip: connection.bridge_ip.clone(),
                    username: connection.username.clone(),
                    light_id: light.id.clone(),
                    state: LightStateUpdate {
                        on: Some(should_turn_on),
                        brightness: None,
                        saturation: None,
                        hue: None,
                        transition_time: Some(3),
                    },
                })
                .collect::<Vec<_>>();

            run_room_update(
                room_id,
                requests,
                set_pending_room_control_ids,
                set_lights,
                set_notice,
                if should_turn_on {
                    "Room turned on"
                } else {
                    "Room turned off"
                },
            );
        }
    });

    let set_room_brightness = Callback::new({
        move |(room_id, brightness): (String, u8)| {
            let connection = match active_connection.get_untracked() {
                Some(connection) => connection,
                None => {
                    set_notice.set(Some(UiNotice::warning(
                        "No active bridge connection",
                        "Connect to a bridge before changing room brightness.",
                    )));
                    return;
                }
            };

            let Some((_, room_lights)) =
                collect_room_context(&groups.get_untracked(), &lights.get_untracked(), &room_id)
            else {
                set_notice.set(Some(UiNotice::warning(
                    "Room not available",
                    "The selected room is not available in the current bridge snapshot.",
                )));
                return;
            };

            if room_lights.is_empty() {
                set_notice.set(Some(UiNotice::warning(
                    "No room devices found",
                    "This room does not currently expose any lights to control.",
                )));
                return;
            }

            let brightness = brightness.max(1);
            let requests = room_lights
                .iter()
                .map(|light| SetLightStateRequest {
                    bridge_ip: connection.bridge_ip.clone(),
                    username: connection.username.clone(),
                    light_id: light.id.clone(),
                    state: LightStateUpdate {
                        on: Some(true),
                        brightness: Some(brightness),
                        saturation: None,
                        hue: None,
                        transition_time: Some(4),
                    },
                })
                .collect::<Vec<_>>();

            run_room_update(
                room_id,
                requests,
                set_pending_room_control_ids,
                set_lights,
                set_notice,
                "Room brightness updated",
            );
        }
    });

    let activate_scene = Callback::new({
        move |request: ActivateSceneRequest| {
            let scene_id = request.scene_id.clone();
            let scene_name = scenes
                .get_untracked()
                .into_iter()
                .find(|scene| scene.id == scene_id)
                .map(|scene| scene.name)
                .unwrap_or_else(|| "Scene".to_string());

            set_pending_scene_id.set(Some(scene_id));
            spawn_local(async move {
                match hue::activate_hue_scene(request).await {
                    Ok(()) => {
                        set_notice.set(Some(UiNotice::new(
                            NoticeTone::Success,
                            "Scene activated",
                            format!("{scene_name} is now active on the bridge."),
                        )));
                    }
                    Err(error) => {
                        set_notice.set(Some(UiNotice::error("Scene activation failed", error)));
                    }
                }

                set_pending_scene_id.set(None);
            });
        }
    });

    let reorder_rooms = Callback::new({
        move |ordered_room_ids: Vec<String>| {
            set_room_order.set(ordered_room_ids.clone());
            let active_connection = active_connection.get_untracked();

            spawn_local(async move {
                if let Some(connection) = active_connection {
                    if let Err(error) =
                        storage::save_room_order(&connection, &ordered_room_ids).await
                    {
                        set_notice.set(Some(UiNotice::warning(
                            "Room order changed, but not saved",
                            error,
                        )));
                        return;
                    }
                }

                set_notice.set(Some(UiNotice::success(
                    "Room order saved",
                    "The current room arrangement was stored locally for this bridge.",
                )));
            });
        }
    });

    let create_curated_room_scenes = Callback::new({
        move |room_id: String| {
            let connection = match active_connection.get_untracked() {
                Some(connection) => connection,
                None => {
                    set_notice.set(Some(UiNotice::warning(
                        "No active bridge connection",
                        "Connect to a bridge before creating room scenes.",
                    )));
                    return;
                }
            };

            let room = match groups
                .get_untracked()
                .into_iter()
                .find(|group| group.id == room_id && matches!(group.kind, GroupKind::Room))
            {
                Some(room) => room,
                None => {
                    set_notice.set(Some(UiNotice::warning(
                        "Room not available",
                        "Curated scene creation currently works only for bridge rooms.",
                    )));
                    return;
                }
            };

            let lights_by_id: HashMap<String, Light> = lights
                .get_untracked()
                .into_iter()
                .map(|light| (light.id.clone(), light))
                .collect();
            let room_lights: Vec<Light> = room
                .light_ids
                .iter()
                .filter_map(|light_id| lights_by_id.get(light_id).cloned())
                .collect();

            if room_lights.is_empty() {
                set_notice.set(Some(UiNotice::warning(
                    "No room devices found",
                    "This room does not currently expose any lights to build scenes from.",
                )));
                return;
            }

            let existing_scene_names: HashSet<String> = scenes
                .get_untracked()
                .into_iter()
                .filter(|scene| scene.group_id.as_deref() == Some(room.id.as_str()))
                .map(|scene| scene.name.to_ascii_lowercase())
                .collect();
            let missing_presets: Vec<_> = curated_room_scenes()
                .iter()
                .copied()
                .filter(|preset| !existing_scene_names.contains(&preset.name.to_ascii_lowercase()))
                .collect();

            if missing_presets.is_empty() {
                set_notice.set(Some(UiNotice::info(
                    "Curated scenes already exist",
                    format!("{} already has the built-in scene pack.", room.name),
                )));
                return;
            }

            let original_states = snapshot_room_lights(&room_lights);
            let room_name = room.name.clone();
            let room_group_id = room.id.clone();
            let room_light_ids: Vec<String> =
                room_lights.iter().map(|light| light.id.clone()).collect();

            set_pending_room_ids.update(|pending| {
                pending.insert(room_group_id.clone());
            });
            set_notice.set(Some(UiNotice::info(
                "Crafting curated scenes",
                format!(
                    "Building {} scene{} for {room_name}.",
                    missing_presets.len(),
                    pluralize(missing_presets.len())
                ),
            )));

            spawn_local(async move {
                let creation_result = create_room_scene_pack(
                    connection.clone(),
                    room_group_id.clone(),
                    room_lights.clone(),
                    room_light_ids.clone(),
                    missing_presets.clone(),
                )
                .await;

                let restore_result = restore_room_lights(connection.clone(), original_states).await;

                match fetch_bridge_snapshot(connection.clone()).await {
                    Ok((fetched_lights, fetched_scenes, fetched_groups)) => {
                        set_lights.set(fetched_lights);
                        set_scenes.set(fetched_scenes);
                        set_groups.set(fetched_groups);
                        set_active_connection.set(Some(connection));
                    }
                    Err(error) => {
                        set_notice.set(Some(UiNotice::error(
                            "Scene pack created, but refresh failed",
                            error,
                        )));
                        set_pending_room_ids.update(|pending| {
                            pending.remove(&room_group_id);
                        });
                        return;
                    }
                }

                match (creation_result, restore_result) {
                    (Ok(created_names), Ok(())) => {
                        set_notice.set(Some(UiNotice::success(
                            "Curated scenes added",
                            format!("{room_name} now has {}.", created_names.join(", ")),
                        )));
                    }
                    (Ok(created_names), Err(error)) => {
                        set_notice.set(Some(UiNotice::warning(
                            "Scenes created, restore incomplete",
                            format!(
                                "{} were saved for {room_name}, but the room could not be fully restored: {error}",
                                created_names.join(", ")
                            ),
                        )));
                    }
                    (Err(error), Ok(())) => {
                        set_notice.set(Some(UiNotice::error("Scene creation failed", error)));
                    }
                    (Err(error), Err(restore_error)) => {
                        set_notice.set(Some(UiNotice::error(
                            "Scene creation failed",
                            format!("{error}. Restore also failed: {restore_error}"),
                        )));
                    }
                }

                set_pending_room_ids.update(|pending| {
                    pending.remove(&room_group_id);
                });
            });
        }
    });

    let active_light_count = Signal::derive(move || {
        lights
            .get()
            .iter()
            .filter(|light| light.is_on.unwrap_or(false))
            .count()
    });
    let reachable_light_count = Signal::derive(move || {
        lights
            .get()
            .iter()
            .filter(|light| light.reachable.unwrap_or(false))
            .count()
    });

    view! {
        <main class="app-shell">
            <div class="ambient-glow ambient-glow-primary"></div>
            <div class="ambient-glow ambient-glow-secondary"></div>

            <header class="topbar">
                <div class="topbar-brand">
                    <span class="topbar-kicker">"Hue Desktop"</span>
                    <strong>"Seasons"</strong>
                </div>

                <div class="topbar-actions">
                    <button
                        class="ghost-button"
                        on:click=move |_| {
                            set_page.set(if page.get() == AppPage::Home {
                                AppPage::Settings
                            } else {
                                AppPage::Home
                            });
                        }
                    >
                        {move || if page.get() == AppPage::Home { "Settings" } else { "Back to devices" }}
                    </button>
                </div>
            </header>

            <section class="overview-strip surface-panel">
                <div class="overview-copy">
                    <p class="hero-kicker">"Hue Home View"</p>
                    <h1>"Seasons"</h1>
                    <p class="hero-text">
                        "A quieter room-first layout with scenes and lights grouped where they belong."
                    </p>
                </div>

                <div class="overview-pills">
                    <span class="overview-pill">
                        {move || {
                            active_connection
                                .get()
                                .map(|connection| connection.bridge_ip)
                                .unwrap_or_else(|| "Offline".to_string())
                        }}
                    </span>
                    <span class="overview-pill">{move || format!("{} on", active_light_count.get())}</span>
                    <span class="overview-pill">{move || format!("{} reachable", reachable_light_count.get())}</span>
                </div>
            </section>

            <StatusBanner notice=notice />

            {move || {
                if page.get() == AppPage::Settings {
                    view! {
                        <section class="settings-shell surface-panel">
                            <div class="settings-header">
                                <div>
                                    <p class="panel-kicker">"Settings"</p>
                                    <h2>"Bridge management"</h2>
                                </div>
                                <button class="secondary-button" on:click=move |_| forget_bridge.run(())>
                                    "Forget saved bridge"
                                </button>
                            </div>

                            <p class="panel-copy">
                                "Bridge setup is intentionally kept off the main page. Pair once here, then return to the device view for daily use."
                            </p>

                            <BridgePanel
                                discovered_bridges=discovered_bridges
                                selected_bridge_ip=selected_bridge_ip
                                username=username
                                device_type=device_type
                                active_connection=active_connection
                                is_discovering=is_discovering
                                is_connecting=is_connecting
                                is_registering=is_registering
                                is_refreshing=is_refreshing
                                on_select_bridge=Callback::new(move |value: String| set_selected_bridge_ip.set(value))
                                on_username_input=Callback::new(move |value: String| set_username.set(value))
                                on_device_type_input=Callback::new(move |value: String| set_device_type.set(value))
                                on_discover=discover_bridges
                                on_connect=connect_bridge
                                on_register=pair_new_app
                            />
                        </section>
                    }.into_any()
                } else {
                    view! {
                        <section class="workspace-grid">
                            <LightGrid
                                lights=lights
                                groups=groups
                                scenes=scenes
                                room_order=room_order
                                pending_light_ids=pending_light_ids
                                pending_scene_id=pending_scene_id
                                pending_room_ids=pending_room_ids
                                pending_room_control_ids=pending_room_control_ids
                                active_connection=active_connection
                                is_refreshing=is_refreshing
                                on_open_settings=Callback::new(move |_| set_page.set(AppPage::Settings))
                                on_toggle_light=toggle_light
                                on_set_brightness=set_brightness
                                on_toggle_room=toggle_room
                                on_set_room_brightness=set_room_brightness
                                on_activate_scene=activate_scene
                                on_create_curated_scenes=create_curated_room_scenes
                                on_reorder_rooms=reorder_rooms
                            />
                        </section>
                    }.into_any()
                }
            }}
        </main>
    }
}

fn run_light_update(
    request: SetLightStateRequest,
    set_pending_light_ids: WriteSignal<HashSet<String>>,
    set_lights: WriteSignal<Vec<Light>>,
    set_notice: WriteSignal<Option<UiNotice>>,
    success_title: &'static str,
) {
    let light_id = request.light_id.clone();
    let state = request.state.clone();

    set_pending_light_ids.update(|pending| {
        pending.insert(light_id.clone());
    });

    spawn_local(async move {
        match hue::set_hue_light_state(request).await {
            Ok(()) => {
                set_lights.update(|lights| {
                    if let Some(light) = lights.iter_mut().find(|light| light.id == light_id) {
                        apply_state_update(light, &state);
                    }
                });
                set_notice.set(Some(UiNotice::new(
                    NoticeTone::Success,
                    success_title,
                    "The latest change was accepted by the bridge.",
                )));
            }
            Err(error) => {
                set_notice.set(Some(UiNotice::error("Device update failed", error)));
            }
        }

        set_pending_light_ids.update(|pending| {
            pending.remove(&light_id);
        });
    });
}

fn run_room_update(
    room_id: String,
    requests: Vec<SetLightStateRequest>,
    set_pending_room_control_ids: WriteSignal<HashSet<String>>,
    set_lights: WriteSignal<Vec<Light>>,
    set_notice: WriteSignal<Option<UiNotice>>,
    success_title: &'static str,
) {
    if requests.is_empty() {
        return;
    }

    let light_states = requests
        .iter()
        .map(|request| (request.light_id.clone(), request.state.clone()))
        .collect::<HashMap<_, _>>();

    set_pending_room_control_ids.update(|pending| {
        pending.insert(room_id.clone());
    });

    spawn_local(async move {
        let mut error = None;

        for request in requests {
            if let Err(request_error) = hue::set_hue_light_state(request).await {
                error = Some(request_error);
                break;
            }
        }

        match error {
            None => {
                set_lights.update(|lights| {
                    for light in lights {
                        if let Some(state) = light_states.get(&light.id) {
                            apply_state_update(light, state);
                        }
                    }
                });
                set_notice.set(Some(UiNotice::new(
                    NoticeTone::Success,
                    success_title,
                    "The latest room change was accepted by the bridge.",
                )));
            }
            Some(error) => {
                set_notice.set(Some(UiNotice::error("Room update failed", error)));
            }
        }

        set_pending_room_control_ids.update(|pending| {
            pending.remove(&room_id);
        });
    });
}

async fn fetch_bridge_snapshot(
    connection: BridgeConnection,
) -> Result<(Vec<Light>, Vec<Scene>, Vec<Group>), String> {
    let fetched_lights = hue::list_hue_lights(connection.clone()).await?;
    let fetched_scenes = hue::list_hue_scenes(connection.clone()).await?;
    let fetched_groups = hue::list_hue_groups(connection).await?;
    Ok((fetched_lights, fetched_scenes, fetched_groups))
}

async fn create_room_scene_pack(
    connection: BridgeConnection,
    room_group_id: String,
    room_lights: Vec<Light>,
    room_light_ids: Vec<String>,
    presets: Vec<hue::CuratedScenePreset>,
) -> Result<Vec<String>, String> {
    let mut created_names = Vec::new();

    for preset in presets {
        for (index, light) in room_lights.iter().enumerate() {
            let request = SetLightStateRequest {
                bridge_ip: connection.bridge_ip.clone(),
                username: connection.username.clone(),
                light_id: light.id.clone(),
                state: preset_light_state(preset, index, room_lights.len()),
            };
            hue::set_hue_light_state(request).await?;
        }

        let request = CreateSceneRequest {
            bridge_ip: connection.bridge_ip.clone(),
            username: connection.username.clone(),
            group_id: room_group_id.clone(),
            scene_name: preset.name.to_string(),
            light_ids: room_light_ids.clone(),
        };
        hue::create_hue_scene(request).await?;
        created_names.push(preset.name.to_string());
    }

    Ok(created_names)
}

fn snapshot_room_lights(lights: &[Light]) -> Vec<SetLightStateRequest> {
    lights
        .iter()
        .map(|light| SetLightStateRequest {
            bridge_ip: String::new(),
            username: String::new(),
            light_id: light.id.clone(),
            state: LightStateUpdate {
                on: light.is_on,
                brightness: light.brightness,
                saturation: light.saturation,
                hue: light.hue,
                transition_time: Some(4),
            },
        })
        .collect()
}

async fn restore_room_lights(
    connection: BridgeConnection,
    mut states: Vec<SetLightStateRequest>,
) -> Result<(), String> {
    for request in &mut states {
        request.bridge_ip = connection.bridge_ip.clone();
        request.username = connection.username.clone();
        hue::set_hue_light_state(request.clone()).await?;
    }

    Ok(())
}

fn collect_room_context(
    groups: &[Group],
    lights: &[Light],
    room_id: &str,
) -> Option<(Group, Vec<Light>)> {
    let room = groups
        .iter()
        .find(|group| group.id == room_id && matches!(group.kind, GroupKind::Room))?
        .clone();

    let room_lights = room
        .light_ids
        .iter()
        .filter_map(|light_id| lights.iter().find(|light| light.id == *light_id).cloned())
        .collect::<Vec<_>>();

    Some((room, room_lights))
}

fn apply_state_update(light: &mut Light, state: &LightStateUpdate) {
    if let Some(is_on) = state.on {
        light.is_on = Some(is_on);
    }

    if let Some(brightness) = state.brightness {
        light.brightness = Some(brightness);
    }

    if let Some(saturation) = state.saturation {
        light.saturation = Some(saturation);
    }

    if let Some(hue) = state.hue {
        light.hue = Some(hue);
    }
}

fn pluralize(count: usize) -> &'static str {
    if count == 1 {
        ""
    } else {
        "s"
    }
}
