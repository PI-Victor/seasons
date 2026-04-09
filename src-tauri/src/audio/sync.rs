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

use crate::audio::analysis::AudioFeatures;
use crate::audio::capture::{AudioCaptureHandle, start_sink_capture};
use crate::hue::entertainment::{EntertainmentStreamSession, build_rgb_channels, empty_rgb_frame};
use crate::hue::error::HueError;
use crate::hue::models::AudioSyncStartResult;
use crate::hue::models::{
    AudioSyncColorPalette, AudioSyncPreview, AudioSyncSpeedMode, LightStateUpdate,
};
use crate::hue::{BridgeConnection, EntertainmentArea, HueBridgeClient, HueBridgeConfig, Light};
use std::collections::HashSet;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex, RwLock};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::{self, Duration};
use tracing::{debug, info, trace};

pub struct AudioSyncManager {
    running: Mutex<Option<RunningAudioSyncSession>>,
    preview: Arc<RwLock<Option<AudioSyncPreview>>>,
}

struct RunningAudioSyncSession {
    area_id: String,
    connection: BridgeConnection,
    stop_tx: oneshot::Sender<()>,
    stream_task: JoinHandle<()>,
    capture: AudioCaptureHandle,
    profile: Arc<RwLock<StreamProfile>>,
    restore_snapshot: Vec<LightRestoreState>,
}

#[derive(Clone)]
struct LightRestoreState {
    light_id: String,
    state: LightStateUpdate,
}

