use crate::audio::analysis::{AudioAnalyzer, AudioFeatures};
use crate::hue::error::HueError;
use crate::hue::models::PipeWireOutputTarget;
use pipewire as pw;
use pw::properties::properties;
use pw::registry::GlobalObject;
use pw::spa;
use pw::spa::param::format::{MediaSubtype, MediaType};
use pw::spa::param::format_utils;
use pw::spa::pod::Pod;
use pw::spa::utils::dict::DictRef;
use pw::types::ObjectType;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::{Arc, Mutex};
use tracing::{debug, info, warn};

struct CaptureUserData {
    format: spa::param::audio::AudioInfoRaw,
    analyzer: AudioAnalyzer,
    sender: SyncSender<AudioFeatures>,
}

pub struct AudioCaptureHandle {
    stop: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl AudioCaptureHandle {
    pub fn stop(mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

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
        if let Err(error) = run_capture_loop(thread_stop, feature_tx, startup_tx, target_object) {
            warn!(%error, "PipeWire capture loop exited with an error");
            let _ = startup_error_tx.send(Err(error.to_string()));
        }
    });

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
                "failed to receive PipeWire startup status: {error}"
            )))
        }
    }
}

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
            collect_output_target(global, &targets_for_listener);
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

fn run_capture_loop(
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

fn collect_output_target(
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

fn prop(props: &DictRef, key: &str) -> Option<String> {
    props.get(key).map(|value| value.to_string())
}
