use crate::mixer::{Channel, Mixer};
use crate::modulation::Binding;
use crate::node::Node;
use crate::param::ParamValue;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, StreamConfig};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};

const SCOPE_LEN: usize = 4096;

#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("No default output device available")]
    NoDevice,
    #[error("Unsupported sample format: {0:?}")]
    UnsupportedFormat(SampleFormat),
    #[error(transparent)]
    Cpal(#[from] cpal::Error),
}

enum Command {
    SetParam {
        channel: usize,
        index: usize,
        id: &'static str,
        value: ParamValue,
    },
    Push {
        channel: usize,
        node: Box<dyn Node>,
    },
    Remove {
        channel: usize,
        index: usize,
    },
    Swap {
        channel: usize,
        a: usize,
        b: usize,
    },
    SetEnabled {
        channel: usize,
        index: usize,
        enabled: bool,
    },
    Replace {
        channel: usize,
        index: usize,
        node: Box<dyn Node>,
    },
    SetBinding {
        channel: usize,
        binding: Binding,
    },
    RemoveBinding {
        channel: usize,
        node: usize,
        param: &'static str,
        base: ParamValue,
    },
    AddChannel(Channel),
    RemoveChannel(usize),
    SetChannelGain(usize, f32),
    SetChannelMuted(usize, bool),
    SetChannelSoloed(usize, bool),
    Reset,
}

struct Shared {
    playing: AtomicBool,
    volume: AtomicU32,
    scope: Mutex<Scope>,
}

pub struct AudioEngine {
    _stream: cpal::Stream,
    commands: Sender<Command>,
    shared: Arc<Shared>,
    sample_rate: u32,
    device_name: Option<String>,
}

impl AudioEngine {
    pub fn output_devices() -> Vec<String> {
        let host = cpal::default_host();
        let mut names = Vec::new();
        if let Ok(devices) = host.output_devices() {
            for device in devices {
                names.push(device.to_string());
            }
        }
        names
    }

    pub fn default_device_name() -> Option<String> {
        cpal::default_host()
            .default_output_device()
            .map(|device| device.to_string())
    }

    pub fn spawn(mixer: Mixer) -> Result<Self, AudioError> {
        Self::spawn_on(mixer, None)
    }

