use crate::ui::icons;
use crate::ui::windows::Window;
use egui::{ComboBox, DragValue};
use lemon_noise::WavFormat;

const PRESET_RATES: [u32; 4] = [22_050, 44_100, 48_000, 96_000];

pub struct ExportRequest {
    pub seconds: f32,
    pub sample_rate: u32,
    pub format: WavFormat,
}

pub struct ExportWindow {
    open: bool,
    seconds: f32,
    sample_rate: u32,
    format: WavFormat,
    device_rate: u32,
    request: Option<ExportRequest>,
}

impl ExportWindow {
    pub fn new() -> Self {
        Self {
            open: false,
            seconds: 10.0,
            sample_rate: 44_100,
            format: WavFormat::Pcm16,
            device_rate: 44_100,
            request: Option::None,
        }
    }

    pub fn seconds(&self) -> f32 {
        self.seconds
    }

    pub fn format(&self) -> WavFormat {
        self.format
    }

    pub fn restore(&mut self, seconds: f32, format: WavFormat) {
        self.seconds = seconds;
        self.format = format;
    }

    pub fn set_device_rate(&mut self, rate: u32) {
        self.device_rate = rate;
    }

    pub fn take_request(&mut self) -> Option<ExportRequest> {
        self.request.take()
    }

    fn rate_options(&self) -> Vec<u32> {
        let mut rates = vec![self.device_rate];
        for rate in PRESET_RATES {
            if !rates.contains(&rate) {
                rates.push(rate);
            }
        }
        rates
    }
}

impl Window for ExportWindow {
    fn title(&self) -> &str {
        "Export audio"
    }

    fn is_open(&self) -> bool {
        self.open
    }

    fn set_open(&mut self, open: bool) {
        self.open = open;
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("export_options")
            .num_columns(2)
            .spacing([12.0, 8.0])
            .show(ui, |ui| {
                ui.label("Length");
                ui.add(
                    DragValue::new(&mut self.seconds)
                        .speed(0.5)
                        .range(0.1..=600.0)
                        .suffix(" s"),
                );
                ui.end_row();

                ui.label("Sample rate");
                ComboBox::from_id_salt("export_rate")
                    .selected_text(format!("{} Hz", self.sample_rate))
                    .show_ui(ui, |ui| {
                        for rate in self.rate_options() {
                            let label = if rate == self.device_rate {
                                format!("{rate} Hz (device)")
                            } else {
                                format!("{rate} Hz")
                            };
                            ui.selectable_value(&mut self.sample_rate, rate, label);
                        }
                    });
                ui.end_row();

                ui.label("Format");
                ComboBox::from_id_salt("export_format")
                    .selected_text(self.format.label())
                    .show_ui(ui, |ui| {
                        for option in WavFormat::ALL {
                            ui.selectable_value(&mut self.format, option, option.label());
                        }
                    });
                ui.end_row();
            });

        ui.separator();
        if ui
            .button(format!("{} Render and save", icons::DOWNLOAD_SIMPLE))
            .clicked()
        {
            self.request = Some(ExportRequest {
                seconds: self.seconds,
                sample_rate: self.sample_rate,
                format: self.format,
            });
        }
    }
}
