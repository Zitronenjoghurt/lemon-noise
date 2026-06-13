use crate::ui::icons;
use crate::ui::node_type_icon;
use crate::ui::widgets::{CardAction, ModEvent, NodeCard, Waveform};
use crate::ui::windows::{ExportRequest, ExportWindow, Window};
use crate::utils::file_loader::FileLoader;
use crate::utils::file_saver::FileSaver;
use crate::utils::persistence::default_backend;
use eframe::CreationContext;
use egui::{Align, Frame, Layout, Slider, TextEdit};
use egui_notify::Toasts;
use lemon_noise::persistence::{self, PersistenceBackend};
use lemon_noise::{
    AudioEngine, Binding, Channel, DEFAULT_SAMPLE_RATE, Mixer, ParamValue, ProjectState, Registry,
    WavFormat, encode_wav,
};
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{Receiver, Sender, channel};

#[derive(Serialize, Deserialize)]
struct SavedState {
    project: ProjectState,
    volume: f32,
    export_seconds: f32,
    export_format: WavFormat,
    selected_device: Option<String>,
}

enum ChannelOp {
    Add,
    Remove(usize),
    Rename(usize, String),
    Gain(usize, f32),
    Muted(usize, bool),
    Soloed(usize, bool),
    AddNode(usize, &'static str),
}

pub struct LemonNoise {
    registry: Registry,
    mixer: Mixer,
    audio: Option<AudioEngine>,
    devices: Vec<String>,
    selected_device: Option<String>,
    scope: Vec<f32>,
    volume: f32,
    export_window: ExportWindow,
    backend: Box<dyn PersistenceBackend>,
    import_tx: Sender<Vec<u8>>,
    import_rx: Receiver<Vec<u8>>,
    toasts: Toasts,
}

impl LemonNoise {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        Self::setup_fonts(&cc.egui_ctx);

        let registry = Registry::with_builtins();
        let backend = default_backend();
        let saved: Option<SavedState> = backend.load().and_then(|raw| persistence::decode(&raw));

        let mut volume = 0.5;
        let mut selected_device = AudioEngine::default_device_name();
        let mut export_window = ExportWindow::new();
        let mixer = match &saved {
            Some(state) => {
                volume = state.volume;
                selected_device = state.selected_device.clone();
                export_window.restore(state.export_seconds, state.export_format);
                state.project.apply(&registry, DEFAULT_SAMPLE_RATE)
            }
            None => default_mixer(&registry),
        };

        let (import_tx, import_rx) = channel();

        Self {
            registry,
            mixer,
            audio: None,
            devices: AudioEngine::output_devices(),
            selected_device,
            scope: Vec::new(),
            volume,
            export_window,
            backend,
            import_tx,
            import_rx,
            toasts: Toasts::default(),
        }
    }

    fn persist(&self) {
        let state = SavedState {
            project: ProjectState::capture(&self.mixer),
            volume: self.volume,
            export_seconds: self.export_window.seconds(),
            export_format: self.export_window.format(),
            selected_device: self.selected_device.clone(),
        };
        if let Some(bytes) = persistence::encode(&state) {
            self.backend.save(&bytes);
        }
    }

    fn export_config(&mut self) {
        let project = ProjectState::capture(&self.mixer);
        if let Some(bytes) = persistence::encode(&project) {
            FileSaver::new()
                .title("Export node configuration")
                .file_name("config.lmns")
                .dispatch(bytes);
            self.toasts.info("Choose where to save the configuration.");
        } else {
            self.toasts.error("Could not encode the configuration.");
        }
    }

    fn import_config(&self) {
        FileLoader::new()
            .title("Import node configuration")
            .add_filter("Lemon Noise config", &["lmns"])
            .dispatch(self.import_tx.clone());
    }

    fn load_project(&mut self, project: ProjectState) {
        let sample_rate = self.sample_rate();
        self.mixer = project.apply(&self.registry, sample_rate);
        if let Some(engine) = &self.audio {
            let playing = engine.is_playing();
            self.audio = None;
            self.start_audio(playing);
        }
    }

    fn sample_rate(&self) -> u32 {
        self.audio
            .as_ref()
            .map(|engine| engine.sample_rate())
            .unwrap_or_else(|| self.mixer.sample_rate())
    }

