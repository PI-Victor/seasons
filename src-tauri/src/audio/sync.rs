use crate::audio::analysis::AudioFeatures;
use crate::audio::capture::{start_sink_capture, AudioCaptureHandle};
use crate::hue::entertainment::{build_rgb_channels, empty_rgb_frame, EntertainmentStreamSession};
use crate::hue::error::HueError;
use crate::hue::models::AudioSyncStartResult;
use crate::hue::models::{AudioSyncColorPalette, AudioSyncSpeedMode};
use crate::hue::{BridgeConnection, EntertainmentArea, HueBridgeClient, HueBridgeConfig};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex, RwLock};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::{self, Duration};
use tracing::{debug, info, trace};

pub struct AudioSyncManager {
    running: Mutex<Option<RunningAudioSyncSession>>,
}

struct RunningAudioSyncSession {
    area_id: String,
    connection: BridgeConnection,
    stop_tx: oneshot::Sender<()>,
    stream_task: JoinHandle<()>,
    capture: AudioCaptureHandle,
    profile: Arc<RwLock<StreamProfile>>,
}

impl AudioSyncManager {
    pub fn new() -> Self {
        Self {
            running: Mutex::new(None),
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
        debug!("connecting DTLS entertainment stream");
        let mut stream = EntertainmentStreamSession::connect(&resolved_connection, &area).await?;
        let test_frame = build_rgb_channels(&area.channels, 1.0, 0.24, 0.78, 0.45);
        debug!("writing initial entertainment test frame");
        stream.write_rgb_frame(&test_frame).await?;

        let (capture, feature_rx) = start_sink_capture(pipewire_target_object.as_deref())?;
        let (stop_tx, stop_rx) = oneshot::channel();
        let task_area = area.clone();
        let stream_profile = Arc::new(RwLock::new(StreamProfile::new(
            speed_mode,
            color_palette,
            base_color_hex,
            brightness_ceiling,
        )));
        let task_profile = Arc::clone(&stream_profile);

        let stream_task = tokio::spawn(async move {
            let _ = run_stream_loop(stream, task_area, feature_rx, stop_rx, task_profile).await;
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
        client.stop_entertainment_area(&running.area_id).await?;
        info!("Hue audio sync stopped");
        Ok(())
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
) -> Result<(), HueError> {
    let mut latest = AudioFeatures::default();
    let mut smoothed = AudioFeatures::default();
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
                while let Ok(features) = feature_rx.try_recv() {
                    latest = features;
                    if features.level > 0.015
                        || features.bass > 0.02
                        || features.mid > 0.02
                        || features.treble > 0.02
                    {
                        saw_audio = true;
                    }
                }

                let channels = if !saw_audio && frame_index < 50 {
                    if frame_index == 0 {
                        debug!("no audio features received yet; sending startup pulse frames");
                    }
                    startup_pulse_channels(&area, frame_index)
                } else {
                    smoothed.level = smooth(smoothed.level, latest.level, profile.level_smoothing);
                    smoothed.bass = smooth(smoothed.bass, latest.bass, profile.band_smoothing);
                    smoothed.mid = smooth(smoothed.mid, latest.mid, profile.band_smoothing);
                    smoothed.treble =
                        smooth(smoothed.treble, latest.treble, profile.treble_smoothing);

                    let level = amplify(smoothed.level, profile.level_gain);
                    let bass = amplify(smoothed.bass, profile.bass_gain);
                    let mid = amplify(smoothed.mid, profile.mid_gain);
                    let treble = amplify(smoothed.treble, profile.treble_gain);

                    let (red, green, blue) = if profile.lock_base_hue {
                        profile.base_color
                    } else {
                        let low_weight = (0.18 + bass * 0.82).clamp(0.0, 1.0);
                        let mid_weight = (0.15 + mid * 0.85).clamp(0.0, 1.0);
                        let high_weight = (0.12 + treble * 0.88).clamp(0.0, 1.0);

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

                    let intensity_driver = (
                        level * profile.level_intensity_weight
                        + bass * profile.bass_intensity_weight
                        + mid * profile.mid_intensity_weight
                        + treble * profile.treble_intensity_weight
                    ).clamp(0.0, 1.0);

                    let intensity = (profile.intensity_floor
                        + intensity_driver.powf(profile.level_power) * profile.intensity_range)
                        .clamp(0.0, profile.intensity_ceiling);

                    build_rgb_channels(&area.channels, red, green, blue, intensity)
                };
                if frame_index % 25 == 0 {
                    trace!(
                        frame_index,
                        saw_audio,
                        level = latest.level,
                        bass = latest.bass,
                        mid = latest.mid,
                        treble = latest.treble,
                        "audio sync feature frame"
                    );
                }
                stream.write_rgb_frame(&channels).await?;
                frame_index = frame_index.wrapping_add(1);
            }
        }
    }
}

fn smooth(current: f32, target: f32, factor: f32) -> f32 {
    current + (target - current) * factor
}

fn amplify(value: f32, gain: f32) -> f32 {
    (value * gain).clamp(0.0, 1.0)
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
    level_power: f32,
    level_intensity_weight: f32,
    bass_intensity_weight: f32,
    mid_intensity_weight: f32,
    treble_intensity_weight: f32,
    level_gain: f32,
    bass_gain: f32,
    mid_gain: f32,
    treble_gain: f32,
    level_smoothing: f32,
    band_smoothing: f32,
    treble_smoothing: f32,
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
            treble_smoothing,
            level_gain,
            bass_gain,
            mid_gain,
            treble_gain,
            intensity_floor_factor,
            min_intensity_floor,
            min_intensity_range,
            level_power,
            level_intensity_weight,
            bass_intensity_weight,
            mid_intensity_weight,
            treble_intensity_weight,
        ) = match speed_mode {
            AudioSyncSpeedMode::Slow => {
                (60, 0.10, 0.12, 0.10, 12.0, 9.0, 8.5, 9.5, 0.62, 0.26, 0.28, 1.0, 0.70, 0.20, 0.07, 0.03)
            }
            AudioSyncSpeedMode::Medium => {
                (35, 0.22, 0.24, 0.20, 12.0, 9.0, 8.5, 9.5, 0.62, 0.26, 0.28, 1.0, 0.55, 0.30, 0.10, 0.05)
            }
            AudioSyncSpeedMode::High => {
                (14, 0.74, 0.76, 0.72, 20.0, 18.0, 13.0, 12.0, 0.20, 0.12, 0.62, 0.58, 0.46, 0.34, 0.12, 0.08)
            }
        };
        let min_intensity_floor = min_intensity_floor as f32;
        let min_intensity_range = min_intensity_range as f32;

        let brightness_ratio = brightness_ceiling.unwrap_or(100).clamp(1, 100) as f32 / 100.0;
        let intensity_ceiling =
            (0.04 + brightness_ratio.powf(1.55) * 0.96).clamp(0.04, 1.0);
        let intensity_floor = (intensity_ceiling * intensity_floor_factor)
            .clamp(min_intensity_floor.min(intensity_ceiling * 0.4), intensity_ceiling * 0.9);
        let intensity_range =
            (intensity_ceiling - intensity_floor).max((intensity_ceiling * min_intensity_range).min(intensity_ceiling));

        Self {
            base_color,
            low_color,
            mid_color,
            high_color,
            lock_base_hue,
            intensity_floor,
            intensity_range,
            intensity_ceiling,
            level_power,
            level_intensity_weight,
            bass_intensity_weight,
            mid_intensity_weight,
            treble_intensity_weight,
            level_gain,
            bass_gain,
            mid_gain,
            treble_gain,
            level_smoothing,
            band_smoothing,
            treble_smoothing,
            tick_ms,
        }
    }

