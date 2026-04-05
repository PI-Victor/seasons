mod app_state;
mod audio;
mod commands;
mod hue;
mod theme;

use tracing_subscriber::EnvFilter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_logging();

    let builder = tauri::Builder::default()
        .manage(audio::AudioSyncManager::new())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::discover_hue_bridges,
            commands::create_hue_user,
            commands::list_hue_lights,
            commands::list_hue_scenes,
            commands::list_hue_sensors,
            commands::list_hue_groups,
            commands::list_hue_entertainment_areas,
            commands::list_hue_automations,
            commands::get_hue_automation_detail,
            commands::list_pipewire_output_targets,
            commands::set_hue_light_state,
            commands::set_hue_automation_enabled,
            commands::update_hue_automation,
            commands::activate_hue_scene,
            commands::create_hue_scene,
            commands::delete_hue_scene,
            commands::start_hue_audio_sync,
            commands::stop_hue_audio_sync,
            commands::update_hue_audio_sync,
            commands::load_persisted_bridge_connection,
            commands::save_persisted_bridge_connection,
            commands::clear_persisted_bridge_connection,
            commands::load_persisted_room_order,
            commands::save_persisted_room_order,
            commands::clear_persisted_room_order,
            commands::load_audio_sync_preferences,
            commands::save_audio_sync_preferences,
            commands::quit_app,
            commands::load_theme_preference,
            commands::save_theme_preference,
        ]);

    if let Err(error) = builder.run(tauri::generate_context!()) {
        eprintln!("error while running tauri application: {error}");
    }
}

fn init_logging() {
    let default_filter = if cfg!(debug_assertions) {
        "seasons_lib=debug"
    } else {
        "seasons_lib=info"
    };

    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(default_filter))
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_names(true)
        .compact()
        .try_init();
}