    fn export_wav(&mut self, request: ExportRequest) {
        let sample_rate = request.sample_rate.max(1);

        let mut mixer = self.mixer.clone();
        mixer.set_sample_rate(sample_rate);
        mixer.reset();

        let frames = (request.seconds.max(0.1) * sample_rate as f32) as usize;
        let mut samples = mixer.render(frames);
        for sample in &mut samples {
            *sample *= self.volume;
        }

        self.toasts.info("Rendering and saving the mix…");
        let bytes = encode_wav(&samples, sample_rate, request.format);
        FileSaver::new()
            .title("Export noise as WAV")
            .file_name("lemon-noise.wav")
            .dispatch(bytes);
    }

    fn setup_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
        ctx.set_fonts(fonts);
    }

    fn toggle_audio(&mut self) {
        match &self.audio {
            Some(engine) => engine.set_playing(!engine.is_playing()),
            None => self.start_audio(true),
        }
    }

    fn start_audio(&mut self, playing: bool) {
        match AudioEngine::spawn_on(self.mixer.clone(), self.selected_device.as_deref()) {
            Ok(engine) => {
                engine.set_volume(self.volume);
                engine.set_playing(playing);
                self.audio = Some(engine);
            }
            Err(error) => {
                log::error!("Failed to start audio: {error}");
                self.toasts.error(format!("Failed to start audio: {error}"));
            }
        }
    }

    fn select_device(&mut self, name: Option<String>) {
        if name == self.selected_device {
            return;
        }
        self.selected_device = name;
        if let Some(engine) = &self.audio {
            let playing = engine.is_playing();
            self.audio = None;
            self.start_audio(playing);
        }
    }

    fn is_playing(&self) -> bool {
        self.audio
            .as_ref()
            .is_some_and(|engine| engine.is_playing())
    }

    fn device_picker(&mut self, ui: &mut egui::Ui) {
        ui.label(icons::HEADPHONES);
        if ui
            .button(icons::ARROWS_CLOCKWISE)
            .on_hover_text("Refresh output devices")
            .clicked()
        {
            self.devices = AudioEngine::output_devices();
        }

        let selected_text = self
            .selected_device
            .clone()
            .unwrap_or_else(|| "System default".to_string());
        let mut choice = self.selected_device.clone();

        egui::ComboBox::from_id_salt("output_device")
            .selected_text(selected_text)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut choice, None, "System default");
                for name in &self.devices {
                    ui.selectable_value(&mut choice, Some(name.clone()), name);
                }
            })
            .response
            .on_hover_text("Audio output device");

        ui.weak(format!("{} Hz", self.sample_rate()))
            .on_hover_text("Output sample rate of the selected device");

        self.select_device(choice);
    }

    fn set_param(&mut self, channel: usize, index: usize, id: &'static str, value: ParamValue) {
        if let Some(channel_ref) = self.mixer.channel_mut(channel)
            && let Some(node) = channel_ref.pipeline.node_mut(index)
        {
            node.set_param(id, value);
        }
        if let Some(engine) = &self.audio {
            engine.set_param(channel, index, id, value);
        }
    }

    fn set_binding(&mut self, channel: usize, binding: Binding) {
        if let Some(engine) = &self.audio {
            engine.set_binding(channel, binding.clone());
        }
        if let Some(channel_ref) = self.mixer.channel_mut(channel) {
            channel_ref.pipeline.set_binding(binding);
        }
    }

    fn remove_binding(&mut self, channel: usize, node: usize, param: &'static str) {
        let Some(channel_ref) = self.mixer.channel_mut(channel) else {
            return;
        };
        let base = channel_ref
            .pipeline
            .binding(node, param)
            .map(|binding| binding.base);
        channel_ref.pipeline.remove_binding(node, param);
        if let Some(base) = base {
            if let Some(target) = channel_ref.pipeline.node_mut(node) {
                target.set_param(param, base);
            }
            if let Some(engine) = &self.audio {
                engine.remove_binding(channel, node, param, base);
            }
        }
    }

    fn add_node(&mut self, channel: usize, id: &str) {
        if let Some(node) = self.registry.create(id) {
            if let Some(engine) = &self.audio {
                engine.push(channel, node.clone());
            }
            if let Some(channel_ref) = self.mixer.channel_mut(channel) {
                channel_ref.pipeline.push(node);
            }
        }
    }

    fn remove_node(&mut self, channel: usize, index: usize) {
        if let Some(channel_ref) = self.mixer.channel_mut(channel) {
            channel_ref.pipeline.remove(index);
        }
        if let Some(engine) = &self.audio {
            engine.remove(channel, index);
        }
    }

    fn set_node_enabled(&mut self, channel: usize, index: usize, enabled: bool) {
        if let Some(channel_ref) = self.mixer.channel_mut(channel) {
            channel_ref.pipeline.set_enabled(index, enabled);
        }
        if let Some(engine) = &self.audio {
            engine.set_enabled(channel, index, enabled);
        }
    }

    fn change_node_type(&mut self, channel: usize, index: usize, id: &str) {
        if let Some(node) = self.registry.create(id) {
            if let Some(engine) = &self.audio {
                engine.replace(channel, index, node.clone());
            }
            if let Some(channel_ref) = self.mixer.channel_mut(channel) {
                channel_ref.pipeline.replace(index, node);
            }
        }
    }

    fn move_node(&mut self, channel: usize, a: usize, b: usize) {
        if let Some(channel_ref) = self.mixer.channel_mut(channel) {
            channel_ref.pipeline.swap(a, b);
        }
        if let Some(engine) = &self.audio {
            engine.swap(channel, a, b);
        }
    }

    fn add_channel(&mut self) {
        let name = format!("Channel {}", self.mixer.len() + 1);
        let index = self.mixer.add_channel(name);
        if let Some(engine) = &self.audio
            && let Some(channel) = self.mixer.channel(index)
        {
            engine.add_channel(channel.clone());
        }
    }

    fn remove_channel(&mut self, index: usize) {
        self.mixer.remove_channel(index);
        if let Some(engine) = &self.audio {
            engine.remove_channel(index);
        }
    }

    fn rename_channel(&mut self, index: usize, name: String) {
        if let Some(channel) = self.mixer.channel_mut(index) {
            channel.name = name;
        }
    }

    fn set_channel_gain(&mut self, index: usize, gain: f32) {
        if let Some(channel) = self.mixer.channel_mut(index) {
            channel.gain = gain;
        }
        if let Some(engine) = &self.audio {
            engine.set_channel_gain(index, gain);
        }
    }

    fn set_channel_muted(&mut self, index: usize, muted: bool) {
        if let Some(channel) = self.mixer.channel_mut(index) {
            channel.muted = muted;
        }
        if let Some(engine) = &self.audio {
            engine.set_channel_muted(index, muted);
        }
    }

    fn set_channel_soloed(&mut self, index: usize, soloed: bool) {
        if let Some(channel) = self.mixer.channel_mut(index) {
            channel.soloed = soloed;
        }
        if let Some(engine) = &self.audio {
            engine.set_channel_soloed(index, soloed);
        }
    }

    fn reset_mixer(&mut self) {
        self.mixer.reset();
        if let Some(engine) = &self.audio {
            engine.reset();
        }
    }

    fn top_bar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("top_bar").show_inside(ui, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.heading(format!("{} Lemon Noise", icons::WAVE_SINE));
                ui.separator();

                let playing = self.is_playing();
                let play_icon = if playing { icons::PAUSE } else { icons::PLAY };
                if ui
                    .button(play_icon)
                    .on_hover_text(if playing { "Pause" } else { "Play" })
                    .clicked()
                {
                    self.toggle_audio();
                }

                if ui
                    .button(icons::ARROW_COUNTER_CLOCKWISE)
                    .on_hover_text("Reset all node state")
                    .clicked()
                {
                    self.reset_mixer();
                }

                ui.menu_button(format!("{} Project", icons::FLOPPY_DISK), |ui| {
                    if ui
                        .button(format!("{} Export config", icons::EXPORT))
                        .on_hover_text("Save the current channels and nodes to a file")
                        .clicked()
                    {
                        self.export_config();
                        ui.close();
                    }
                    if ui
                        .button(format!("{} Import config", icons::UPLOAD_SIMPLE))
                        .on_hover_text("Load channels and nodes from a file")
                        .clicked()
                    {
                        self.import_config();
                        ui.close();
                    }
                });

                ui.separator();
                ui.label(icons::SPEAKER_HIGH);
                if ui
                    .add(Slider::new(&mut self.volume, 0.0..=1.0).show_value(false))
                    .changed()
                    && let Some(engine) = &self.audio
                {
                    engine.set_volume(self.volume);
                }

                ui.separator();
                self.device_picker(ui);

                ui.separator();
                if ui
                    .button(format!("{} Export", icons::DOWNLOAD_SIMPLE))
                    .on_hover_text("Open the export window")
                    .clicked()
                {
                    self.export_window.toggle();
                }
            });
            ui.add_space(2.0);
        });
    }

    fn channel_rack(&mut self, ui: &mut egui::Ui) {
        let mut param_changes: Vec<(usize, usize, &'static str, ParamValue)> = Vec::new();
        let mut mod_events: Vec<(usize, ModEvent)> = Vec::new();
        let mut node_actions: Vec<(usize, usize, CardAction)> = Vec::new();
        let mut channel_ops: Vec<ChannelOp> = Vec::new();

        {
            let mixer = &self.mixer;
            let registry = &self.registry;
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (ci, channel) in mixer.channels().iter().enumerate() {
                    Frame::group(ui.style()).show(ui, |ui| {
                        channel_header(ui, ci, channel, registry, &mut channel_ops);
                        ui.separator();
                        channel_strip(
                            ui,
                            ci,
                            channel,
                            registry,
                            &mut param_changes,
                            &mut mod_events,
                            &mut node_actions,
                        );
                    });
                    ui.add_space(6.0);
                }

                if ui.button(format!("{} Add channel", icons::PLUS)).clicked() {
                    channel_ops.push(ChannelOp::Add);
                }
            });
        }

        for (channel, index, id, value) in param_changes {
            self.set_param(channel, index, id, value);
        }

        for (channel, event) in mod_events {
            match event {
                ModEvent::Set(binding) => self.set_binding(channel, binding),
                ModEvent::Remove { node, param } => self.remove_binding(channel, node, param),
            }
        }

        for op in channel_ops {
            match op {
                ChannelOp::Add => self.add_channel(),
                ChannelOp::Remove(index) => self.remove_channel(index),
                ChannelOp::Rename(index, name) => self.rename_channel(index, name),
                ChannelOp::Gain(index, gain) => self.set_channel_gain(index, gain),
                ChannelOp::Muted(index, muted) => self.set_channel_muted(index, muted),
                ChannelOp::Soloed(index, soloed) => self.set_channel_soloed(index, soloed),
                ChannelOp::AddNode(index, id) => self.add_node(index, id),
            }
        }

        for (channel, index, action) in node_actions {
            match action {
                CardAction::Remove => self.remove_node(channel, index),
                CardAction::MoveLeft => self.move_node(channel, index, index - 1),
                CardAction::MoveRight => self.move_node(channel, index, index + 1),
                CardAction::SetEnabled(enabled) => self.set_node_enabled(channel, index, enabled),
                CardAction::ChangeType(id) => self.change_node_type(channel, index, id),
            }
        }
    }
}

