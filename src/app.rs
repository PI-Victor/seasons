use crate::desktop;
use crate::hue::{
    self, curated_room_scenes, preset_light_state, ActivateSceneRequest, AudioSyncColorPalette,
    AudioSyncSpeedMode, AudioSyncStartRequest, AudioSyncUpdateRequest, Automation,
    BridgeConnection, CreateSceneRequest, CreateUserRequest, DeleteSceneRequest, EntertainmentArea,
    Group, GroupKind, Light, LightStateUpdate, PipeWireOutputTarget, Scene, Sensor,
    SetAutomationEnabledRequest, SetLightStateRequest,
};
use crate::storage::{self, AudioSyncPreferences};
use crate::theme::{apply_theme_preference, ThemeMode, ThemePalette, ThemePreference};
use crate::ui::{
    AudioSyncPanel, AutomationPanel, BridgePanel, DevicePanel, LightGrid, NoticeTone,
    SceneComposerRequest, StatusBanner, ThemePanel, UiNotice,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::collections::{HashMap, HashSet};
use wasm_bindgen::{closure::Closure, JsCast};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AppPage {
    Home,
    Devices,
    Automations,
    Settings,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AudioSyncSelection {
    area_id: String,
    pipewire_target_object: String,
    speed_mode: AudioSyncSpeedMode,
    color_palette: AudioSyncColorPalette,
    base_color_hex: Option<String>,
    brightness_ceiling: Option<u8>,
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
    let (sensors, set_sensors) = signal(Vec::<Sensor>::new());
    let (automations, set_automations) = signal(Vec::<Automation>::new());
    let (active_connection, set_active_connection) = signal(None::<BridgeConnection>);
    let (entertainment_areas, set_entertainment_areas) = signal(Vec::<EntertainmentArea>::new());
    let (selected_entertainment_area_id, set_selected_entertainment_area_id) =
        signal(String::new());
    let (pipewire_targets, set_pipewire_targets) = signal(Vec::<PipeWireOutputTarget>::new());
    let (selected_pipewire_target_object, set_selected_pipewire_target_object) =
        signal(String::new());
    let (selected_sync_speed_mode, set_selected_sync_speed_mode) =
        signal(AudioSyncSpeedMode::default());
    let (selected_sync_color_palette, set_selected_sync_color_palette) =
        signal(AudioSyncColorPalette::default());
    let (notice, set_notice) = signal(Some(UiNotice::info(
        "Local-first Hue control",
        "Bridge setup is stored locally after a successful connection. Use settings only when you need to change it.",
    )));
    let (is_discovering, set_is_discovering) = signal(false);
    let (is_registering, set_is_registering) = signal(false);
    let (is_connecting, set_is_connecting) = signal(false);
    let (is_refreshing, set_is_refreshing) = signal(false);
    let (is_loading_entertainment_areas, set_is_loading_entertainment_areas) = signal(false);
    let (is_loading_pipewire_targets, set_is_loading_pipewire_targets) = signal(false);
    let (is_audio_syncing, set_is_audio_syncing) = signal(false);
    let (is_audio_sync_starting, set_is_audio_sync_starting) = signal(false);
    let (did_restore_session, set_did_restore_session) = signal(false);
    let (did_restore_theme, set_did_restore_theme) = signal(false);
    let (did_restore_audio_sync_preferences, set_did_restore_audio_sync_preferences) =
        signal(false);
    let (did_load_pipewire_targets, set_did_load_pipewire_targets) = signal(false);
    let (last_applied_audio_sync_selection, set_last_applied_audio_sync_selection) =
        signal(AudioSyncSelection {
            area_id: String::new(),
            pipewire_target_object: String::new(),
            speed_mode: AudioSyncSpeedMode::default(),
            color_palette: AudioSyncColorPalette::default(),
            base_color_hex: None,
            brightness_ceiling: None,
        });
    let (pending_scene_id, set_pending_scene_id) = signal(None::<String>);
    let (pending_room_ids, set_pending_room_ids) = signal(HashSet::<String>::new());
    let (pending_room_control_ids, set_pending_room_control_ids) = signal(HashSet::<String>::new());
    let (pending_room_brightness_timeouts, set_pending_room_brightness_timeouts) =
        signal(HashMap::<String, i32>::new());
    let (pending_light_ids, set_pending_light_ids) = signal(HashSet::<String>::new());
    let (pending_automation_ids, set_pending_automation_ids) = signal(HashSet::<String>::new());
    let (active_scene_by_group, set_active_scene_by_group) =
        signal(HashMap::<String, String>::new());
    let (last_activated_scene_id, set_last_activated_scene_id) = signal(None::<String>);
    let (room_order, set_room_order) = signal(Vec::<String>::new());
    let (theme_preference, set_theme_preference) = signal(ThemePreference::default());

    let refresh_entertainment_areas = Callback::new({
        move |connection: BridgeConnection| {
            set_is_loading_entertainment_areas.set(true);
            spawn_local(async move {
                match hue::list_hue_entertainment_areas(connection).await {
                    Ok(areas) => {
                        let current_selection = selected_entertainment_area_id
                            .get_untracked()
                            .trim()
                            .to_string();
                        let has_current = areas.iter().any(|area| area.id == current_selection);
                        let next_selection = if has_current {
                            current_selection
                        } else {
                            areas
                                .first()
                                .map(|area| area.id.clone())
                                .unwrap_or_default()
                        };

                        set_entertainment_areas.set(areas);
                        set_selected_entertainment_area_id.set(next_selection.clone());
                        let preferences = AudioSyncPreferences {
                            selected_entertainment_area_id: if next_selection.is_empty() {
                                None
                            } else {
                                Some(next_selection)
                            },
                            selected_pipewire_target_object: if selected_pipewire_target_object
                                .get_untracked()
                                .trim()
                                .is_empty()
                            {
                                None
                            } else {
                                Some(selected_pipewire_target_object.get_untracked())
                            },
                            selected_sync_speed_mode: selected_sync_speed_mode.get_untracked(),
                            selected_sync_color_palette: selected_sync_color_palette
                                .get_untracked(),
                        };
                        let _ = storage::save_audio_sync_preferences(&preferences).await;
                    }
                    Err(error) => {
                        set_entertainment_areas.set(Vec::new());
                        set_selected_entertainment_area_id.set(String::new());
                        set_notice.set(Some(UiNotice::warning(
                            "Could not load entertainment areas",
                            error,
                        )));
                    }
                }

                set_is_loading_entertainment_areas.set(false);
            });
        }
    });

    let refresh_automations = Callback::new({
        move |connection: BridgeConnection| {
            spawn_local(async move {
                match hue::list_hue_automations(connection).await {
                    Ok(loaded_automations) => set_automations.set(loaded_automations),
                    Err(error) => {
                        set_automations.set(Vec::new());
                        set_notice
                            .set(Some(UiNotice::warning("Could not load automations", error)));
                    }
                }
            });
        }
    });

    let refresh_pipewire_targets = Callback::new({
        move |()| {
            set_is_loading_pipewire_targets.set(true);
            spawn_local(async move {
                match hue::list_pipewire_output_targets().await {
                    Ok(targets) => {
                        let current_selection = selected_pipewire_target_object
                            .get_untracked()
                            .trim()
                            .to_string();
                        let has_current = targets
                            .iter()
                            .any(|target| target.target_object == current_selection);
                        let next_selection = if has_current {
                            current_selection
                        } else {
                            targets
                                .first()
                                .map(|target| target.target_object.clone())
                                .unwrap_or_default()
                        };

                        set_pipewire_targets.set(targets);
                        set_selected_pipewire_target_object.set(next_selection.clone());

                        let preferences = AudioSyncPreferences {
                            selected_entertainment_area_id: if selected_entertainment_area_id
                                .get_untracked()
                                .trim()
                                .is_empty()
                            {
                                None
                            } else {
                                Some(selected_entertainment_area_id.get_untracked())
                            },
                            selected_pipewire_target_object: if next_selection.is_empty() {
                                None
                            } else {
                                Some(next_selection)
                            },
                            selected_sync_speed_mode: selected_sync_speed_mode.get_untracked(),
                            selected_sync_color_palette: selected_sync_color_palette
                                .get_untracked(),
                        };
                        let _ = storage::save_audio_sync_preferences(&preferences).await;
                    }
                    Err(error) => {
                        set_pipewire_targets.set(Vec::new());
                        set_selected_pipewire_target_object.set(String::new());
                        set_notice.set(Some(UiNotice::warning(
                            "Could not load audio sources",
                            error,
                        )));
                    }
                }

                set_is_loading_pipewire_targets.set(false);
            });
        }
    });

    let refresh_bridge_state = Callback::new({
        move |connection: BridgeConnection| {
            set_is_refreshing.set(true);
            spawn_local({
                let connection = connection.clone();
                async move {
                    match fetch_bridge_snapshot(connection.clone()).await {
                        Ok((fetched_lights, fetched_scenes, fetched_groups, fetched_sensors)) => {
                            let light_count = fetched_lights.len();
                            let scene_count = fetched_scenes.len();
                            let sensor_count = fetched_sensors.len();
                            let saved_room_order = storage::load_room_order(&connection)
                                .await
                                .unwrap_or_else(|_| Vec::new());
                            set_lights.set(fetched_lights);
                            set_scenes.set(fetched_scenes);
                            set_groups.set(fetched_groups);
                            set_sensors.set(fetched_sensors);
                            set_active_scene_by_group.set(HashMap::new());
                            set_last_activated_scene_id.set(None);
                            set_room_order.set(saved_room_order);
                            set_active_connection.set(Some(connection.clone()));
                            refresh_entertainment_areas.run(connection.clone());
                            refresh_automations.run(connection.clone());
                            if let Err(error) = storage::save_bridge_connection(&connection).await {
                                set_notice.set(Some(UiNotice::warning(
                                    "Session active, but not persisted",
                                    error,
                                )));
                            } else {
                                set_notice.set(Some(UiNotice::success(
                                    "Bridge connected",
                                    format!(
                                        "Loaded {light_count} devices, {sensor_count} sensor{}, and {scene_count} scenes from the bridge.",
                                        if sensor_count == 1 { "" } else { "s" }
                                    ),
                                )));
                            }
                        }
                        Err(error) => {
                            set_active_connection.set(None);
                            set_lights.set(Vec::new());
                            set_scenes.set(Vec::new());
                            set_groups.set(Vec::new());
                            set_sensors.set(Vec::new());
                            set_automations.set(Vec::new());
                            set_active_scene_by_group.set(HashMap::new());
                            set_last_activated_scene_id.set(None);
                            set_room_order.set(Vec::new());
                            set_entertainment_areas.set(Vec::new());
                            set_selected_entertainment_area_id.set(String::new());
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

    Effect::new(move |_| {
        if did_restore_theme.get() {
            return;
        }

        set_did_restore_theme.set(true);
        spawn_local(async move {
            match storage::load_theme_preference().await {
                Ok(preference) => set_theme_preference.set(preference),
                Err(error) => {
                    set_notice.set(Some(UiNotice::warning(
                        "Could not restore saved theme",
                        error,
                    )));
                }
            }
        });
    });

    Effect::new(move |_| {
        if did_restore_audio_sync_preferences.get() {
            return;
        }

        set_did_restore_audio_sync_preferences.set(true);
        spawn_local(async move {
            match storage::load_audio_sync_preferences().await {
                Ok(preferences) => {
                    if let Some(area_id) = preferences.selected_entertainment_area_id {
                        set_selected_entertainment_area_id.set(area_id);
                    }
                    if let Some(target_object) = preferences.selected_pipewire_target_object {
                        set_selected_pipewire_target_object.set(target_object);
                    }
                    set_selected_sync_speed_mode.set(preferences.selected_sync_speed_mode);
                    set_selected_sync_color_palette.set(preferences.selected_sync_color_palette);
                }
                Err(error) => {
                    set_notice.set(Some(UiNotice::warning(
                        "Could not restore audio sync preferences",
                        error,
                    )));
                }
            }
        });
    });

    Effect::new(move |_| {
        if did_load_pipewire_targets.get() {
            return;
        }

        set_did_load_pipewire_targets.set(true);
        refresh_pipewire_targets.run(());
    });

    Effect::new(move |_| {
        let area_id = selected_entertainment_area_id.get();
        let color_palette = selected_sync_color_palette.get();
        let (base_color_hex, brightness_ceiling) = if area_id.trim().is_empty()
            || !matches!(color_palette, AudioSyncColorPalette::CurrentRoom)
        {
            (None, None)
        } else {
            derive_audio_sync_visual_profile(
                &entertainment_areas.get(),
                &groups.get(),
                &lights.get(),
                &scenes.get(),
                &active_scene_by_group.get(),
                last_activated_scene_id.get().as_deref(),
                &area_id,
            )
        };

        let selection = AudioSyncSelection {
            area_id,
            pipewire_target_object: selected_pipewire_target_object.get(),
            speed_mode: selected_sync_speed_mode.get(),
            color_palette,
            base_color_hex,
            brightness_ceiling,
        };

        if !is_audio_syncing.get() {
            set_last_applied_audio_sync_selection.set(selection);
            return;
        }

        let previous = last_applied_audio_sync_selection.get();
        if previous == selection {
            return;
        }

        let Some(connection) = active_connection.get() else {
            return;
        };

        let areas = entertainment_areas.get();
        let current_groups = groups.get();
        let current_lights = lights.get();
        set_last_applied_audio_sync_selection.set(selection.clone());

        spawn_local(async move {
            let can_update_in_place = previous.area_id == selection.area_id
                && previous.pipewire_target_object == selection.pipewire_target_object;

            if can_update_in_place {
                match request_audio_sync_update(
                    selection.speed_mode,
                    selection.color_palette,
                    &areas,
                    &current_groups,
                    &current_lights,
                    &scenes.get(),
                    &active_scene_by_group.get(),
                    last_activated_scene_id.get().as_deref(),
                    &selection.area_id,
                )
                .await
                {
                    Ok(()) => {}
                    Err(error) => {
                        set_is_audio_syncing.set(false);
                        set_notice.set(Some(UiNotice::error("Audio sync failed", error)));
                    }
                }
            } else {
                match request_audio_sync_start(
                    connection.clone(),
                    selection.area_id.clone(),
                    selection.pipewire_target_object.clone(),
                    selection.speed_mode,
                    selection.color_palette,
                    &areas,
                    &current_groups,
                    &current_lights,
                    &scenes.get(),
                    &active_scene_by_group.get(),
                    last_activated_scene_id.get().as_deref(),
                )
                .await
                {
                    Ok(result) => {
                        set_active_connection.set(Some(result.connection.clone()));
                        let _ = storage::save_bridge_connection(&result.connection).await;
                        set_notice.set(Some(UiNotice::success(
                            "Audio sync updated",
                            "Hue Entertainment streaming was refreshed with the new sync settings.",
                        )));
                    }
                    Err(error) => {
                        set_is_audio_syncing.set(false);
                        set_notice.set(Some(UiNotice::error("Audio sync failed", error)));
                    }
                }
            }
        });
    });

    Effect::new(move |_| {
        let preference = theme_preference.get();
        let _ = apply_theme_preference(&preference);
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
            let remembered_connection = active_connection.get_untracked().filter(|connection| {
                connection.bridge_ip == bridge_ip && connection.username == username_value
            });

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
                    client_key: remembered_connection
                        .as_ref()
                        .and_then(|connection| connection.client_key.clone()),
                    application_id: remembered_connection
                        .as_ref()
                        .and_then(|connection| connection.application_id.clone()),
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
                            client_key: registered.client_key.clone(),
                            application_id: None,
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
                let _ = hue::stop_hue_audio_sync().await;
                set_is_audio_syncing.set(false);
                set_entertainment_areas.set(Vec::new());
                set_selected_entertainment_area_id.set(String::new());
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
                set_sensors.set(Vec::new());
                set_automations.set(Vec::new());
                set_room_order.set(Vec::new());
                set_notice.set(Some(UiNotice::success(
                    "Saved bridge removed",
                    "The local bridge session was cleared. Use settings to connect again.",
                )));
                set_page.set(AppPage::Settings);
            });
        }
    });

    let save_theme_preference = Callback::new({
        move |preference: ThemePreference| {
            set_theme_preference.set(preference.clone());
            spawn_local(async move {
                if let Err(error) = storage::save_theme_preference(&preference).await {
                    set_notice.set(Some(UiNotice::warning(
                        "Theme changed, but not persisted",
                        error,
                    )));
                }
            });
        }
    });

    let set_theme_mode = Callback::new({
        move |mode: ThemeMode| {
            let mut preference = theme_preference.get_untracked();
            preference.mode = mode;
            save_theme_preference.run(preference);
        }
    });

    let set_theme_palette = Callback::new({
        move |palette: ThemePalette| {
            let mut preference = theme_preference.get_untracked();
            preference.palette = palette;
            save_theme_preference.run(preference);
        }
    });

    let select_entertainment_area = Callback::new({
        move |area_id: String| {
            let trimmed = area_id.trim().to_string();
            set_selected_entertainment_area_id.set(trimmed.clone());
            spawn_local(async move {
                let preferences = AudioSyncPreferences {
                    selected_entertainment_area_id: if trimmed.is_empty() {
                        None
                    } else {
                        Some(trimmed)
                    },
                    selected_pipewire_target_object: if selected_pipewire_target_object
                        .get_untracked()
                        .trim()
                        .is_empty()
                    {
                        None
                    } else {
                        Some(selected_pipewire_target_object.get_untracked())
                    },
                    selected_sync_speed_mode: selected_sync_speed_mode.get_untracked(),
                    selected_sync_color_palette: selected_sync_color_palette.get_untracked(),
                };
                if let Err(error) = storage::save_audio_sync_preferences(&preferences).await {
                    set_notice.set(Some(UiNotice::warning(
                        "Audio sync area changed, but not persisted",
                        error,
                    )));
                }
            });
        }
    });

    let select_pipewire_target = Callback::new({
        move |target_object: String| {
            let trimmed = target_object.trim().to_string();
            set_selected_pipewire_target_object.set(trimmed.clone());
            spawn_local(async move {
                let preferences = AudioSyncPreferences {
                    selected_entertainment_area_id: if selected_entertainment_area_id
                        .get_untracked()
                        .trim()
                        .is_empty()
                    {
                        None
                    } else {
                        Some(selected_entertainment_area_id.get_untracked())
                    },
                    selected_pipewire_target_object: if trimmed.is_empty() {
                        None
                    } else {
                        Some(trimmed)
                    },
                    selected_sync_speed_mode: selected_sync_speed_mode.get_untracked(),
                    selected_sync_color_palette: selected_sync_color_palette.get_untracked(),
                };
                if let Err(error) = storage::save_audio_sync_preferences(&preferences).await {
                    set_notice.set(Some(UiNotice::warning(
                        "Audio source changed, but not persisted",
                        error,
                    )));
                }
            });
        }
    });

    let select_sync_speed_mode = Callback::new({
        move |mode: AudioSyncSpeedMode| {
            set_selected_sync_speed_mode.set(mode);
            spawn_local(async move {
                let preferences = AudioSyncPreferences {
                    selected_entertainment_area_id: if selected_entertainment_area_id
                        .get_untracked()
                        .trim()
                        .is_empty()
                    {
                        None
                    } else {
                        Some(selected_entertainment_area_id.get_untracked())
                    },
                    selected_pipewire_target_object: if selected_pipewire_target_object
                        .get_untracked()
                        .trim()
                        .is_empty()
                    {
                        None
                    } else {
                        Some(selected_pipewire_target_object.get_untracked())
                    },
                    selected_sync_speed_mode: mode,
                    selected_sync_color_palette: selected_sync_color_palette.get_untracked(),
                };
                if let Err(error) = storage::save_audio_sync_preferences(&preferences).await {
                    set_notice.set(Some(UiNotice::warning(
                        "Sync speed changed, but not persisted",
                        error,
                    )));
                }
            });
        }
    });

    let select_sync_color_palette = Callback::new({
        move |palette: AudioSyncColorPalette| {
            set_selected_sync_color_palette.set(palette);
            spawn_local(async move {
                let preferences = AudioSyncPreferences {
                    selected_entertainment_area_id: if selected_entertainment_area_id
                        .get_untracked()
                        .trim()
                        .is_empty()
                    {
                        None
                    } else {
                        Some(selected_entertainment_area_id.get_untracked())
                    },
                    selected_pipewire_target_object: if selected_pipewire_target_object
                        .get_untracked()
                        .trim()
                        .is_empty()
                    {
                        None
                    } else {
                        Some(selected_pipewire_target_object.get_untracked())
                    },
                    selected_sync_speed_mode: selected_sync_speed_mode.get_untracked(),
                    selected_sync_color_palette: palette,
                };
                if let Err(error) = storage::save_audio_sync_preferences(&preferences).await {
                    set_notice.set(Some(UiNotice::warning(
                        "Sync palette changed, but not persisted",
                        error,
                    )));
                }
            });
        }
    });

    let start_audio_sync = Callback::new({
        move |()| {
            if is_audio_sync_starting.get_untracked() {
                return;
            }

            let Some(connection) = active_connection.get_untracked() else {
                set_notice.set(Some(UiNotice::warning(
                    "No active bridge connection",
                    "Connect to the bridge before starting Hue audio sync.",
                )));
                return;
            };

            if connection.client_key.as_deref().unwrap_or("").is_empty() {
                set_notice.set(Some(UiNotice::warning(
                    "Streaming credentials missing",
                    "This bridge session does not have the Hue client key needed for Entertainment streaming. Pair this app again from settings to enable audio sync.",
                )));
                return;
            }

            let area_id = selected_entertainment_area_id
                .get_untracked()
                .trim()
                .to_string();
            if area_id.is_empty() {
                set_notice.set(Some(UiNotice::warning(
                    "Entertainment area required",
                    "Create or choose an entertainment area in the Hue app before starting audio sync.",
                )));
                return;
            }

            let pipewire_target_object = selected_pipewire_target_object
                .get_untracked()
                .trim()
                .to_string();
            let speed_mode = selected_sync_speed_mode.get_untracked();
            let color_palette = selected_sync_color_palette.get_untracked();
            let areas = entertainment_areas.get_untracked();
            let current_groups = groups.get_untracked();
            let current_lights = lights.get_untracked();
            let (base_color_hex, brightness_ceiling) =
                if matches!(color_palette, AudioSyncColorPalette::CurrentRoom) {
                    derive_audio_sync_visual_profile(
                        &areas,
                        &current_groups,
                        &current_lights,
                        &scenes.get_untracked(),
                        &active_scene_by_group.get_untracked(),
                        last_activated_scene_id.get_untracked().as_deref(),
                        &area_id,
                    )
                } else {
                    (None, None)
                };

            set_is_audio_sync_starting.set(true);
            spawn_local(async move {
                match request_audio_sync_start(
                    connection.clone(),
                    area_id.clone(),
                    pipewire_target_object,
                    speed_mode,
                    color_palette,
                    &areas,
                    &current_groups,
                    &current_lights,
                    &scenes.get_untracked(),
                    &active_scene_by_group.get_untracked(),
                    last_activated_scene_id.get_untracked().as_deref(),
                )
                .await
                {
                    Ok(result) => {
                        set_active_connection.set(Some(result.connection.clone()));
                        let _ = storage::save_bridge_connection(&result.connection).await;
                        set_is_audio_syncing.set(true);
                        set_is_audio_sync_starting.set(false);
                        set_last_applied_audio_sync_selection.set(AudioSyncSelection {
                            area_id: area_id.clone(),
                            pipewire_target_object: selected_pipewire_target_object.get_untracked(),
                            speed_mode,
                            color_palette,
                            base_color_hex,
                            brightness_ceiling,
                        });
                        set_notice.set(Some(UiNotice::success(
                            "Audio sync started",
                            "Hue Entertainment streaming is active for the selected area.",
                        )));
                    }
                    Err(error) => {
                        set_is_audio_syncing.set(false);
                        set_is_audio_sync_starting.set(false);
                        set_notice.set(Some(UiNotice::error("Audio sync failed", error)));
                    }
                }
            });
        }
    });

    let stop_audio_sync = Callback::new({
        move |()| {
            spawn_local(async move {
                match hue::stop_hue_audio_sync().await {
                    Ok(()) => {
                        set_is_audio_syncing.set(false);
                        set_is_audio_sync_starting.set(false);
                        set_notice.set(Some(UiNotice::success(
                            "Audio sync stopped",
                            "Hue Entertainment streaming has been disabled for the current area.",
                        )));
                    }
                    Err(error) => {
                        set_is_audio_sync_starting.set(false);
                        set_notice.set(Some(UiNotice::error("Could not stop audio sync", error)));
                    }
                }
            });
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
            let optimistic_state = LightStateUpdate {
                on: Some(true),
                brightness: Some(brightness),
                saturation: None,
                hue: None,
                transition_time: Some(4),
            };
            let room_light_ids = room_lights
                .iter()
                .map(|light| light.id.clone())
                .collect::<HashSet<_>>();
            set_lights.update(|lights| {
                for light in lights {
                    if room_light_ids.contains(&light.id) {
                        apply_state_update(light, &optimistic_state);
                    }
                }
            });

            if let Some(previous_timeout) = pending_room_brightness_timeouts
                .get_untracked()
                .get(&room_id)
                .copied()
            {
                if let Some(window) = leptos::web_sys::window() {
                    window.clear_timeout_with_handle(previous_timeout);
                }
            }

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
            let scheduled_requests = requests.clone();

            let timeout_room_id = room_id.clone();
            let scheduled_room_id = room_id.clone();
            let callback = Closure::once(move || {
                set_pending_room_brightness_timeouts.update(|timeouts| {
                    timeouts.remove(&timeout_room_id);
                });
                run_room_update(
                    scheduled_room_id,
                    scheduled_requests,
                    set_pending_room_control_ids,
                    set_lights,
                    set_notice,
                    "Room brightness updated",
                );
            });

            if let Some(window) = leptos::web_sys::window() {
                match window.set_timeout_with_callback_and_timeout_and_arguments_0(
                    callback.as_ref().unchecked_ref(),
                    140,
                ) {
                    Ok(timeout_id) => {
                        set_pending_room_brightness_timeouts.update(|timeouts| {
                            timeouts.insert(room_id, timeout_id);
                        });
                        callback.forget();
                    }
                    Err(_) => {
                        run_room_update(
                            room_id,
                            requests,
                            set_pending_room_control_ids,
                            set_lights,
                            set_notice,
                            "Room brightness updated",
                        );
                    }
                }
            } else {
                run_room_update(
                    room_id,
                    requests,
                    set_pending_room_control_ids,
                    set_lights,
                    set_notice,
                    "Room brightness updated",
                );
            }
        }
    });

    let toggle_all_lights = Callback::new({
        move |()| {
            let connection = match active_connection.get_untracked() {
                Some(connection) => connection,
                None => {
                    set_notice.set(Some(UiNotice::warning(
                        "No active bridge connection",
                        "Connect to a bridge before changing all devices.",
                    )));
                    return;
                }
            };

            let current_lights = lights.get_untracked();
            if current_lights.is_empty() {
                set_notice.set(Some(UiNotice::warning(
                    "No devices found",
                    "The current bridge snapshot does not contain any lights to control.",
                )));
                return;
            }

            let should_turn_on = current_lights
                .iter()
                .all(|light| !light.is_on.unwrap_or(false));
            let requests = current_lights
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
                "__all__".to_string(),
                requests,
                set_pending_room_control_ids,
                set_lights,
                set_notice,
                if should_turn_on {
                    "All devices turned on"
                } else {
                    "All devices turned off"
                },
            );
        }
    });

    let activate_scene = Callback::new({
        move |request: ActivateSceneRequest| {
            let scene_id = request.scene_id.clone();
            let group_id = request.group_id.clone();
            let refresh_connection = BridgeConnection {
                bridge_ip: request.bridge_ip.clone(),
                username: request.username.clone(),
                client_key: None,
                application_id: None,
            };
            let scene_name = scenes
                .get_untracked()
                .into_iter()
                .find(|scene| scene.id == scene_id)
                .map(|scene| scene.name)
                .unwrap_or_else(|| "Scene".to_string());

            set_pending_scene_id.set(Some(scene_id.clone()));
            spawn_local(async move {
                match hue::activate_hue_scene(request).await {
                    Ok(()) => {
                        if let Some(group_id) = group_id {
                            set_active_scene_by_group.update(|active_scenes| {
                                active_scenes.insert(group_id, scene_id.clone());
                            });
                        }
                        set_last_activated_scene_id.set(Some(scene_id.clone()));

                        let refresh_error = match fetch_bridge_snapshot(refresh_connection).await {
                            Ok((
                                fetched_lights,
                                fetched_scenes,
                                fetched_groups,
                                fetched_sensors,
                            )) => {
                                set_lights.set(fetched_lights);
                                set_scenes.set(fetched_scenes);
                                set_groups.set(fetched_groups);
                                set_sensors.set(fetched_sensors);
                                None
                            }
                            Err(error) => Some(error),
                        };

                        if let Some(error) = refresh_error {
                            set_notice.set(Some(UiNotice::warning(
                                "Scene activated, refresh failed",
                                format!("{scene_name} is active, but the app could not reload bridge state: {error}"),
                            )));
                        } else {
                            set_notice.set(Some(UiNotice::new(
                                NoticeTone::Success,
                                "Scene activated",
                                format!("{scene_name} is now active on the bridge."),
                            )));
                        }
                    }
                    Err(error) => {
                        set_notice.set(Some(UiNotice::error("Scene activation failed", error)));
                    }
                }

                set_pending_scene_id.set(None);
            });
        }
    });

    let delete_scene = Callback::new({
        move |request: DeleteSceneRequest| {
            let scene_id = request.scene_id.clone();
            let refresh_connection = BridgeConnection {
                bridge_ip: request.bridge_ip.clone(),
                username: request.username.clone(),
                client_key: None,
                application_id: None,
            };
            let deleted_scene_name = scenes
                .get_untracked()
                .into_iter()
                .find(|scene| scene.id == scene_id)
                .map(|scene| scene.name)
                .unwrap_or_else(|| "Scene".to_string());

            set_pending_scene_id.set(Some(scene_id.clone()));
            spawn_local(async move {
                match hue::delete_hue_scene(request).await {
                    Ok(()) => {
                        set_active_scene_by_group.update(|active_scenes| {
                            active_scenes.retain(|_, active_scene_id| active_scene_id != &scene_id);
                        });

                        match fetch_bridge_snapshot(refresh_connection).await {
                            Ok((
                                fetched_lights,
                                fetched_scenes,
                                fetched_groups,
                                fetched_sensors,
                            )) => {
                                set_lights.set(fetched_lights);
                                set_scenes.set(fetched_scenes);
                                set_groups.set(fetched_groups);
                                set_sensors.set(fetched_sensors);
                                set_notice.set(Some(UiNotice::success(
                                    "Scene deleted",
                                    format!("{deleted_scene_name} was removed from the bridge."),
                                )));
                            }
                            Err(error) => {
                                set_notice.set(Some(UiNotice::warning(
                                    "Scene deleted, refresh failed",
                                    format!("{deleted_scene_name} was removed, but the app could not reload bridge state: {error}"),
                                )));
                            }
                        }
                    }
                    Err(error) => {
                        set_notice.set(Some(UiNotice::error("Scene deletion failed", error)));
                    }
                }

                set_pending_scene_id.set(None);
            });
        }
    });

    let toggle_automation = Callback::new({
        move |(automation_id, enabled): (String, bool)| {
            let Some(connection) = active_connection.get_untracked() else {
                set_notice.set(Some(UiNotice::warning(
                    "No active bridge connection",
                    "Connect to a bridge before changing automations.",
                )));
                return;
            };

            let automation_name = automations
                .get_untracked()
                .into_iter()
                .find(|automation| automation.id == automation_id)
                .map(|automation| automation.name)
                .unwrap_or_else(|| "Automation".to_string());

            set_pending_automation_ids.update(|pending| {
                pending.insert(automation_id.clone());
            });

            spawn_local(async move {
                let request = SetAutomationEnabledRequest {
                    connection,
                    automation_id: automation_id.clone(),
                    enabled,
                };

                match hue::set_hue_automation_enabled(request).await {
                    Ok(()) => {
                        set_automations.update(|items| {
                            if let Some(automation) = items
                                .iter_mut()
                                .find(|automation| automation.id == automation_id)
                            {
                                automation.enabled = Some(enabled);
                            }
                        });
                        set_notice.set(Some(UiNotice::success(
                            "Automation updated",
                            format!(
                                "{automation_name} was turned {}.",
                                if enabled { "on" } else { "off" }
                            ),
                        )));
                    }
                    Err(error) => {
                        set_notice.set(Some(UiNotice::error("Automation update failed", error)));
                    }
                }

                set_pending_automation_ids.update(|pending| {
                    pending.remove(&automation_id);
                });
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
                        "Connect to a bridge before changing a device.",
                    )));
                    return;
                }
            };

            let Some(light) = lights
                .get_untracked()
                .into_iter()
                .find(|light| light.id == light_id)
            else {
                set_notice.set(Some(UiNotice::warning(
                    "Device not available",
                    "The selected device is not available in the current bridge snapshot.",
                )));
                return;
            };

            let should_turn_on = !light.is_on.unwrap_or(false);
            let request = SetLightStateRequest {
                bridge_ip: connection.bridge_ip,
                username: connection.username,
                light_id: light.id.clone(),
                state: LightStateUpdate {
                    on: Some(should_turn_on),
                    brightness: None,
                    saturation: None,
                    hue: None,
                    transition_time: Some(3),
                },
            };

            run_light_update(
                light.id,
                request,
                set_pending_light_ids,
                set_lights,
                set_notice,
                if should_turn_on {
                    "Device turned on"
                } else {
                    "Device turned off"
                },
            );
        }
    });

    let set_light_brightness = Callback::new({
        move |(light_id, brightness): (String, u8)| {
            let connection = match active_connection.get_untracked() {
                Some(connection) => connection,
                None => {
                    set_notice.set(Some(UiNotice::warning(
                        "No active bridge connection",
                        "Connect to a bridge before changing device brightness.",
                    )));
                    return;
                }
            };

            let Some(light) = lights
                .get_untracked()
                .into_iter()
                .find(|light| light.id == light_id)
            else {
                set_notice.set(Some(UiNotice::warning(
                    "Device not available",
                    "The selected device is not available in the current bridge snapshot.",
                )));
                return;
            };

            let request = SetLightStateRequest {
                bridge_ip: connection.bridge_ip,
                username: connection.username,
                light_id: light.id.clone(),
                state: LightStateUpdate {
                    on: Some(true),
                    brightness: Some(brightness.max(1)),
                    saturation: None,
                    hue: None,
                    transition_time: Some(4),
                },
            };

            run_light_update(
                light.id,
                request,
                set_pending_light_ids,
                set_lights,
                set_notice,
                "Device brightness updated",
            );
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
                    Ok((fetched_lights, fetched_scenes, fetched_groups, fetched_sensors)) => {
                        set_lights.set(fetched_lights);
                        set_scenes.set(fetched_scenes);
                        set_groups.set(fetched_groups);
                        set_sensors.set(fetched_sensors);
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

    let create_custom_room_scene = Callback::new({
        move |request: SceneComposerRequest| {
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

            let room =
                match groups.get_untracked().into_iter().find(|group| {
                    group.id == request.room_id && matches!(group.kind, GroupKind::Room)
                }) {
                    Some(room) => room,
                    None => {
                        set_notice.set(Some(UiNotice::warning(
                            "Room not available",
                            "Custom scene creation currently works only for bridge rooms.",
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

            let scene_name = request.scene_name.trim().to_string();
            if scene_name.is_empty() {
                set_notice.set(Some(UiNotice::warning(
                    "Scene name required",
                    "Give the scene a name before saving it to the bridge.",
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
                "Saving custom scene",
                format!("Building {scene_name} for {room_name}."),
            )));

            spawn_local(async move {
                let scene_state = scene_composer_state(&request);
                let creation_result = create_single_room_scene(
                    connection.clone(),
                    room_group_id.clone(),
                    &scene_name,
                    &room_lights,
                    &room_light_ids,
                    scene_state,
                )
                .await;

                let restore_result = restore_room_lights(connection.clone(), original_states).await;

                match fetch_bridge_snapshot(connection.clone()).await {
                    Ok((fetched_lights, fetched_scenes, fetched_groups, fetched_sensors)) => {
                        set_lights.set(fetched_lights);
                        set_scenes.set(fetched_scenes);
                        set_groups.set(fetched_groups);
                        set_sensors.set(fetched_sensors);
                    }
                    Err(error) => {
                        set_notice.set(Some(UiNotice::warning(
                            "Scene created, refresh failed",
                            format!("{scene_name} was saved, but the app could not reload bridge state: {error}"),
                        )));
                    }
                }

                match (creation_result, restore_result) {
                    (Ok(()), Ok(())) => {
                        set_notice.set(Some(UiNotice::success(
                            "Custom scene saved",
                            format!("{scene_name} is now available in {room_name}."),
                        )));
                    }
                    (Ok(()), Err(error)) => {
                        set_notice.set(Some(UiNotice::warning(
                            "Scene saved, restore failed",
                            format!("{scene_name} was stored, but the room could not be restored: {error}"),
                        )));
                    }
                    (Err(error), _) => {
                        set_notice.set(Some(UiNotice::error("Scene creation failed", error)));
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

    let quit_application = Callback::new({
        move |()| {
            spawn_local(async move {
                if let Err(error) = desktop::quit_app().await {
                    set_notice.set(Some(UiNotice::error("Could not close the app", error)));
                }
            });
        }
    });
    let reachable_light_count = Signal::derive(move || {
        lights
            .get()
            .iter()
            .filter(|light| light.reachable.unwrap_or(false))
            .count()
    });
    let temperature_summary = Signal::derive(move || {
        sensors.get().iter().find_map(|sensor| {
            let is_temperature = sensor
                .sensor_type
                .as_deref()
                .unwrap_or_default()
                .to_ascii_lowercase()
                .contains("temperature");
            if is_temperature {
                sensor.summary.clone()
            } else {
                None
            }
        })
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

                <div class="topbar-toolbar">
                    <div class="topbar-actions">
                        <button
                            class=move || {
                                if page.get() == AppPage::Home {
                                    "ghost-button nav-button is-active"
                                } else {
                                    "ghost-button nav-button"
                                }
                            }
                            on:click=move |_| set_page.set(AppPage::Home)
                        >
                            <span class="nav-button-content">
                                <span class="fa-solid fa-house" aria-hidden="true"></span>
                                <span>"Rooms"</span>
                            </span>
                        </button>
                        <button
                            class=move || {
                                if page.get() == AppPage::Devices {
                                    "ghost-button nav-button is-active"
                                } else {
                                    "ghost-button nav-button"
                                }
                            }
                            on:click=move |_| set_page.set(AppPage::Devices)
                        >
                            <span class="nav-button-content">
                                <span class="fa-solid fa-lightbulb" aria-hidden="true"></span>
                                <span>"Devices"</span>
                            </span>
                        </button>
                        <button
                            class=move || {
                                if page.get() == AppPage::Automations {
                                    "ghost-button nav-button is-active"
                                } else {
                                    "ghost-button nav-button"
                                }
                            }
                            on:click=move |_| set_page.set(AppPage::Automations)
                        >
                            <span class="nav-button-content">
                                <span class="fa-solid fa-wand-magic-sparkles" aria-hidden="true"></span>
                                <span>"Automations"</span>
                            </span>
                        </button>
                        <button
                            class=move || {
                                if page.get() == AppPage::Settings {
                                    "ghost-button nav-button is-active"
                                } else {
                                    "ghost-button nav-button"
                                }
                            }
                            on:click=move |_| set_page.set(AppPage::Settings)
                        >
                            <span class="nav-button-content">
                                <span class="fa-solid fa-gear" aria-hidden="true"></span>
                                <span>"Settings"</span>
                            </span>
                        </button>
                    </div>
                    <button
                        class="ghost-button quit-button topbar-quit"
                        on:click=move |_| quit_application.run(())
                    >
                        <span class="nav-button-content">
                            <span class="fa-solid fa-xmark" aria-hidden="true"></span>
                            <span>"Quit"</span>
                        </span>
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
                    {move || {
                        temperature_summary
                            .get()
                            .map(|summary| view! { <span class="overview-pill">{summary}</span> })
                    }}
                </div>
            </section>

            <StatusBanner notice=notice />

            {move || {
                if page.get() == AppPage::Settings {
                    view! {
                        <section class="settings-stack">
                            <ThemePanel
                                theme_preference=theme_preference
                                on_palette_change=set_theme_palette
                                on_mode_change=set_theme_mode
                            />

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
                                on_forget=forget_bridge
                            />
                        </section>
                    }.into_any()
                } else if page.get() == AppPage::Devices {
                    view! {
                        <section class="workspace-grid">
                            <DevicePanel
                                lights=lights
                                sensors=sensors
                                groups=groups
                                pending_light_ids=pending_light_ids
                                on_toggle_light=toggle_light
                                on_set_light_brightness=set_light_brightness
                            />
                        </section>
                    }.into_any()
                } else if page.get() == AppPage::Automations {
                    view! {
                        <section class="workspace-grid">
                            <AutomationPanel
                                active_connection=active_connection
                                automations=automations
                                pending_automation_ids=pending_automation_ids
                                on_toggle_automation=toggle_automation
                            />
                        </section>
                    }.into_any()
                } else {
                    view! {
                        <section class="workspace-grid">
                            <AudioSyncPanel
                                active_connection=active_connection
                                entertainment_areas=entertainment_areas
                                selected_entertainment_area_id=selected_entertainment_area_id
                                pipewire_targets=pipewire_targets
                                selected_pipewire_target_object=selected_pipewire_target_object
                                selected_sync_speed_mode=selected_sync_speed_mode
                                selected_sync_color_palette=selected_sync_color_palette
                                is_loading_areas=is_loading_entertainment_areas
                                is_loading_pipewire_targets=is_loading_pipewire_targets
                                is_audio_syncing=is_audio_syncing
                                is_audio_sync_starting=is_audio_sync_starting
                                on_select_area=select_entertainment_area
                                on_select_pipewire_target=select_pipewire_target
                                on_select_sync_speed_mode=select_sync_speed_mode
                                on_select_sync_color_palette=select_sync_color_palette
                                on_start=start_audio_sync
                                on_stop=stop_audio_sync
                            />

                            <LightGrid
                                lights=lights
                                groups=groups
                                scenes=scenes
                                room_order=room_order
                                pending_scene_id=pending_scene_id
                                pending_room_ids=pending_room_ids
                                pending_room_control_ids=pending_room_control_ids
                                pending_light_ids=pending_light_ids
                                active_scene_by_group=active_scene_by_group
                                active_connection=active_connection
                                is_refreshing=is_refreshing
                                on_open_settings=Callback::new(move |_| set_page.set(AppPage::Settings))
                                on_toggle_all_lights=toggle_all_lights
                                on_toggle_room=toggle_room
                                on_set_room_brightness=set_room_brightness
                                on_toggle_light=toggle_light
                                on_set_light_brightness=set_light_brightness
                                on_activate_scene=activate_scene
                                on_delete_scene=delete_scene
                                on_create_curated_scenes=create_curated_room_scenes
                                on_create_custom_scene=create_custom_room_scene
                                on_reorder_rooms=reorder_rooms
                            />
                        </section>
                    }.into_any()
                }
            }}
        </main>
    }
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

fn run_light_update(
    light_id: String,
    request: SetLightStateRequest,
    set_pending_light_ids: WriteSignal<HashSet<String>>,
    set_lights: WriteSignal<Vec<Light>>,
    set_notice: WriteSignal<Option<UiNotice>>,
    success_title: &'static str,
) {
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
                    "The latest device change was accepted by the bridge.",
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

async fn fetch_bridge_snapshot(
    connection: BridgeConnection,
) -> Result<(Vec<Light>, Vec<Scene>, Vec<Group>, Vec<Sensor>), String> {
    let fetched_lights = hue::list_hue_lights(connection.clone()).await?;
    let fetched_scenes = hue::list_hue_scenes(connection.clone()).await?;
    let fetched_groups = hue::list_hue_groups(connection.clone()).await?;
    let fetched_sensors = hue::list_hue_sensors(connection).await?;
    Ok((
        fetched_lights,
        fetched_scenes,
        fetched_groups,
        fetched_sensors,
    ))
}

async fn request_audio_sync_start(
    connection: BridgeConnection,
    entertainment_area_id: String,
    pipewire_target_object: String,
    speed_mode: AudioSyncSpeedMode,
    color_palette: AudioSyncColorPalette,
    entertainment_areas: &[EntertainmentArea],
    groups: &[Group],
    lights: &[Light],
    scenes: &[Scene],
    active_scene_by_group: &HashMap<String, String>,
    last_activated_scene_id: Option<&str>,
) -> Result<hue::AudioSyncStartResult, String> {
    let (base_color_hex, brightness_ceiling) = derive_audio_sync_visual_profile(
        entertainment_areas,
        groups,
        lights,
        scenes,
        active_scene_by_group,
        last_activated_scene_id,
        &entertainment_area_id,
    );

    hue::start_hue_audio_sync(AudioSyncStartRequest {
        connection,
        entertainment_area_id,
        pipewire_target_object: if pipewire_target_object.trim().is_empty() {
            None
        } else {
            Some(pipewire_target_object)
        },
        speed_mode,
        color_palette,
        base_color_hex,
        brightness_ceiling,
    })
    .await
}

async fn request_audio_sync_update(
    speed_mode: AudioSyncSpeedMode,
    color_palette: AudioSyncColorPalette,
    entertainment_areas: &[EntertainmentArea],
    groups: &[Group],
    lights: &[Light],
    scenes: &[Scene],
    active_scene_by_group: &HashMap<String, String>,
    last_activated_scene_id: Option<&str>,
    entertainment_area_id: &str,
) -> Result<(), String> {
    let (base_color_hex, brightness_ceiling) = derive_audio_sync_visual_profile(
        entertainment_areas,
        groups,
        lights,
        scenes,
        active_scene_by_group,
        last_activated_scene_id,
        entertainment_area_id,
    );

    hue::update_hue_audio_sync(AudioSyncUpdateRequest {
        speed_mode,
        color_palette,
        base_color_hex,
        brightness_ceiling,
    })
    .await
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

async fn create_single_room_scene(
    connection: BridgeConnection,
    room_group_id: String,
    scene_name: &str,
    room_lights: &[Light],
    room_light_ids: &[String],
    state: LightStateUpdate,
) -> Result<(), String> {
    for light in room_lights {
        let request = SetLightStateRequest {
            bridge_ip: connection.bridge_ip.clone(),
            username: connection.username.clone(),
            light_id: light.id.clone(),
            state: state.clone(),
        };
        hue::set_hue_light_state(request).await?;
    }

    let request = CreateSceneRequest {
        bridge_ip: connection.bridge_ip,
        username: connection.username,
        group_id: room_group_id,
        scene_name: scene_name.to_string(),
        light_ids: room_light_ids.to_vec(),
    };
    hue::create_hue_scene(request).await?;

    Ok(())
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

fn scene_composer_state(request: &SceneComposerRequest) -> LightStateUpdate {
    LightStateUpdate {
        on: Some(true),
        brightness: Some(request.brightness.max(1)),
        saturation: Some(request.saturation.max(1)),
        hue: Some(((f32::from(request.hue_degrees) / 360.0) * 65_535.0).round() as u16),
        transition_time: Some(4),
    }
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
        light.xy = None;
    }

    if let Some(hue) = state.hue {
        light.hue = Some(hue);
        light.xy = None;
    }
}

fn derive_audio_sync_visual_profile(
    areas: &[EntertainmentArea],
    groups: &[Group],
    lights: &[Light],
    scenes: &[Scene],
    active_scene_by_group: &HashMap<String, String>,
    last_activated_scene_id: Option<&str>,
    area_id: &str,
) -> (Option<String>, Option<u8>) {
    let selected_scene = last_activated_scene_id
        .and_then(|scene_id| scenes.iter().find(|scene| scene.id == scene_id));
    let matched_group = areas
        .iter()
        .find(|area| area.id == area_id)
        .and_then(|area| best_matching_audio_sync_group(area, groups));
    let scene_group = selected_scene
        .and_then(|scene| scene.group_id.as_deref())
        .and_then(|group_id| groups.iter().find(|group| group.id == group_id));
    let brightness_group = scene_group.or(matched_group);

    let brightness = brightness_group
        .and_then(|group| lights_for_group(group, lights))
        .and_then(|group_lights| preferred_sync_brightness(&group_lights))
        .or_else(|| preferred_sync_brightness(&lights.iter().collect::<Vec<_>>()));

    let color = selected_scene
        .and_then(|scene| scene.preview_color_main.clone())
        .or_else(|| {
            matched_group
                .and_then(|group| {
                    active_scene_by_group
                        .get(&group.id)
                        .and_then(|scene_id| scenes.iter().find(|scene| scene.id == *scene_id))
                })
                .and_then(|scene| scene.preview_color_main.clone())
        })
        .or_else(|| {
            brightness_group
                .and_then(|group| lights_for_group(group, lights))
                .and_then(|group_lights| {
                    average_light_color_identity_rgb(&group_lights).map(rgb_to_hex)
                })
        });

    (color, brightness)
}

fn lights_for_group<'a>(group: &'a Group, lights: &'a [Light]) -> Option<Vec<&'a Light>> {
    let group_lights = group
        .light_ids
        .iter()
        .filter_map(|light_id| lights.iter().find(|light| light.id == *light_id))
        .collect::<Vec<_>>();

    if group_lights.is_empty() {
        None
    } else {
        Some(group_lights)
    }
}

fn best_matching_audio_sync_group<'a>(
    area: &EntertainmentArea,
    groups: &'a [Group],
) -> Option<&'a Group> {
    let area_name = normalize_audio_sync_name(&area.name);
    let mut best: Option<(&Group, usize)> = None;

    for group in groups.iter().filter(|group| {
        matches!(group.kind, GroupKind::Room | GroupKind::Zone) && !group.light_ids.is_empty()
    }) {
        let group_name = normalize_audio_sync_name(&group.name);
        let score = if area_name == group_name {
            100
        } else if area_name.contains(&group_name) || group_name.contains(&area_name) {
            75
        } else {
            let overlap = shared_token_count(&area_name, &group_name);
            if overlap == 0 {
                continue;
            }
            overlap * 10
        };

        match best {
            Some((_, best_score)) if best_score >= score => {}
            _ => best = Some((group, score)),
        }
    }

    best.map(|(group, _)| group)
}

fn normalize_audio_sync_name(name: &str) -> String {
    name.to_lowercase()
        .replace("entertainment", "")
        .replace("area", "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

fn shared_token_count(left: &str, right: &str) -> usize {
    let right_tokens = right.split_whitespace().collect::<HashSet<_>>();
    left.split_whitespace()
        .filter(|token| right_tokens.contains(token))
        .count()
}

fn average_light_brightness(lights: &[&Light]) -> Option<u8> {
    let (sum, count) = lights
        .iter()
        .filter_map(|light| light.brightness.map(u32::from))
        .fold((0_u32, 0_u32), |(sum, count), value| {
            (sum + value, count + 1)
        });

    let raw_average = sum.checked_div(count)? as f32;
    Some(((raw_average / 254.0) * 100.0).round().clamp(1.0, 100.0) as u8)
}

fn preferred_sync_brightness(lights: &[&Light]) -> Option<u8> {
    brightest_active_light_brightness(lights)
        .or_else(|| average_active_light_brightness(lights))
        .or_else(|| brightest_known_light_brightness(lights))
        .or_else(|| average_light_brightness(lights))
}

fn brightest_active_light_brightness(lights: &[&Light]) -> Option<u8> {
    lights
        .iter()
        .filter(|light| light.is_on.unwrap_or(false))
        .filter_map(|light| light.brightness)
        .max()
        .map(brightness_to_percent)
}

fn average_active_light_brightness(lights: &[&Light]) -> Option<u8> {
    let active_lights = lights
        .iter()
        .copied()
        .filter(|light| light.is_on.unwrap_or(false))
        .collect::<Vec<_>>();

    if active_lights.is_empty() {
        None
    } else {
        average_light_brightness(&active_lights)
    }
}

fn brightest_known_light_brightness(lights: &[&Light]) -> Option<u8> {
    lights
        .iter()
        .filter_map(|light| light.brightness)
        .max()
        .map(brightness_to_percent)
}

fn brightness_to_percent(value: u8) -> u8 {
    ((f32::from(value) / 254.0) * 100.0)
        .round()
        .clamp(1.0, 100.0) as u8
}

fn average_light_color_identity_rgb(lights: &[&Light]) -> Option<(u8, u8, u8)> {
    let (sum_red, sum_green, sum_blue, count) = lights
        .iter()
        .filter_map(|light| light_color_identity_rgb(light))
        .fold(
            (0_u32, 0_u32, 0_u32, 0_u32),
            |(sr, sg, sb, count), (r, g, b)| {
                (
                    sr + u32::from(r),
                    sg + u32::from(g),
                    sb + u32::from(b),
                    count + 1,
                )
            },
        );

    Some((
        sum_red.checked_div(count)? as u8,
        sum_green.checked_div(count)? as u8,
        sum_blue.checked_div(count)? as u8,
    ))
}

fn light_color_identity_rgb(light: &Light) -> Option<(u8, u8, u8)> {
    if let Some([x, y]) = light.xy {
        if let Some(rgb) = xy_to_rgb(x, y, 254) {
            return Some(rgb);
        }
    }

    let hue = light.hue?;
    let saturation = light.saturation?;
    Some(hsv_to_rgb(hue, saturation, 254))
}

fn rgb_to_hex((red, green, blue): (u8, u8, u8)) -> String {
    format!("#{red:02x}{green:02x}{blue:02x}")
}

fn hsv_to_rgb(hue: u16, saturation: u8, brightness: u8) -> (u8, u8, u8) {
    let hue = (hue as f32 / 65_535.0) * 360.0;
    let saturation = (saturation as f32 / 254.0).clamp(0.0, 1.0);
    let value = ((brightness as f32 / 254.0) * 0.12 + 0.86).clamp(0.0, 1.0);
    hsv_to_rgb_float(hue, saturation, value)
}

fn xy_to_rgb(x: f32, y: f32, brightness: u8) -> Option<(u8, u8, u8)> {
    if !(0.0..=1.0).contains(&x) || !(0.0..=1.0).contains(&y) || y <= f32::EPSILON {
        return None;
    }

    let z = 1.0 - x - y;
    let luminance = (brightness as f32 / 254.0).clamp(0.08, 1.0);
    let x_component = (luminance / y) * x;
    let z_component = (luminance / y) * z;

    let mut red = x_component * 1.656492 - luminance * 0.354851 - z_component * 0.255038;
    let mut green = -x_component * 0.707196 + luminance * 1.655397 + z_component * 0.036152;
    let mut blue = x_component * 0.051713 - luminance * 0.121364 + z_component * 1.01153;

    red = gamma_correct(red.max(0.0));
    green = gamma_correct(green.max(0.0));
    blue = gamma_correct(blue.max(0.0));

    let max = red.max(green).max(blue);
    if max > 1.0 {
        red /= max;
        green /= max;
        blue /= max;
    }

    Some((
        (red * 255.0).round() as u8,
        (green * 255.0).round() as u8,
        (blue * 255.0).round() as u8,
    ))
}

fn gamma_correct(component: f32) -> f32 {
    if component <= 0.003_130_8 {
        12.92 * component
    } else {
        (1.0 + 0.055) * component.powf(1.0 / 2.4) - 0.055
    }
}

fn hsv_to_rgb_float(hue: f32, saturation: f32, value: f32) -> (u8, u8, u8) {
    let hue = hue.rem_euclid(360.0);
    let chroma = value * saturation;
    let segment = hue / 60.0;
    let x = chroma * (1.0 - ((segment % 2.0) - 1.0).abs());

    let (red, green, blue) = match segment as u32 {
        0 => (chroma, x, 0.0),
        1 => (x, chroma, 0.0),
        2 => (0.0, chroma, x),
        3 => (0.0, x, chroma),
        4 => (x, 0.0, chroma),
        _ => (chroma, 0.0, x),
    };

    let match_value = value - chroma;
    (
        ((red + match_value) * 255.0).round() as u8,
        ((green + match_value) * 255.0).round() as u8,
        ((blue + match_value) * 255.0).round() as u8,
    )
}

fn pluralize(count: usize) -> &'static str {
    if count == 1 {
        ""
    } else {
        "s"
    }
}
