#[cfg(any(target_os = "linux", target_os = "macos"))]
use crate::audio::analysis::AudioAnalyzer;
use crate::audio::analysis::AudioFeatures;
use crate::hue::error::HueError;
use crate::hue::models::PipeWireOutputTarget;
#[cfg(target_os = "linux")]
use pipewire as pw;
#[cfg(target_os = "linux")]
use pw::properties::properties;
#[cfg(target_os = "linux")]
use pw::registry::GlobalObject;
#[cfg(target_os = "linux")]
use pw::spa;
#[cfg(target_os = "linux")]
use pw::spa::param::format::{MediaSubtype, MediaType};
#[cfg(target_os = "linux")]
use pw::spa::param::format_utils;
#[cfg(target_os = "linux")]
use pw::spa::pod::Pod;
#[cfg(target_os = "linux")]
use pw::spa::utils::dict::DictRef;
#[cfg(target_os = "linux")]
use pw::types::ObjectType;
#[cfg(target_os = "macos")]
use screencapturekit::dispatch_queue::{DispatchQoS, DispatchQueue};
#[cfg(target_os = "macos")]
use screencapturekit::prelude::*;
#[cfg(target_os = "macos")]
use screencapturekit::AudioBufferList;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::sync::mpsc::{sync_channel, SyncSender};
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::sync::{Arc, Mutex};
#[cfg(any(target_os = "linux", target_os = "macos"))]
use tracing::{debug, info, warn};