fn default_mixer(registry: &Registry) -> Mixer {
    let mut mixer = Mixer::new(DEFAULT_SAMPLE_RATE);
    let index = mixer.add_channel("Channel 1");
    if let Some(channel) = mixer.channel_mut(index) {
        if let Some(node) = registry.create("pink") {
            channel.pipeline.push(node);
        }
        if let Some(node) = registry.create("low_pass") {
            channel.pipeline.push(node);
        }
    }
    mixer
}

fn channel_header(
    ui: &mut egui::Ui,
    index: usize,
    channel: &Channel,
    registry: &Registry,
    ops: &mut Vec<ChannelOp>,
) {
    ui.horizontal(|ui| {
        let mut name = channel.name.clone();
        if ui
            .add(TextEdit::singleline(&mut name).desired_width(140.0))
            .changed()
        {
            ops.push(ChannelOp::Rename(index, name));
        }

        if ui
            .selectable_label(channel.muted, "M")
            .on_hover_text("Mute")
            .clicked()
        {
            ops.push(ChannelOp::Muted(index, !channel.muted));
        }
        if ui
            .selectable_label(channel.soloed, "S")
            .on_hover_text("Solo")
            .clicked()
        {
            ops.push(ChannelOp::Soloed(index, !channel.soloed));
        }

        ui.label(icons::SPEAKER_HIGH);
        let mut gain = channel.gain;
        if ui
            .add(Slider::new(&mut gain, 0.0..=1.5).show_value(false))
            .on_hover_text("Channel level")
            .changed()
        {
            ops.push(ChannelOp::Gain(index, gain));
        }

        ui.menu_button(format!("{} Add node", icons::PLUS), |ui| {
            for descriptor in registry.descriptors() {
                let icon = node_type_icon(descriptor.is_source);
                if ui
                    .button(format!("{icon}  {}", descriptor.label))
                    .on_hover_text(descriptor.description)
                    .clicked()
                {
                    ops.push(ChannelOp::AddNode(index, descriptor.id));
                    ui.close();
                }
            }
        });

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui
                .small_button(icons::TRASH)
                .on_hover_text("Remove channel")
                .clicked()
            {
                ops.push(ChannelOp::Remove(index));
            }
        });
    });
}

