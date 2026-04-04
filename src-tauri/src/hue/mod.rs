pub mod client;
pub mod config;
pub mod entertainment;
pub mod error;
pub mod models;

pub use client::HueBridgeClient;
pub use config::HueBridgeConfig;
pub use models::{
    ActivateSceneRequest, AudioSyncColorPalette, AudioSyncSpeedMode, AudioSyncStartRequest,
    AudioSyncStartResult, AudioSyncUpdateRequest, BridgeConnection, CreateSceneRequest, CreateUserRequest,
    DeleteSceneRequest, DiscoveredBridge, EntertainmentArea, Group, Light, PipeWireOutputTarget,
    RegisteredApp, Scene, SetLightStateRequest,
};
