use egui::{ComboBox, DragValue, Slider, Ui};
use lemon_noise::{ParamKind, ParamSpec, ParamValue};

pub struct ParamControl<'a> {
    spec: &'a ParamSpec,
    value: ParamValue,
}

impl<'a> ParamControl<'a> {
    pub fn new(spec: &'a ParamSpec, value: ParamValue) -> Self {
        Self { spec, value }
    }

    pub fn show(self, ui: &mut Ui) -> Option<ParamValue> {
        let spec = self.spec;

        match spec.kind {
            ParamKind::Float {
                min,
                max,
                logarithmic,
                ..
            } => {
                let mut value = self.value.as_float();
                let mut slider = Slider::new(&mut value, min..=max).logarithmic(logarithmic);
                if let Some(unit) = spec.unit {
                    slider = slider.suffix(unit);
                }
                let response = with_hint(ui.add(slider), spec.description);
                response.changed().then_some(ParamValue::Float(value))
            }
            ParamKind::Int { min, max, .. } => {
                ui.horizontal(|ui| {
                    let mut value = self.value.as_int();
                    let response = with_hint(
                        ui.add(DragValue::new(&mut value).speed(1.0).range(min..=max)),
                        spec.description,
                    );
                    with_hint(ui.label(spec.label), spec.description);
                    response.changed().then_some(ParamValue::Int(value))
                })
                .inner
            }
            ParamKind::Bool { .. } => {
                let mut value = self.value.as_bool();
                let response = with_hint(ui.checkbox(&mut value, spec.label), spec.description);
                response.changed().then_some(ParamValue::Bool(value))
            }
            ParamKind::Choice { options, .. } => {
                let mut selected = (self.value.as_int().max(0) as usize).min(options.len() - 1);
                let mut changed = false;
                ui.horizontal(|ui| {
                    let response = ComboBox::from_id_salt(spec.id)
                        .selected_text(options[selected])
                        .show_ui(ui, |ui| {
                            for (index, option) in options.iter().enumerate() {
                                changed |=
                                    ui.selectable_value(&mut selected, index, *option).clicked();
                            }
                        })
                        .response;
                    with_hint(response, spec.description);
                    with_hint(ui.label(spec.label), spec.description);
                });
                changed.then_some(ParamValue::Int(selected as i64))
            }
        }
    }
}

fn with_hint(response: egui::Response, hint: &str) -> egui::Response {
    if hint.is_empty() {
        response
    } else {
        response.on_hover_text(hint)
    }
}