fn channel_strip(
    ui: &mut egui::Ui,
    channel_index: usize,
    channel: &Channel,
    registry: &Registry,
    param_changes: &mut Vec<(usize, usize, &'static str, ParamValue)>,
    mod_events: &mut Vec<(usize, ModEvent)>,
    node_actions: &mut Vec<(usize, usize, CardAction)>,
) {
    let pipeline = &channel.pipeline;
    let bindings = pipeline.bindings();
    let count = pipeline.len();

    egui::ScrollArea::horizontal()
        .id_salt(("strip", channel_index))
        .show(ui, |ui| {
            ui.horizontal_top(|ui| {
                if count == 0 {
                    ui.label("Empty — use \"Add node\" to start this layer.");
                }
                for index in 0..count {
                    if let Some(node) = pipeline.node(index) {
                        let response = NodeCard::new(
                            node,
                            index,
                            count,
                            pipeline.is_enabled(index),
                            bindings,
                            registry,
                        )
                        .show(ui);
                        for (id, value) in response.changes {
                            param_changes.push((channel_index, index, id, value));
                        }
                        for event in response.mods {
                            mod_events.push((channel_index, event));
                        }
                        if let Some(action) = response.action {
                            node_actions.push((channel_index, index, action));
                        }
                    }
                    if index + 1 < count {
                        ui.add_space(4.0);
                        ui.label(icons::ARROW_RIGHT);
                        ui.add_space(4.0);
                    }
                }
            });
        });
}

impl eframe::App for LemonNoise {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        while let Ok(bytes) = self.import_rx.try_recv() {
            match persistence::decode::<ProjectState>(&bytes) {
                Some(project) => {
                    self.load_project(project);
                    self.toasts.success("Configuration imported.");
                }
                None => {
                    self.toasts.error("Could not read that configuration file.");
                }
            }
        }

        let sample_rate = self.sample_rate();

        if let Some(engine) = self.audio.as_ref() {
            engine.read_scope(&mut self.scope);
            ui.ctx().request_repaint();
        }

        self.top_bar(ui);

        egui::CentralPanel::default().show_inside(ui, |ui| {
            Waveform::new(&self.scope, sample_rate)
                .height(160.0)
                .show(ui);
            ui.add_space(8.0);
            self.channel_rack(ui);
        });

        self.export_window.set_device_rate(sample_rate);
        self.export_window.show(ui.ctx());
        if let Some(request) = self.export_window.take_request() {
            self.export_wav(request);
        }

        self.toasts.show(ui.ctx());
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        self.persist();
    }

    fn auto_save_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_mixer_has_one_channel_with_two_nodes() {
        let registry = Registry::with_builtins();
        let mixer = default_mixer(&registry);
        assert_eq!(mixer.len(), 1);
        assert_eq!(mixer.channel(0).unwrap().pipeline.len(), 2);
    }

    #[test]
    fn saved_state_roundtrips_through_bytes() {
        let registry = Registry::with_builtins();
        let state = SavedState {
            project: ProjectState::capture(&default_mixer(&registry)),
            volume: 0.33,
            export_seconds: 7.0,
            export_format: WavFormat::Float32,
            selected_device: Some("Speakers".to_string()),
        };

        let bytes = persistence::encode(&state).expect("encode");
        let restored: SavedState = persistence::decode(&bytes).expect("decode");

        assert!((restored.volume - 0.33).abs() < 1e-6);
        assert!((restored.export_seconds - 7.0).abs() < 1e-6);
        assert_eq!(restored.export_format, WavFormat::Float32);
        assert_eq!(restored.selected_device.as_deref(), Some("Speakers"));

        let mixer = restored.project.apply(&registry, 48_000);
        assert_eq!(mixer.len(), 1);
        assert_eq!(mixer.sample_rate(), 48_000);
    }
}
