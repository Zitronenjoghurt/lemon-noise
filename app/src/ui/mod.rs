pub mod widgets;
pub mod windows;

pub use egui_phosphor::regular as icons;

pub fn node_type_icon(is_source: bool) -> &'static str {
    if is_source {
        icons::WAVEFORM
    } else {
        icons::FADERS_HORIZONTAL
    }
}
