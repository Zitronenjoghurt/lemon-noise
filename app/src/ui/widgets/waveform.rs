use egui::Color32;
use egui_plot::{Line, Plot, PlotBounds, PlotPoints};

const ACCENT: Color32 = Color32::from_rgb(224, 200, 64);
const WINDOW_MS: f64 = 50.0;

pub struct Waveform<'a> {
    samples: &'a [f32],
    sample_rate: u32,
    height: f32,
}

impl<'a> Waveform<'a> {
    pub fn new(samples: &'a [f32], sample_rate: u32) -> Self {
        Self {
            samples,
            sample_rate,
            height: 180.0,
        }
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn show(self, ui: &mut egui::Ui) {
        let sample_rate = self.sample_rate.max(1) as f64;
        let window = ((sample_rate * WINDOW_MS / 1000.0) as usize).max(2);
        let start = self.samples.len().saturating_sub(window);
        let slice = &self.samples[start..];

        let points: PlotPoints = slice
            .iter()
            .enumerate()
            .map(|(i, &sample)| [i as f64 / sample_rate * 1000.0, sample as f64])
            .collect();

        Plot::new("scope")
            .height(self.height)
            .allow_drag(false)
            .allow_zoom(false)
            .allow_scroll(false)
            .allow_boxed_zoom(false)
            .show_grid(true)
            .x_axis_label("ms")
            .y_axis_label("amplitude")
            .show(ui, |plot_ui| {
                plot_ui.set_plot_bounds(PlotBounds::from_min_max([0.0, -1.05], [WINDOW_MS, 1.05]));
                plot_ui.line(Line::new("signal", points).color(ACCENT));
            });
    }
}
