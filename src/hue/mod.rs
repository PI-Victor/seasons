pub mod api;
pub mod models;
pub mod presets;

pub use api::{
    activate_hue_scene, create_hue_scene, create_hue_user, delete_hue_scene, discover_hue_bridges,
    list_hue_entertainment_areas, list_hue_groups, list_hue_lights, list_hue_scenes,
    list_pipewire_output_targets, set_hue_light_state, start_hue_audio_sync, stop_hue_audio_sync,
    update_hue_audio_sync,
};
pub use models::{
    ActivateSceneRequest, AudioSyncColorPalette, AudioSyncSpeedMode, AudioSyncStartRequest,
    AudioSyncStartResult, AudioSyncUpdateRequest, BridgeConnection, CreateSceneRequest, CreateUserRequest,
    DeleteSceneRequest, DiscoveredBridge, EntertainmentArea, Group, GroupKind, Light,
    LightStateUpdate, PipeWireOutputTarget, Scene, SetLightStateRequest,
};
pub use presets::{curated_room_scenes, preset_light_state, CuratedScenePreset};
