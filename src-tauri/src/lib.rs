mod app_state;
mod commands;
mod hue;
mod theme;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::discover_hue_bridges,
            commands::create_hue_user,
            commands::list_hue_lights,
            commands::list_hue_scenes,
            commands::list_hue_groups,
            commands::set_hue_light_state,
            commands::activate_hue_scene,
            commands::create_hue_scene,
            commands::delete_hue_scene,
            commands::load_persisted_bridge_connection,
            commands::save_persisted_bridge_connection,
            commands::clear_persisted_bridge_connection,
            commands::load_persisted_room_order,
            commands::save_persisted_room_order,
            commands::clear_persisted_room_order,
            commands::quit_app,
            commands::load_theme_preference,
            commands::save_theme_preference,
        ]);

    if let Err(error) = builder.run(tauri::generate_context!()) {
        eprintln!("error while running tauri application: {error}");
    }
}