#[cfg(target_os = "linux")]
struct CaptureUserData {
    format: spa::param::audio::AudioInfoRaw,
    analyzer: AudioAnalyzer,
    sender: SyncSender<AudioFeatures>,
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub struct AudioCaptureHandle {
    stop: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub struct AudioCaptureHandle;

#[cfg(any(target_os = "linux", target_os = "macos"))]
impl AudioCaptureHandle {
    pub fn stop(mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
impl AudioCaptureHandle {
    pub fn stop(self) {}
}

#[cfg(target_os = "linux")]
pub fn start_sink_capture(
    target_object: Option<&str>,
) -> Result<(AudioCaptureHandle, Receiver<AudioFeatures>), HueError> {
    let target_object = target_object.map(str::to_string);
    info!(target_object = ?target_object, "starting PipeWire audio capture");
    let (feature_tx, feature_rx) = sync_channel::<AudioFeatures>(8);
    let (startup_tx, startup_rx) = std::sync::mpsc::channel::<Result<(), String>>();
    let stop = Arc::new(AtomicBool::new(false));
    let thread_stop = Arc::clone(&stop);

    let thread = std::thread::spawn(move || {
        pw::init();
        let startup_error_tx = startup_tx.clone();
        if let Err(error) =
            run_linux_capture_loop(thread_stop, feature_tx, startup_tx, target_object)
        {
            warn!(%error, "PipeWire capture loop exited with an error");
            let _ = startup_error_tx.send(Err(error.to_string()));
        }
    });

    finish_capture_startup(stop, thread, startup_rx, feature_rx)
}

#[cfg(target_os = "macos")]
pub fn start_sink_capture(
    target_object: Option<&str>,
) -> Result<(AudioCaptureHandle, Receiver<AudioFeatures>), HueError> {
    let target_object = target_object.map(str::to_string);
    info!(target_object = ?target_object, "starting ScreenCaptureKit audio capture");
    let (feature_tx, feature_rx) = sync_channel::<AudioFeatures>(8);
    let (startup_tx, startup_rx) = std::sync::mpsc::channel::<Result<(), String>>();
    let stop = Arc::new(AtomicBool::new(false));
    let thread_stop = Arc::clone(&stop);

    let thread = std::thread::spawn(move || {
        let startup_error_tx = startup_tx.clone();
        if let Err(error) =
            run_macos_capture_loop(thread_stop, feature_tx, startup_tx, target_object)
        {
            warn!(%error, "ScreenCaptureKit capture loop exited with an error");
            let _ = startup_error_tx.send(Err(error.to_string()));
        }
    });

    finish_capture_startup(stop, thread, startup_rx, feature_rx)
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub fn start_sink_capture(
    target_object: Option<&str>,
) -> Result<(AudioCaptureHandle, Receiver<AudioFeatures>), HueError> {
    let _ = target_object;
    Err(HueError::AudioCapture(
        "audio sync is currently only available on Linux and macOS".to_string(),
    ))
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn finish_capture_startup(
    stop: Arc<AtomicBool>,
    thread: std::thread::JoinHandle<()>,
    startup_rx: std::sync::mpsc::Receiver<Result<(), String>>,
    feature_rx: Receiver<AudioFeatures>,
) -> Result<(AudioCaptureHandle, Receiver<AudioFeatures>), HueError> {
    match startup_rx.recv() {
        Ok(Ok(())) => Ok((
            AudioCaptureHandle {
                stop,
                thread: Some(thread),
            },
            feature_rx,
        )),
        Ok(Err(error)) => {
            let _ = thread.join();
            Err(HueError::AudioCapture(error))
        }
        Err(error) => {
            let _ = thread.join();
            Err(HueError::AudioCapture(format!(
                "failed to receive capture startup status: {error}"
            )))
        }
    }
}

#[cfg(target_os = "linux")]
pub fn list_output_targets() -> Result<Vec<PipeWireOutputTarget>, HueError> {
    pw::init();
    debug!("listing PipeWire audio sink targets");

    let mainloop = pw::main_loop::MainLoopBox::new(None)
        .map_err(|error| HueError::AudioCapture(error.to_string()))?;
    let context = pw::context::ContextBox::new(&mainloop.loop_(), None)
        .map_err(|error| HueError::AudioCapture(error.to_string()))?;
    let core = context
        .connect(None)
        .map_err(|error| HueError::AudioCapture(error.to_string()))?;
    let registry = core
        .get_registry()
        .map_err(|error| HueError::AudioCapture(error.to_string()))?;

    let targets = Arc::new(Mutex::new(Vec::<PipeWireOutputTarget>::new()));
    let targets_for_listener = Arc::clone(&targets);

    let _listener = registry
        .add_listener_local()
        .global(move |global| {
            collect_linux_output_target(global, &targets_for_listener);
        })
        .register();

    for _ in 0..6 {
        let _ = mainloop
            .loop_()
            .iterate(std::time::Duration::from_millis(80).into());
    }

    let mut targets = targets
        .lock()
        .map_err(|_| HueError::AudioCapture("pipewire target list lock poisoned".to_string()))?
        .clone();
    targets.sort_by(|left, right| {
        left.description
            .cmp(&right.description)
            .then(left.target_object.cmp(&right.target_object))
    });
    targets.dedup_by(|left, right| left.target_object == right.target_object);
    debug!(
        count = targets.len(),
        "discovered PipeWire audio sink targets"
    );
    Ok(targets)
}

#[cfg(target_os = "macos")]
pub fn list_output_targets() -> Result<Vec<PipeWireOutputTarget>, HueError> {
    let content = SCShareableContent::get()
        .map_err(|error| HueError::AudioCapture(format!("failed to list displays: {error}")))?;
    let mut targets = content
        .displays()
        .into_iter()
        .map(|display| {
            let display_id = display.display_id();
            PipeWireOutputTarget {
                target_object: display_id.to_string(),
                name: format!("Display {display_id}"),
                description: format!(
                    "Display {display_id} system audio ({}x{})",
                    display.width(),
                    display.height()
                ),
                media_class: "Display/SystemAudio".to_string(),
            }
        })
        .collect::<Vec<_>>();
    targets.sort_by(|left, right| left.target_object.cmp(&right.target_object));
    Ok(targets)
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub fn list_output_targets() -> Result<Vec<PipeWireOutputTarget>, HueError> {
    Ok(Vec::new())
}

#[cfg(target_os = "linux")]
fn run_linux_capture_loop(
    stop: Arc<AtomicBool>,
    sender: SyncSender<AudioFeatures>,
    startup_tx: std::sync::mpsc::Sender<Result<(), String>>,
    target_object: Option<String>,
) -> Result<(), HueError> {
    let mainloop = pw::main_loop::MainLoopBox::new(None)
        .map_err(|error| HueError::AudioCapture(error.to_string()))?;
    let context = pw::context::ContextBox::new(&mainloop.loop_(), None)
        .map_err(|error| HueError::AudioCapture(error.to_string()))?;
    let core = context
        .connect(None)
        .map_err(|error| HueError::AudioCapture(error.to_string()))?;

    let mut props = properties! {
        *pw::keys::MEDIA_TYPE => "Audio",
        *pw::keys::MEDIA_CATEGORY => "Capture",
        *pw::keys::MEDIA_ROLE => "Music",
    };
    props.insert(*pw::keys::STREAM_CAPTURE_SINK, "true");
    if let Some(target_object) = target_object
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        props.insert("target.object", target_object);
        info!(
            target_object,
            "binding PipeWire capture to explicit output target"
        );
    } else {
        info!("binding PipeWire capture to default output target");
    }

    let stream = pw::stream::StreamBox::new(&core, "seasons-audio-sync", props)
        .map_err(|error| HueError::AudioCapture(error.to_string()))?;
    let data = CaptureUserData {
        format: Default::default(),
        analyzer: AudioAnalyzer::new(),
        sender,
    };

    let _listener = stream
        .add_local_listener_with_user_data(data)
        .param_changed(|_, user_data, id, param| {
            let Some(param) = param else {
                return;
            };
            if id != spa::param::ParamType::Format.as_raw() {
                return;
            }

            let (media_type, media_subtype) = match format_utils::parse_format(param) {
                Ok(value) => value,
                Err(_) => return,
            };

            if media_type != MediaType::Audio || media_subtype != MediaSubtype::Raw {
                return;
            }

            let _ = user_data.format.parse(param);
        })
        .process(|stream, user_data| {
            let Some(mut buffer) = stream.dequeue_buffer() else {
                return;
            };

            let datas = buffer.datas_mut();
            if datas.is_empty() {
                return;
            }

            let data = &mut datas[0];
            let channel_count = user_data.format.channels() as usize;
            let sample_rate = user_data.format.rate();
            if channel_count == 0 {
                return;
            }

            if let Some(samples) = data.data() {
                let features =
                    user_data
                        .analyzer
                        .analyze_interleaved_f32(samples, channel_count, sample_rate);
                let _ = user_data.sender.try_send(features);
            }
        })
        .register()
        .map_err(|error| HueError::AudioCapture(error.to_string()))?;

    let mut audio_info = spa::param::audio::AudioInfoRaw::new();
    audio_info.set_format(spa::param::audio::AudioFormat::F32LE);
    let object = spa::pod::Object {
        type_: spa::utils::SpaTypes::ObjectParamFormat.as_raw(),
        id: spa::param::ParamType::EnumFormat.as_raw(),
        properties: audio_info.into(),
    };
    let values = spa::pod::serialize::PodSerializer::serialize(
        std::io::Cursor::new(Vec::new()),
        &spa::pod::Value::Object(object),
    )
    .map_err(|error| HueError::AudioCapture(error.to_string()))?
    .0
    .into_inner();
    let mut params = [Pod::from_bytes(&values).ok_or(HueError::AudioCapture(
        "failed to create PipeWire audio format pod".to_string(),
    ))?];

    stream
        .connect(
            spa::utils::Direction::Input,
            None,
            pw::stream::StreamFlags::AUTOCONNECT
                | pw::stream::StreamFlags::MAP_BUFFERS
                | pw::stream::StreamFlags::RT_PROCESS,
            &mut params,
        )
        .map_err(|error| HueError::AudioCapture(error.to_string()))?;

    info!("PipeWire stream connected");
    let _ = startup_tx.send(Ok(()));

    while !stop.load(Ordering::Relaxed) {
        let _ = mainloop
            .loop_()
            .iterate(std::time::Duration::from_millis(100).into());
    }

    let _ = stream.disconnect();
    debug!("PipeWire stream disconnected");
    Ok(())
}

#[cfg(target_os = "macos")]
fn run_macos_capture_loop(
    stop: Arc<AtomicBool>,
    sender: SyncSender<AudioFeatures>,
    startup_tx: std::sync::mpsc::Sender<Result<(), String>>,
    target_object: Option<String>,
) -> Result<(), HueError> {
    let content = SCShareableContent::get().map_err(|error| {
        HueError::AudioCapture(format!(
            "failed to query shareable displays. Check Screen Recording permission: {error}"
        ))
    })?;
    let mut displays = content.displays().into_iter();
    let selected_display = if let Some(target_object) = target_object
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        displays
            .find(|display| display.display_id().to_string() == target_object)
            .ok_or_else(|| {
                HueError::AudioCapture(format!(
                    "the selected display audio source `{target_object}` is no longer available"
                ))
            })?
    } else {
        displays.next().ok_or_else(|| {
            HueError::AudioCapture(
                "no displays are available for ScreenCaptureKit audio capture".to_string(),
            )
        })?
    };

    let display_id = selected_display.display_id();
    let filter = SCContentFilter::create()
        .with_display(&selected_display)
        .with_excluding_windows(&[])
        .build();
    let config = SCStreamConfiguration::new()
        .with_width(selected_display.width())
        .with_height(selected_display.height())
        .with_captures_audio(true)
        .with_sample_rate(48_000)
        .with_channel_count(2);

    let analyzer = Arc::new(Mutex::new(AudioAnalyzer::new()));
    let queue = DispatchQueue::new(
        "io.cloudflavor.seasons.audio-sync",
        DispatchQoS::UserInteractive,
    );
    let handler_analyzer = Arc::clone(&analyzer);
    let handler_sender = sender.clone();

    let mut stream = SCStream::new(&filter, &config);
    let _handler_id = stream
        .add_output_handler_with_queue(
            move |sample, output_type| {
                if output_type != SCStreamOutputType::Audio {
                    return;
                }

                if let Some(features) =
                    extract_macos_audio_features(&sample, &handler_analyzer, 48_000)
                {
                    let _ = handler_sender.try_send(features);
                }
            },
            SCStreamOutputType::Audio,
            Some(&queue),
        )
        .ok_or_else(|| {
            HueError::AudioCapture(
                "failed to register the ScreenCaptureKit audio output handler".to_string(),
            )
        })?;

    stream.start_capture().map_err(|error| {
        HueError::AudioCapture(format!(
            "failed to start ScreenCaptureKit capture. Check Screen Recording permission: {error}"
        ))
    })?;
    info!(display_id, "ScreenCaptureKit audio capture started");
    let _ = startup_tx.send(Ok(()));

    while !stop.load(Ordering::Relaxed) {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    let _ = stream.stop_capture();
    debug!(display_id, "ScreenCaptureKit audio capture stopped");
    Ok(())
}

#[cfg(target_os = "macos")]
fn extract_macos_audio_features(
    sample: &CMSampleBuffer,
    analyzer: &Arc<Mutex<AudioAnalyzer>>,
    sample_rate: u32,
) -> Option<AudioFeatures> {
    let audio_buffer_list = sample.audio_buffer_list()?;
    let (bytes, channel_count) = audio_buffer_list_to_interleaved_f32(&audio_buffer_list)?;
    let mut analyzer = analyzer.lock().ok()?;
    Some(analyzer.analyze_interleaved_f32(&bytes, channel_count, sample_rate))
}

#[cfg(target_os = "macos")]
fn audio_buffer_list_to_interleaved_f32(buffer_list: &AudioBufferList) -> Option<(Vec<u8>, usize)> {
    match buffer_list.num_buffers() {
        0 => None,
        1 => {
            let buffer = buffer_list.get(0)?;
            let channel_count = buffer.number_channels.max(1) as usize;
            let data = buffer.data();
            if data.is_empty() || data.len() % std::mem::size_of::<f32>() != 0 {
                return None;
            }
            Some((data.to_vec(), channel_count))
        }
        channel_count => {
            let mut channel_buffers = Vec::with_capacity(channel_count);
            let mut frame_count = usize::MAX;

            for index in 0..channel_count {
                let buffer = buffer_list.get(index)?;
                if buffer.number_channels != 1 {
                    return None;
                }

                let data = buffer.data();
                if data.is_empty() || data.len() % std::mem::size_of::<f32>() != 0 {
                    return None;
                }

                frame_count = frame_count.min(data.len() / std::mem::size_of::<f32>());
                channel_buffers.push(data);
            }

            if frame_count == 0 || frame_count == usize::MAX {
                return None;
            }

            let mut interleaved =
                Vec::with_capacity(frame_count * channel_count * std::mem::size_of::<f32>());

            for frame_index in 0..frame_count {
                let sample_offset = frame_index * std::mem::size_of::<f32>();
                for buffer in &channel_buffers {
                    interleaved.extend_from_slice(
                        &buffer[sample_offset..sample_offset + std::mem::size_of::<f32>()],
                    );
                }
            }

            Some((interleaved, channel_count))
        }
    }
}

#[cfg(target_os = "linux")]
fn collect_linux_output_target(
    global: &GlobalObject<&DictRef>,
    targets: &Arc<Mutex<Vec<PipeWireOutputTarget>>>,
) {
    if global.type_ != ObjectType::Node {
        return;
    }

    let Some(props) = global.props.as_ref() else {
        return;
    };

    let media_class = prop(props, *pw::keys::MEDIA_CLASS);
    if media_class.as_deref() != Some("Audio/Sink") {
        return;
    }

    let target_object = prop(props, "object.serial").or_else(|| prop(props, *pw::keys::NODE_NAME));
    let Some(target_object) = target_object.filter(|value| !value.is_empty()) else {
        return;
    };

    let description = prop(props, *pw::keys::NODE_DESCRIPTION)
        .or_else(|| prop(props, *pw::keys::NODE_NICK))
        .or_else(|| prop(props, *pw::keys::NODE_NAME))
        .unwrap_or_else(|| "Audio output".to_string());
    let name = prop(props, *pw::keys::NODE_NAME).unwrap_or_else(|| description.clone());

    if let Ok(mut targets) = targets.lock() {
        debug!(
            target_object = %target_object,
            description = %description,
            "found PipeWire output target"
        );
        targets.push(PipeWireOutputTarget {
            target_object,
            name,
            description,
            media_class: media_class.unwrap_or_else(|| "Audio/Sink".to_string()),
        });
    }
}

#[cfg(target_os = "linux")]
fn prop(props: &DictRef, key: &str) -> Option<String> {
    props.get(key).map(|value| value.to_string())
}