impl AudioSyncManager {
    pub fn new() -> Self {
        Self {
            running: Mutex::new(None),
            preview: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn start(
        &self,
        connection: BridgeConnection,
        area: EntertainmentArea,
        pipewire_target_object: Option<String>,
        speed_mode: AudioSyncSpeedMode,
        color_palette: AudioSyncColorPalette,
        base_color_hex: Option<String>,
        brightness_ceiling: Option<u8>,
    ) -> Result<AudioSyncStartResult, HueError> {
        self.stop().await?;
        info!(
            bridge_ip = %connection.bridge_ip,
            area_id = %area.id,
            area_name = %area.name,
            pipewire_target_object = ?pipewire_target_object,
            speed_mode = ?speed_mode,
            color_palette = ?color_palette,
            base_color_hex = ?base_color_hex,
            brightness_ceiling = ?brightness_ceiling,
            "starting Hue audio sync"
        );

        let config = HueBridgeConfig::authenticated(
            connection.bridge_ip.clone(),
            connection.username.clone(),
        )?;
        let client = HueBridgeClient::new(config)?;
        let restore_snapshot = capture_restore_snapshot(&client, &area).await?;

        let mut resolved_connection = connection.clone();
        if resolved_connection
            .application_id
            .as_deref()
            .unwrap_or("")
            .is_empty()
        {
            debug!("resolving hue-application-id for entertainment streaming");
            resolved_connection.application_id = Some(client.resolve_application_id().await?);
        }

        debug!("starting entertainment area");
        client.start_entertainment_area(&area.id).await?;
        let stream_profile = Arc::new(RwLock::new(StreamProfile::new(
            speed_mode,
            color_palette,
            base_color_hex,
            brightness_ceiling,
        )));
        debug!("connecting DTLS entertainment stream");
        let mut stream = EntertainmentStreamSession::connect(&resolved_connection, &area).await?;
        let initial_frame = {
            let profile = stream_profile.read().map_err(|_| {
                HueError::EntertainmentStream("audio sync profile lock poisoned".to_string())
            })?;
            profile.hold_channels(&area)
        };
        debug!("writing initial entertainment hold frame");
        stream.write_rgb_frame(&initial_frame).await?;

        let (capture, feature_rx) = start_sink_capture(pipewire_target_object.as_deref())?;
        let (stop_tx, stop_rx) = oneshot::channel();
        let task_area = area.clone();
        let task_profile = Arc::clone(&stream_profile);
        let task_preview = Arc::clone(&self.preview);

        let stream_task = tokio::spawn(async move {
            let _ = run_stream_loop(
                stream,
                task_area,
                feature_rx,
                stop_rx,
                task_profile,
                task_preview,
            )
            .await;
        });

        let mut guard = self.running.lock().map_err(|_| {
            HueError::EntertainmentStream("audio sync manager lock poisoned".to_string())
        })?;
        *guard = Some(RunningAudioSyncSession {
            area_id: area.id.clone(),
            connection: resolved_connection.clone(),
            stop_tx,
            stream_task,
            capture,
            profile: stream_profile,
            restore_snapshot,
        });

        info!("Hue audio sync started successfully");

        Ok(AudioSyncStartResult {
            connection: resolved_connection,
            entertainment_area_id: area.id,
        })
    }

    pub async fn stop(&self) -> Result<(), HueError> {
        let running = {
            let mut guard = self.running.lock().map_err(|_| {
                HueError::EntertainmentStream("audio sync manager lock poisoned".to_string())
            })?;
            guard.take()
        };

        let Some(running) = running else {
            if let Ok(mut preview) = self.preview.write() {
                *preview = None;
            }
            return Ok(());
        };
        info!(area_id = %running.area_id, "stopping Hue audio sync");

        let _ = running.stop_tx.send(());
        let _ = running.stream_task.await;
        running.capture.stop();

        let config = HueBridgeConfig::authenticated(
            running.connection.bridge_ip.clone(),
            running.connection.username.clone(),
        )?;
        let client = HueBridgeClient::new(config)?;
        let stop_result = client.stop_entertainment_area(&running.area_id).await;
        let restore_result = restore_snapshot(&client, &running.restore_snapshot).await;
        if let Ok(mut preview) = self.preview.write() {
            *preview = None;
        }

        if let Err(stop_error) = stop_result {
            if let Err(restore_error) = restore_result {
                return Err(HueError::EntertainmentStream(format!(
                    "stopping stream failed: {stop_error}; restoring previous light states also failed: {restore_error}"
                )));
            }
            return Err(stop_error);
        }
        restore_result?;
        info!("Hue audio sync stopped");
        Ok(())
    }

    pub fn preview(&self) -> Result<Option<AudioSyncPreview>, HueError> {
        let preview = self.preview.read().map_err(|_| {
            HueError::EntertainmentStream("audio sync preview lock poisoned".to_string())
        })?;
        Ok(preview.clone())
    }

    pub fn update(
        &self,
        speed_mode: AudioSyncSpeedMode,
        color_palette: AudioSyncColorPalette,
        base_color_hex: Option<String>,
        brightness_ceiling: Option<u8>,
    ) -> Result<(), HueError> {
        let profile = {
            let guard = self.running.lock().map_err(|_| {
                HueError::EntertainmentStream("audio sync manager lock poisoned".to_string())
            })?;
            let Some(running) = guard.as_ref() else {
                return Err(HueError::EntertainmentStream(
                    "audio sync is not currently active".to_string(),
                ));
            };
            Arc::clone(&running.profile)
        };

        let mut profile_guard = profile.write().map_err(|_| {
            HueError::EntertainmentStream("audio sync profile lock poisoned".to_string())
        })?;
        *profile_guard = StreamProfile::new(
            speed_mode,
            color_palette,
            base_color_hex.clone(),
            brightness_ceiling,
        );
        info!(
            speed_mode = ?speed_mode,
            color_palette = ?color_palette,
            base_color_hex = ?base_color_hex,
            brightness_ceiling = ?brightness_ceiling,
            "updated running Hue audio sync profile"
        );
        Ok(())
    }
}

async fn run_stream_loop(
    mut stream: EntertainmentStreamSession,
    area: EntertainmentArea,
    feature_rx: Receiver<AudioFeatures>,
    mut stop_rx: oneshot::Receiver<()>,
    profile: Arc<RwLock<StreamProfile>>,
    preview: Arc<RwLock<Option<AudioSyncPreview>>>,
) -> Result<(), HueError> {
    let mut latest = AudioFeatures::default();
    let mut window_peak = AudioFeatures::default();
    let mut smoothed = AudioFeatures::default();
    let mut smoothed_onset = 0.0_f32;
    let mut display_intensity = 0.0_f32;
    let mut accent_envelope = 0.0_f32;
    let mut steady_agc = AdaptiveGain::new();
    let mut transient_agc = AdaptiveGain::new();
    let mut pulse_agc = AdaptiveGain::new();
    let mut level_range = DynamicRange::new(0.01, 0.18);
    let mut bass_range = DynamicRange::new(0.01, 0.20);
    let mut mid_range = DynamicRange::new(0.01, 0.20);
    let mut attack_range = DynamicRange::new(0.01, 0.22);
    let mut treble_range = DynamicRange::new(0.01, 0.20);
    let mut onset_range = DynamicRange::new(0.01, 0.26);
    let mut pulse_range = DynamicRange::new(0.01, 0.24);
    let mut saw_audio = false;
    let mut frame_index = 0_u32;
    debug!(area_id = %area.id, channel_count = area.channels.len(), "audio sync stream loop running");

    loop {
        let tick_interval = {
            let profile = profile.read().map_err(|_| {
                HueError::EntertainmentStream("audio sync profile lock poisoned".to_string())
            })?;
            profile.tick_interval()
        };
        tokio::select! {
            _ = &mut stop_rx => {
                let blackout = empty_rgb_frame(&area);
                let _ = stream.write_rgb_frame(&blackout).await;
                return Ok(());
            }
            _ = time::sleep(tick_interval) => {
                let profile = *profile.read().map_err(|_| {
                    HueError::EntertainmentStream("audio sync profile lock poisoned".to_string())
                })?;
                let mut saw_new_features = false;
                while let Ok(features) = feature_rx.try_recv() {
                    latest = features;
                    if !saw_new_features {
                        window_peak = features;
                        saw_new_features = true;
                    } else {
                        window_peak.level = window_peak.level.max(features.level);
                        window_peak.bass = window_peak.bass.max(features.bass);
                        window_peak.mid = window_peak.mid.max(features.mid);
                        window_peak.attack = window_peak.attack.max(features.attack);
                        window_peak.treble = window_peak.treble.max(features.treble);
                        window_peak.onset = window_peak.onset.max(features.onset);
                    }
                    if features.level > 0.015
                        || features.bass > 0.02
                        || features.mid > 0.02
                        || features.attack > 0.02
                        || features.treble > 0.02
                    {
                        saw_audio = true;
                    }
                }

                let channels = if !saw_audio {
                    if frame_index == 0 {
                        debug!("no audio features received yet; holding current profile frame");
                    }
                    if let Ok(mut preview) = preview.write() {
                        let (red, green, blue) = if profile.lock_base_hue {
                            profile.base_color
                        } else {
                            profile.mid_color
                        };
                        *preview = Some(AudioSyncPreview {
                            entertainment_area_id: area.id.clone(),
                            red: (red * 255.0).round().clamp(0.0, 255.0) as u8,
                            green: (green * 255.0).round().clamp(0.0, 255.0) as u8,
                            blue: (blue * 255.0).round().clamp(0.0, 255.0) as u8,
                            intensity: profile.idle_intensity.clamp(0.0, 1.0),
                        });
                    }
                    profile.hold_channels(&area)
                } else {
                    let previous_smoothed_bass = smoothed.bass;
                    smoothed.level = smooth(smoothed.level, latest.level, profile.level_smoothing);
                    smoothed.bass = smooth(smoothed.bass, latest.bass, profile.band_smoothing);
                    smoothed.mid = smooth(smoothed.mid, latest.mid, profile.band_smoothing);
                    smoothed.attack =
                        smooth(smoothed.attack, latest.attack, profile.attack_smoothing);
                    smoothed.treble =
                        smooth(smoothed.treble, latest.treble, profile.treble_smoothing);
                    smoothed_onset =
                        smooth(smoothed_onset, latest.onset, profile.onset_smoothing);

                    let level_raw = amplify(smoothed.level, profile.level_gain);
                    let bass_raw = amplify(smoothed.bass, profile.bass_gain);
                    let mid_raw = amplify(smoothed.mid, profile.mid_gain);
                    let attack_raw = amplify(smoothed.attack, profile.attack_gain);
                    let treble_raw = amplify(smoothed.treble, profile.treble_gain);
                    let onset_raw = amplify(smoothed_onset.max(window_peak.onset), profile.onset_gain);
                    let bass_pulse_raw = amplify(
                        (window_peak.bass.max(latest.bass) - previous_smoothed_bass).max(0.0),
                        profile.bass_pulse_gain,
                    );

                    let floor_attack = 0.014_f32;
                    let floor_release = 0.0024_f32;
                    let peak_attack = 0.42_f32;
                    let peak_release = 0.030_f32;

                    let level = level_range.normalize(
                        level_raw,
                        floor_attack,
                        floor_release,
                        peak_attack,
                        peak_release,
                    );
                    let bass = shape_response(
                        bass_range.normalize(
                            bass_raw.max(window_peak.bass),
                            floor_attack,
                            floor_release,
                            peak_attack,
                            peak_release,
                        ),
                        profile.bass_power,
                    );
                    let bass_pulse = shape_response(
                        pulse_range.normalize(
                            bass_pulse_raw,
                            floor_attack,
                            floor_release,
                            peak_attack,
                            peak_release,
                        ),
                        profile.bass_pulse_power,
                    );
                    let mid = mid_range.normalize(
                        mid_raw.max(window_peak.mid * 0.7),
                        floor_attack,
                        floor_release,
                        peak_attack,
                        peak_release,
                    );
                    let attack = shape_response(
                        attack_range.normalize(
                            attack_raw.max(window_peak.attack),
                            floor_attack,
                            floor_release,
                            peak_attack,
                            peak_release,
                        ),
                        profile.attack_power,
                    );
                    let treble = treble_range.normalize(
                        treble_raw.max(window_peak.treble),
                        floor_attack,
                        floor_release,
                        peak_attack,
                        peak_release,
                    );
                    let onset = onset_range.normalize(
                        onset_raw,
                        floor_attack,
                        floor_release,
                        peak_attack,
                        peak_release,
                    );

                    let (red, green, blue) = if profile.lock_base_hue {
                        profile.base_color
                    } else {
                        let low_driver = (bass * 0.76 + bass_pulse * 0.24).clamp(0.0, 1.0);
                        let mid_driver = (mid * 0.72 + attack * 0.28).clamp(0.0, 1.0);
                        let high_driver = (treble * 0.72 + onset * 0.28).clamp(0.0, 1.0);
                        let band_sum = (low_driver + mid_driver + high_driver).max(0.0001);
                        let low_weight = (0.08 + low_driver / band_sum * 0.92).clamp(0.0, 1.0);
                        let mid_weight = (0.08 + mid_driver / band_sum * 0.92).clamp(0.0, 1.0);
                        let high_weight =
                            (0.08 + high_driver / band_sum * 0.92).clamp(0.0, 1.0);

                        let mut red = (profile.low_color.0 * low_weight
                            + profile.mid_color.0 * mid_weight
                            + profile.high_color.0 * high_weight)
                            .clamp(0.0, 1.0);
                        let mut green = (profile.low_color.1 * low_weight
                            + profile.mid_color.1 * mid_weight
                            + profile.high_color.1 * high_weight)
                            .clamp(0.0, 1.0);
                        let mut blue = (profile.low_color.2 * low_weight
                            + profile.mid_color.2 * mid_weight
                            + profile.high_color.2 * high_weight)
                            .clamp(0.0, 1.0);

                        let max_channel = red.max(green).max(blue);
                        if max_channel > f32::EPSILON {
                            red /= max_channel;
                            green /= max_channel;
                            blue /= max_channel;
                        }

                        (red, green, blue)
                    };

                    let steady_driver_raw = (
                        level * profile.level_intensity_weight
                        + bass * profile.bass_intensity_weight
                        + mid * profile.mid_intensity_weight
                        + treble * profile.treble_intensity_weight
                    ).clamp(0.0, 1.0);
                    let transient_driver_raw = (
                        bass_pulse * profile.bass_pulse_intensity_weight
                        + attack * profile.attack_intensity_weight
                        + onset * profile.onset_intensity_weight
                    ).clamp(0.0, 1.0);
                    // Main pulse should follow groove energy: bass body + mid-band punch,
                    // with snare/attack and bass transients adding the sharp accents.
                    let main_pulse_driver_raw = (
                        bass * 0.34
                        + mid * 0.28
                        + bass_pulse * 0.24
                        + attack * 0.12
                        + onset * 0.02
                    ).clamp(0.0, 1.0);

                    let steady_driver = steady_agc.process(
                        shape_response(steady_driver_raw, profile.steady_driver_power),
                        profile.steady_agc_target,
                        profile.agc_kp,
                        profile.agc_ki,
                        profile.agc_min_gain,
                        profile.agc_max_gain,
                    );
                    let transient_driver = transient_agc.process(
                        shape_response(transient_driver_raw, profile.transient_driver_power),
                        profile.transient_agc_target,
                        profile.agc_kp * 1.15,
                        profile.agc_ki * 1.20,
                        profile.agc_min_gain,
                        profile.agc_max_gain * 1.12,
                    );
                    let main_pulse_driver = pulse_agc.process(
                        fast_top_out(main_pulse_driver_raw, profile.main_pulse_top_out_power),
                        profile.pulse_agc_target,
                        profile.agc_kp * 1.35,
                        profile.agc_ki * 1.45,
                        profile.agc_min_gain,
                        profile.agc_max_gain * 1.24,
                    );

                    let target_intensity = (profile.intensity_floor
                        + steady_driver.powf(profile.level_power) * profile.intensity_range)
                        .clamp(0.0, profile.intensity_ceiling);
                    let intensity_smoothing = if target_intensity >= display_intensity {
                        profile.intensity_attack_smoothing
                    } else {
                        profile.intensity_release_smoothing
                    };
                    display_intensity = smooth(display_intensity, target_intensity, intensity_smoothing);
                    accent_envelope = (accent_envelope * profile.accent_decay)
                        .max(shape_response(main_pulse_driver, 0.52));
                    let flash_boost = accent_envelope
                        * profile.flash_gain
                        * profile.onset_flash_range
                        * (0.56 + 0.44 * transient_driver);
                    let motion_lift =
                        (steady_driver * 0.10 + transient_driver * 0.18 + main_pulse_driver * 0.22)
                        .min(profile.intensity_ceiling * 0.35);

                    let intensity = (display_intensity + motion_lift + flash_boost)
                        .clamp(0.0, profile.intensity_ceiling);

                    if frame_index % 25 == 0 {
                        trace!(
                            frame_index,
                            level,
                            bass,
                            bass_pulse,
                            mid,
                            attack,
                            treble,
                            onset,
                            steady_driver,
                            transient_driver,
                            main_pulse_driver,
                            motion_lift,
                            display_intensity,
                            accent_envelope,
                            flash_boost,
                            intensity,
                            "audio sync rendered frame"
                        );
                    }

                    if let Ok(mut preview) = preview.write() {
                        *preview = Some(AudioSyncPreview {
                            entertainment_area_id: area.id.clone(),
                            red: (red * 255.0).round().clamp(0.0, 255.0) as u8,
                            green: (green * 255.0).round().clamp(0.0, 255.0) as u8,
                            blue: (blue * 255.0).round().clamp(0.0, 255.0) as u8,
                            intensity: intensity.clamp(0.0, 1.0),
                        });
                    }

                    build_rgb_channels(&area.channels, red, green, blue, intensity)
                };
                if frame_index % 25 == 0 {
                    trace!(
                        frame_index,
                        saw_audio,
                        level = latest.level,
                        bass = latest.bass,
                        mid = latest.mid,
                        attack = latest.attack,
                        treble = latest.treble,
                        onset = latest.onset,
                        "audio sync feature frame"
                    );
                }
                stream.write_rgb_frame(&channels).await?;
                frame_index = frame_index.wrapping_add(1);
            }
        }
    }
}

async fn capture_restore_snapshot(
    client: &HueBridgeClient,
    area: &EntertainmentArea,
) -> Result<Vec<LightRestoreState>, HueError> {
    let area_light_ids = area
        .light_ids
        .iter()
        .map(|light_id| light_id.as_str())
        .collect::<HashSet<_>>();
    if area_light_ids.is_empty() {
        return Ok(Vec::new());
    }

    let lights = client.list_lights().await?;
    let snapshot = lights
        .iter()
        .filter(|light| area_light_ids.contains(light.id.as_str()))
        .map(|light| LightRestoreState {
            light_id: light.id.clone(),
            state: restore_state_from_light(light),
        })
        .collect::<Vec<_>>();

    Ok(snapshot)
}

async fn restore_snapshot(
    client: &HueBridgeClient,
    snapshot: &[LightRestoreState],
) -> Result<(), HueError> {
    let mut first_error: Option<HueError> = None;
    for entry in snapshot {
        if let Err(error) = client.set_light_state(&entry.light_id, &entry.state).await {
            if first_error.is_none() {
                first_error = Some(error);
            }
        }
    }

    if let Some(error) = first_error {
        return Err(error);
    }
    Ok(())
}

fn restore_state_from_light(light: &Light) -> LightStateUpdate {
    let is_on = light.is_on.unwrap_or(false);
    if is_on {
        LightStateUpdate {
            on: Some(true),
            brightness: light.brightness,
            saturation: light.saturation,
            hue: light.hue,
            transition_time: Some(4),
        }
    } else {
        LightStateUpdate {
            on: Some(false),
            brightness: None,
            saturation: None,
            hue: None,
            transition_time: Some(4),
        }
    }
}

fn smooth(current: f32, target: f32, factor: f32) -> f32 {
    current + (target - current) * factor
}

fn amplify(value: f32, gain: f32) -> f32 {
    (value * gain).clamp(0.0, 1.0)
}

fn shape_response(value: f32, power: f32) -> f32 {
    value.clamp(0.0, 1.0).powf(power)
}

fn fast_top_out(value: f32, power: f32) -> f32 {
    1.0 - (1.0 - value.clamp(0.0, 1.0)).powf(power.max(0.05))
}

#[derive(Clone, Copy)]
struct DynamicRange {
    floor: f32,
    peak: f32,
}

impl DynamicRange {
    fn new(floor: f32, peak: f32) -> Self {
        Self {
            floor: floor.clamp(0.0, 1.0),
            peak: peak.clamp(0.01, 1.0),
        }
    }

    fn normalize(
        &mut self,
        value: f32,
        floor_attack: f32,
        floor_release: f32,
        peak_attack: f32,
        peak_release: f32,
    ) -> f32 {
        let input = value.clamp(0.0, 1.0);
        if input < self.floor {
            self.floor = smooth(self.floor, input, floor_attack);
        } else {
            self.floor = smooth(self.floor, input, floor_release);
        }

        let minimum_peak = (self.floor + 0.08).clamp(0.08, 1.0);
        if input > self.peak {
            self.peak = smooth(self.peak, input, peak_attack);
        } else {
            self.peak = smooth(self.peak, input.max(minimum_peak), peak_release);
        }
        self.peak = self.peak.max(minimum_peak);

        ((input - self.floor) / (self.peak - self.floor + 0.0001)).clamp(0.0, 1.0)
    }
}

#[derive(Clone, Copy)]
struct AdaptiveGain {
    gain: f32,
    integral: f32,
}

impl AdaptiveGain {
    fn new() -> Self {
        Self {
            gain: 1.0,
            integral: 0.0,
        }
    }

    fn process(
        &mut self,
        input: f32,
        target: f32,
        kp: f32,
        ki: f32,
        min_gain: f32,
        max_gain: f32,
    ) -> f32 {
        let value = input.clamp(0.0, 1.0);
        let output = (value * self.gain).clamp(0.0, 1.0);
        let error = (target - output).clamp(-1.0, 1.0);
        self.integral = (self.integral + error * ki).clamp(-0.9, 0.9);
        self.gain = (self.gain + error * kp + self.integral * 0.18).clamp(min_gain, max_gain);
        (value * self.gain).clamp(0.0, 1.0)
    }
}

#[derive(Clone, Copy)]
struct StreamProfile {
    base_color: (f32, f32, f32),
    low_color: (f32, f32, f32),
    mid_color: (f32, f32, f32),
    high_color: (f32, f32, f32),
    lock_base_hue: bool,
    intensity_floor: f32,
    intensity_range: f32,
    intensity_ceiling: f32,
    idle_intensity: f32,
    level_power: f32,
    bass_power: f32,
    bass_pulse_power: f32,
    attack_power: f32,
    level_intensity_weight: f32,
    bass_intensity_weight: f32,
    bass_pulse_intensity_weight: f32,
    mid_intensity_weight: f32,
    attack_intensity_weight: f32,
    treble_intensity_weight: f32,
    onset_intensity_weight: f32,
    onset_flash_range: f32,
    steady_agc_target: f32,
    transient_agc_target: f32,
    pulse_agc_target: f32,
    agc_kp: f32,
    agc_ki: f32,
    agc_min_gain: f32,
    agc_max_gain: f32,
    steady_driver_power: f32,
    transient_driver_power: f32,
    main_pulse_top_out_power: f32,
    flash_gain: f32,
    level_gain: f32,
    bass_gain: f32,
    bass_pulse_gain: f32,
    mid_gain: f32,
    attack_gain: f32,
    treble_gain: f32,
    onset_gain: f32,
    level_smoothing: f32,
    band_smoothing: f32,
    attack_smoothing: f32,
    treble_smoothing: f32,
    onset_smoothing: f32,
    intensity_attack_smoothing: f32,
    intensity_release_smoothing: f32,
    accent_decay: f32,
    tick_ms: u64,
}

impl StreamProfile {
    fn new(
        speed_mode: AudioSyncSpeedMode,
        color_palette: AudioSyncColorPalette,
        base_color_hex: Option<String>,
        brightness_ceiling: Option<u8>,
    ) -> Self {
        let (base_color, low_color, mid_color, high_color, lock_base_hue) =
            palette_colors(color_palette, base_color_hex.as_deref());
        let (
            tick_ms,
            level_smoothing,
            band_smoothing,
            attack_smoothing,
            treble_smoothing,
            onset_smoothing,
            intensity_attack_smoothing,
            intensity_release_smoothing,
            accent_decay,
            level_gain,
            bass_gain,
            bass_pulse_gain,
            mid_gain,
            attack_gain,
            treble_gain,
            onset_gain,
            intensity_floor_factor,
            min_intensity_floor,
            min_intensity_range,
            level_power,
            bass_power,
            bass_pulse_power,
            attack_power,
            level_intensity_weight,
            bass_intensity_weight,
            bass_pulse_intensity_weight,
            mid_intensity_weight,
            attack_intensity_weight,
            treble_intensity_weight,
            onset_intensity_weight,
            onset_flash_range,
        ) = match speed_mode {
            AudioSyncSpeedMode::Slow => (
                52, 0.14, 0.16, 0.20, 0.14, 0.18, 0.30, 0.17, 0.72, 1.2, 1.45, 2.8, 0.78, 1.45,
                0.36, 2.1, 0.24, 0.06, 0.56, 1.05, 0.84, 1.14, 0.95, 0.26, 0.20, 0.40, 0.07, 0.19,
                0.03, 0.14, 0.10,
            ),
            AudioSyncSpeedMode::Medium => (
                28, 0.26, 0.30, 0.34, 0.24, 0.24, 0.32, 0.12, 0.60, 1.55, 1.95, 5.0, 0.85, 2.35,
                0.58, 3.1, 0.20, 0.06, 0.76, 1.20, 0.64, 0.96, 0.82, 0.10, 0.26, 0.62, 0.09, 0.28,
                0.05, 0.24, 0.18,
            ),
            AudioSyncSpeedMode::High => (
                12, 0.86, 0.90, 0.38, 0.82, 0.46, 0.20, 0.08, 0.54, 1.8, 2.1, 5.8, 0.92, 2.55,
                0.66, 3.4, 0.24, 0.08, 0.70, 0.92, 0.56, 0.90, 0.76, 0.11, 0.26, 0.66, 0.07, 0.30,
                0.03, 0.18, 0.24,
            ),
        };
        let (
            steady_agc_target,
            transient_agc_target,
            pulse_agc_target,
            agc_kp,
            agc_ki,
            agc_min_gain,
            agc_max_gain,
            steady_driver_power,
            transient_driver_power,
            main_pulse_top_out_power,
            flash_gain,
        ) = match speed_mode {
            AudioSyncSpeedMode::Slow => (
                0.72, 0.84, 0.90, 0.14, 0.016, 0.70, 2.6, 0.92, 0.72, 1.35, 0.76,
            ),
            AudioSyncSpeedMode::Medium => (
                0.70, 0.86, 0.94, 0.18, 0.024, 0.64, 3.0, 0.86, 0.66, 1.60, 0.88,
            ),
            AudioSyncSpeedMode::High => (
                0.66, 0.88, 0.96, 0.24, 0.036, 0.56, 3.4, 0.78, 0.58, 1.95, 0.98,
            ),
        };
        let brightness_ratio = brightness_ceiling.unwrap_or(100).clamp(1, 100) as f32 / 100.0;
        let idle_intensity = brightness_ratio.clamp(0.02, 1.0);
        let intensity_ceiling = brightness_ratio.clamp(0.02, 1.0);
        let min_intensity_floor = min_intensity_floor as f32;
        let min_intensity_range = min_intensity_range as f32;
        let intensity_floor = (intensity_ceiling * intensity_floor_factor).clamp(
            min_intensity_floor.min(intensity_ceiling * 0.4),
            intensity_ceiling * 0.98,
        );
        let intensity_range = (intensity_ceiling - intensity_floor)
            .max((intensity_ceiling * min_intensity_range).min(intensity_ceiling));

        Self {
            base_color,
            low_color,
            mid_color,
            high_color,
            lock_base_hue,
            intensity_floor,
            intensity_range,
            intensity_ceiling,
            idle_intensity,
            level_power,
            bass_power,
            bass_pulse_power,
            attack_power,
            level_intensity_weight,
            bass_intensity_weight,
            bass_pulse_intensity_weight,
            mid_intensity_weight,
            attack_intensity_weight,
            treble_intensity_weight,
            onset_intensity_weight,
            onset_flash_range,
            steady_agc_target,
            transient_agc_target,
            pulse_agc_target,
            agc_kp,
            agc_ki,
            agc_min_gain,
            agc_max_gain,
            steady_driver_power,
            transient_driver_power,
            main_pulse_top_out_power,
            flash_gain,
            level_gain,
            bass_gain,
            bass_pulse_gain,
            mid_gain,
            attack_gain,
            treble_gain,
            onset_gain,
            level_smoothing,
            band_smoothing,
            attack_smoothing,
            treble_smoothing,
            onset_smoothing,
            intensity_attack_smoothing,
            intensity_release_smoothing,
            accent_decay,
            tick_ms,
        }
    }

    fn tick_interval(&self) -> Duration {
        Duration::from_millis(self.tick_ms)
    }

    fn hold_channels(
        &self,
        area: &EntertainmentArea,
    ) -> Vec<crate::hue::entertainment::EntertainmentChannelColor> {
        let (red, green, blue) = if self.lock_base_hue {
            self.base_color
        } else {
            self.mid_color
        };
        build_rgb_channels(&area.channels, red, green, blue, self.idle_intensity)
    }
}

fn palette_colors(
    palette: AudioSyncColorPalette,
    base_color_hex: Option<&str>,
) -> (
    (f32, f32, f32),
    (f32, f32, f32),
    (f32, f32, f32),
    (f32, f32, f32),
    bool,
) {
    match palette {
        AudioSyncColorPalette::CurrentRoom => {
            let base =
                normalize_color(parse_hex_color(base_color_hex).unwrap_or((0.92, 0.92, 0.92)));
            let low = mix_color(base, (0.0, 0.0, 0.0), 0.10);
            let mid = base;
            let high = mix_color(base, (1.0, 1.0, 1.0), 0.10);
            (base, low, mid, high, true)
        }
        AudioSyncColorPalette::Sunset => (
            (1.0, 0.66, 0.28),
            (0.98, 0.48, 0.18),
            (1.0, 0.66, 0.28),
            (1.0, 0.86, 0.56),
            false,
        ),
        AudioSyncColorPalette::Aurora => (
            (0.28, 0.74, 1.0),
            (0.22, 0.95, 0.38),
            (0.28, 0.74, 1.0),
            (0.72, 0.34, 1.0),
            false,
        ),
        AudioSyncColorPalette::Ocean => (
            (0.14, 0.82, 0.92),
            (0.0, 0.58, 0.76),
            (0.14, 0.82, 0.92),
            (0.4, 0.58, 1.0),
            false,
        ),
        AudioSyncColorPalette::Rose => (
            (1.0, 0.46, 0.72),
            (0.96, 0.32, 0.44),
            (1.0, 0.46, 0.72),
            (0.8, 0.52, 1.0),
            false,
        ),
        AudioSyncColorPalette::Mono => (
            (0.9, 0.9, 0.9),
            (0.72, 0.72, 0.72),
            (0.9, 0.9, 0.9),
            (1.0, 1.0, 1.0),
            false,
        ),
        AudioSyncColorPalette::NeonPulse => (
            (0.70, 0.30, 1.0),
            (0.08, 0.22, 1.0),
            (1.0, 0.20, 0.84),
            (0.22, 1.0, 0.88),
            false,
        ),
        AudioSyncColorPalette::Prism => (
            (0.92, 0.94, 1.0),
            (1.0, 0.28, 0.16),
            (0.22, 0.96, 0.34),
            (0.26, 0.42, 1.0),
            false,
        ),
        AudioSyncColorPalette::VocalGlow => (
            (1.0, 0.86, 0.66),
            (0.12, 0.20, 0.72),
            (1.0, 0.70, 0.28),
            (0.74, 0.90, 1.0),
            false,
        ),
        AudioSyncColorPalette::FireIce => (
            (0.98, 0.86, 0.78),
            (1.0, 0.30, 0.14),
            (0.96, 0.64, 0.22),
            (0.16, 0.84, 1.0),
            false,
        ),
    }
}

fn normalize_color((red, green, blue): (f32, f32, f32)) -> (f32, f32, f32) {
    let max_channel = red.max(green).max(blue);
    if max_channel <= f32::EPSILON {
        (1.0, 1.0, 1.0)
    } else {
        (red / max_channel, green / max_channel, blue / max_channel)
    }
}

fn parse_hex_color(value: Option<&str>) -> Option<(f32, f32, f32)> {
    let value = value?.trim();

    if let Some(hex) = value.strip_prefix('#') {
        if hex.len() != 6 {
            return None;
        }

        let red = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
        let green = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
        let blue = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
        return Some((red, green, blue));
    }

    let rgb = value
        .strip_prefix("rgb(")
        .and_then(|value| value.strip_suffix(')'))?;
    let parts = rgb
        .split(|character: char| character == ',' || character.is_whitespace())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if parts.len() != 3 {
        return None;
    }

    let red = parts[0].parse::<u8>().ok()? as f32 / 255.0;
    let green = parts[1].parse::<u8>().ok()? as f32 / 255.0;
    let blue = parts[2].parse::<u8>().ok()? as f32 / 255.0;
    Some((red, green, blue))
}

fn mix_color(base: (f32, f32, f32), tint: (f32, f32, f32), amount: f32) -> (f32, f32, f32) {
    (
        base.0 + (tint.0 - base.0) * amount,
        base.1 + (tint.1 - base.1) * amount,
        base.2 + (tint.2 - base.2) * amount,
    )
}

#[cfg(test)]
mod tests {
    use super::{AdaptiveGain, StreamProfile, fast_top_out, palette_colors, parse_hex_color};
    use crate::hue::models::{AudioSyncColorPalette, AudioSyncSpeedMode};

    #[test]
    fn current_room_palette_uses_base_color() {
        let (_, _, mid, _, _) = palette_colors(AudioSyncColorPalette::CurrentRoom, Some("#8040ff"));

        assert!((mid.0 - 0.501).abs() < 0.01);
        assert!((mid.1 - 0.250).abs() < 0.01);
        assert!((mid.2 - 1.0).abs() < 0.01);
    }

    #[test]
    fn extended_palettes_use_dynamic_band_colors() {
        let palettes = [
            AudioSyncColorPalette::NeonPulse,
            AudioSyncColorPalette::Prism,
            AudioSyncColorPalette::VocalGlow,
            AudioSyncColorPalette::FireIce,
        ];

        for palette in palettes {
            let (base, low, mid, high, lock_base_hue) = palette_colors(palette, None);
            assert!(!lock_base_hue);
            assert_ne!(low, mid);
            assert_ne!(mid, high);
            assert_ne!(base, low);
        }
    }

    #[test]
    fn stream_profile_respects_brightness_ceiling() {
        let profile = StreamProfile::new(
            AudioSyncSpeedMode::Medium,
            AudioSyncColorPalette::Sunset,
            None,
            Some(35),
        );

        assert!((profile.intensity_ceiling - 0.35).abs() < f32::EPSILON);
        assert!(profile.intensity_floor < profile.intensity_ceiling);
    }

    #[test]
    fn medium_profile_keeps_more_transient_headroom_than_high() {
        let medium = StreamProfile::new(
            AudioSyncSpeedMode::Medium,
            AudioSyncColorPalette::CurrentRoom,
            Some("#ff3344".to_string()),
            Some(100),
        );
        let high = StreamProfile::new(
            AudioSyncSpeedMode::High,
            AudioSyncColorPalette::CurrentRoom,
            Some("#ff3344".to_string()),
            Some(100),
        );

        assert!(medium.level_intensity_weight < high.level_intensity_weight);
        assert!(medium.onset_intensity_weight > high.onset_intensity_weight);
        assert!(medium.bass_intensity_weight > medium.level_intensity_weight);
        assert!(medium.onset_flash_range < high.onset_flash_range);
    }

    #[test]
    fn sync_profiles_prioritize_bass_over_mid_and_treble() {
        let medium = StreamProfile::new(
            AudioSyncSpeedMode::Medium,
            AudioSyncColorPalette::CurrentRoom,
            Some("#ff3344".to_string()),
            Some(100),
        );
        let high = StreamProfile::new(
            AudioSyncSpeedMode::High,
            AudioSyncColorPalette::CurrentRoom,
            Some("#ff3344".to_string()),
            Some(100),
        );

        assert!(medium.bass_intensity_weight > medium.mid_intensity_weight);
        assert!(medium.bass_intensity_weight > medium.treble_intensity_weight);
        assert!(high.bass_intensity_weight > high.mid_intensity_weight);
        assert!(high.bass_intensity_weight > high.treble_intensity_weight);
        assert!(medium.bass_power < 1.0);
        assert!(high.bass_power < 1.0);
        assert!(medium.bass_pulse_intensity_weight > medium.level_intensity_weight);
        assert!(high.bass_pulse_intensity_weight > high.level_intensity_weight);
    }

    #[test]
    fn parses_hex_color() {
        assert_eq!(parse_hex_color(Some("#ff0000")), Some((1.0, 0.0, 0.0)));
    }

    #[test]
    fn fast_top_out_pushes_midrange_values_higher() {
        let plain = 0.5_f32;
        let boosted = fast_top_out(plain, 1.6);
        assert!(boosted > plain);
        assert!(boosted > 0.65);
        assert!(fast_top_out(1.0, 1.6) <= 1.0);
    }

    #[test]
    fn adaptive_gain_moves_signal_towards_target() {
        let mut agc = AdaptiveGain::new();
        let mut output = 0.0_f32;
        for _ in 0..32 {
            output = agc.process(0.2, 0.75, 0.20, 0.03, 0.5, 3.8);
        }
        assert!(output > 0.45);
    }
}