    fn tick_interval(&self) -> Duration {
        Duration::from_millis(self.tick_ms)
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
            let base = normalize_color(parse_hex_color(base_color_hex).unwrap_or((0.92, 0.92, 0.92)));
            let low = mix_color(base, (0.0, 0.0, 0.0), 0.08);
            let mid = base;
            let high = mix_color(base, (1.0, 1.0, 1.0), 0.06);
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
    use super::{palette_colors, parse_hex_color, StreamProfile};
    use crate::hue::models::{AudioSyncColorPalette, AudioSyncSpeedMode};

    #[test]
    fn current_room_palette_uses_base_color() {
        let (_, mid, _) = palette_colors(AudioSyncColorPalette::CurrentRoom, Some("#8040ff"));

        assert!((mid.0 - 0.501).abs() < 0.01);
        assert!((mid.1 - 0.250).abs() < 0.01);
        assert!((mid.2 - 1.0).abs() < 0.01);
    }

    #[test]
    fn stream_profile_respects_brightness_ceiling() {
        let profile = StreamProfile::new(
            AudioSyncSpeedMode::Medium,
            AudioSyncColorPalette::Sunset,
            None,
            Some(35),
        );

        assert!(profile.intensity_ceiling <= 0.35 + f32::EPSILON);
        assert!(profile.intensity_floor < profile.intensity_ceiling);
    }

    #[test]
    fn parses_hex_color() {
        assert_eq!(parse_hex_color(Some("#ff0000")), Some((1.0, 0.0, 0.0)));
    }
}

fn startup_pulse_channels(
    area: &EntertainmentArea,
    frame_index: u32,
) -> Vec<crate::hue::entertainment::EntertainmentChannelColor> {
    let phase = (frame_index / 10) % 4;
    let (red, green, blue, intensity) = match phase {
        0 => (1.0, 0.16, 0.18, 0.95),
        1 => (0.18, 0.95, 0.25, 0.95),
        2 => (0.22, 0.28, 1.0, 0.95),
        _ => (1.0, 1.0, 1.0, 0.6),
    };

    build_rgb_channels(&area.channels, red, green, blue, intensity)
}
