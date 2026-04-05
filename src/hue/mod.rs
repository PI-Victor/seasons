pub mod api;
pub mod models;
pub mod presets;

pub use api::{
    activate_hue_scene, create_hue_scene, create_hue_user, delete_hue_scene, discover_hue_bridges,
    get_hue_automation_detail, list_hue_automations, list_hue_entertainment_areas, list_hue_groups,
    list_hue_lights, list_hue_scenes, list_hue_sensors, list_pipewire_output_targets,
    set_hue_automation_enabled, set_hue_light_state, start_hue_audio_sync, stop_hue_audio_sync,
    update_hue_audio_sync, update_hue_automation,
};
pub use models::{
    ActivateSceneRequest, AudioSyncColorPalette, AudioSyncSpeedMode, AudioSyncStartRequest,
    AudioSyncStartResult, AudioSyncUpdateRequest, Automation, AutomationConfigEntry,
    AutomationConfigValue, AutomationDetail, BridgeConnection, CreateSceneRequest,
    CreateUserRequest, DeleteSceneRequest, DiscoveredBridge, EntertainmentArea, Group, GroupKind,
    Light, LightStateUpdate, PipeWireOutputTarget, Scene, Sensor, SetAutomationEnabledRequest,
    SetLightStateRequest, UpdateAutomationRequest,
};
pub use presets::{curated_room_scenes, preset_light_state, CuratedScenePreset};