    pub fn spawn_on(mut mixer: Mixer, device_name: Option<&str>) -> Result<Self, AudioError> {
        let host = cpal::default_host();
        let device = pick_device(&host, device_name).ok_or(AudioError::NoDevice)?;
        let resolved_name = Some(device.to_string());
        let supported = device.default_output_config()?;
        let sample_format = supported.sample_format();
        if sample_format != SampleFormat::F32 {
            return Err(AudioError::UnsupportedFormat(sample_format));
        }

        let config: StreamConfig = supported.into();
        let sample_rate = config.sample_rate;
        let channels = config.channels as usize;
        mixer.set_sample_rate(sample_rate);

        let shared = Arc::new(Shared {
            playing: AtomicBool::new(true),
            volume: AtomicU32::new(0.5f32.to_bits()),
            scope: Mutex::new(Scope::new(SCOPE_LEN)),
        });

        let (commands, rx) = channel::<Command>();
        let renderer = Renderer {
            mixer,
            channels,
            shared: shared.clone(),
            rx,
        };

        let stream = device.build_output_stream(
            config,
            renderer.into_callback(),
            |err| log_error(&err),
            None,
        )?;
        stream.play()?;

        Ok(Self {
            _stream: stream,
            commands,
            shared,
            sample_rate,
            device_name: resolved_name,
        })
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn device_name(&self) -> Option<&str> {
        self.device_name.as_deref()
    }

    pub fn set_playing(&self, playing: bool) {
        self.shared.playing.store(playing, Ordering::Relaxed);
    }

    pub fn is_playing(&self) -> bool {
        self.shared.playing.load(Ordering::Relaxed)
    }

    pub fn set_volume(&self, volume: f32) {
        self.shared
            .volume
            .store(volume.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
    }

    pub fn volume(&self) -> f32 {
        f32::from_bits(self.shared.volume.load(Ordering::Relaxed))
    }

    pub fn set_param(&self, channel: usize, index: usize, id: &'static str, value: ParamValue) {
        let _ = self.commands.send(Command::SetParam {
            channel,
            index,
            id,
            value,
        });
    }

    pub fn push(&self, channel: usize, node: Box<dyn Node>) {
        let _ = self.commands.send(Command::Push { channel, node });
    }

    pub fn remove(&self, channel: usize, index: usize) {
        let _ = self.commands.send(Command::Remove { channel, index });
    }

    pub fn swap(&self, channel: usize, a: usize, b: usize) {
        let _ = self.commands.send(Command::Swap { channel, a, b });
    }

    pub fn set_enabled(&self, channel: usize, index: usize, enabled: bool) {
        let _ = self.commands.send(Command::SetEnabled {
            channel,
            index,
            enabled,
        });
    }

    pub fn replace(&self, channel: usize, index: usize, node: Box<dyn Node>) {
        let _ = self.commands.send(Command::Replace {
            channel,
            index,
            node,
        });
    }

    pub fn set_binding(&self, channel: usize, binding: Binding) {
        let _ = self.commands.send(Command::SetBinding { channel, binding });
    }

    pub fn remove_binding(
        &self,
        channel: usize,
        node: usize,
        param: &'static str,
        base: ParamValue,
    ) {
        let _ = self.commands.send(Command::RemoveBinding {
            channel,
            node,
            param,
            base,
        });
    }

    pub fn add_channel(&self, channel: Channel) {
        let _ = self.commands.send(Command::AddChannel(channel));
    }

    pub fn remove_channel(&self, index: usize) {
        let _ = self.commands.send(Command::RemoveChannel(index));
    }

    pub fn set_channel_gain(&self, index: usize, gain: f32) {
        let _ = self.commands.send(Command::SetChannelGain(index, gain));
    }

    pub fn set_channel_muted(&self, index: usize, muted: bool) {
        let _ = self.commands.send(Command::SetChannelMuted(index, muted));
    }

    pub fn set_channel_soloed(&self, index: usize, soloed: bool) {
        let _ = self.commands.send(Command::SetChannelSoloed(index, soloed));
    }

    pub fn reset(&self) {
        let _ = self.commands.send(Command::Reset);
    }

    pub fn read_scope(&self, out: &mut Vec<f32>) {
        if let Ok(scope) = self.shared.scope.lock() {
            scope.copy_into(out);
        }
    }
}

struct Renderer {
    mixer: Mixer,
    channels: usize,
    shared: Arc<Shared>,
    rx: Receiver<Command>,
}

impl Renderer {
    fn apply(&mut self, command: Command) {
        match command {
            Command::SetParam {
                channel,
                index,
                id,
                value,
            } => {
                if let Some(channel) = self.mixer.channel_mut(channel)
                    && let Some(node) = channel.pipeline.node_mut(index)
                {
                    node.set_param(id, value);
                }
            }
            Command::Push { channel, node } => {
                if let Some(channel) = self.mixer.channel_mut(channel) {
                    channel.pipeline.push(node);
                }
            }
            Command::Remove { channel, index } => {
                if let Some(channel) = self.mixer.channel_mut(channel) {
                    channel.pipeline.remove(index);
                }
            }
            Command::Swap { channel, a, b } => {
                if let Some(channel) = self.mixer.channel_mut(channel) {
                    channel.pipeline.swap(a, b);
                }
            }
            Command::SetEnabled {
                channel,
                index,
                enabled,
            } => {
                if let Some(channel) = self.mixer.channel_mut(channel) {
                    channel.pipeline.set_enabled(index, enabled);
                }
            }
            Command::Replace {
                channel,
                index,
                node,
            } => {
                if let Some(channel) = self.mixer.channel_mut(channel) {
                    channel.pipeline.replace(index, node);
                }
            }
            Command::SetBinding { channel, binding } => {
                if let Some(channel) = self.mixer.channel_mut(channel) {
                    channel.pipeline.set_binding(binding);
                }
            }
            Command::RemoveBinding {
                channel,
                node,
                param,
                base,
            } => {
                if let Some(channel) = self.mixer.channel_mut(channel) {
                    channel.pipeline.remove_binding(node, param);
                    if let Some(target) = channel.pipeline.node_mut(node) {
                        target.set_param(param, base);
                    }
                }
            }
            Command::AddChannel(channel) => self.mixer.push_channel(channel),
            Command::RemoveChannel(index) => {
                self.mixer.remove_channel(index);
            }
            Command::SetChannelGain(index, gain) => {
                if let Some(channel) = self.mixer.channel_mut(index) {
                    channel.gain = gain;
                }
            }
            Command::SetChannelMuted(index, muted) => {
                if let Some(channel) = self.mixer.channel_mut(index) {
                    channel.muted = muted;
                }
            }
            Command::SetChannelSoloed(index, soloed) => {
                if let Some(channel) = self.mixer.channel_mut(index) {
                    channel.soloed = soloed;
                }
            }
            Command::Reset => self.mixer.reset(),
        }
    }

    fn into_callback(
        mut self,
    ) -> impl FnMut(&mut [f32], &cpal::OutputCallbackInfo) + Send + 'static {
        move |data, _| {
            while let Ok(command) = self.rx.try_recv() {
                self.apply(command);
            }

            let playing = self.shared.playing.load(Ordering::Relaxed);
            let volume = f32::from_bits(self.shared.volume.load(Ordering::Relaxed));

            let mut guard = self.shared.scope.lock().ok();
            for frame in data.chunks_mut(self.channels) {
                let sample = if playing {
                    self.mixer.next().unwrap_or(0.0) * volume
                } else {
                    0.0
                };
                for out in frame.iter_mut() {
                    *out = sample;
                }
                if let Some(scope) = guard.as_mut() {
                    scope.push(sample);
                }
            }
        }
    }
}

struct Scope {
    buffer: Vec<f32>,
    write: usize,
}

impl Scope {
    fn new(len: usize) -> Self {
        Self {
            buffer: vec![0.0; len],
            write: 0,
        }
    }

    fn push(&mut self, sample: f32) {
        self.buffer[self.write] = sample;
        self.write = (self.write + 1) % self.buffer.len();
    }

    fn copy_into(&self, out: &mut Vec<f32>) {
        out.clear();
        out.extend_from_slice(&self.buffer[self.write..]);
        out.extend_from_slice(&self.buffer[..self.write]);
    }
}

fn pick_device(host: &cpal::Host, name: Option<&str>) -> Option<cpal::Device> {
    if let Some(name) = name
        && let Ok(devices) = host.output_devices()
    {
        for device in devices {
            if device.to_string() == name {
                return Some(device);
            }
        }
    }

    host.default_output_device()
        .or_else(|| host.output_devices().ok().and_then(|mut d| d.next()))
}

fn log_error(_error: &cpal::Error) {}
