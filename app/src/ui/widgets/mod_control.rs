use crate::ui::icons;
use egui::{ComboBox, DragValue, Slider, Ui};
use lemon_noise::{Binding, LfoShape, ModMode, Modulator, ParamKind, ParamSpec, ParamValue};

pub enum ModEvent {
    Set(Binding),
    Remove { node: usize, param: &'static str },
}

pub struct ModControl<'a> {
    spec: &'a ParamSpec,
    node: usize,
    binding: Option<&'a Binding>,
    current: ParamValue,
}

impl<'a> ModControl<'a> {
    pub fn new(
        spec: &'a ParamSpec,
        node: usize,
        binding: Option<&'a Binding>,
        current: ParamValue,
    ) -> Self {
        Self {
            spec,
            node,
            binding,
            current,
        }
    }

    pub fn show(self, ui: &mut Ui) -> Option<ModEvent> {
        let ParamKind::Float { min, max, .. } = self.spec.kind else {
            return None;
        };

        match self.binding {
            None => {
                let toggled = ui
                    .small_button(icons::WAVE_SINE)
                    .on_hover_text("Modulate this parameter")
                    .clicked();
                toggled.then(|| ModEvent::Set(default_binding(self.spec, self.node, self.current)))
            }
            Some(binding) => self.show_active(ui, binding, min, max),
        }
    }

    fn show_active(self, ui: &mut Ui, binding: &Binding, min: f32, max: f32) -> Option<ModEvent> {
        let spec = self.spec;
        let mut working = binding.clone();
        let mut changed = false;
        let mut remove = false;

        ui.group(|ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.strong(format!("{} {}", icons::WAVE_SINE, spec.label));
                if ui
                    .small_button(icons::X)
                    .on_hover_text("Stop modulating")
                    .clicked()
                {
                    remove = true;
                }
            });

            let mut base = working.base.as_float();
            let mut slider = Slider::new(&mut base, min..=max).text("Base");
            if let Some(unit) = spec.unit {
                slider = slider.suffix(unit);
            }
            if ui.add(slider).changed() {
                working.base = ParamValue::Float(base);
                changed = true;
            }

            changed |= shape_picker(ui, spec.id, &mut working.modulator);
            changed |= rate_control(ui, &mut working.modulator);

            let mut depth = working.depth;
            let depth_speed = ((max - min).abs() / 200.0).max(0.001);
            if ui
                .add(DragValue::new(&mut depth).speed(depth_speed))
                .on_hover_text("Modulation amount")
                .changed()
            {
                working.depth = depth;
                changed = true;
            }

            changed |= mode_picker(ui, spec.id, &mut working.mode);
        });

        if remove {
            Some(ModEvent::Remove {
                node: self.node,
                param: spec.id,
            })
        } else if changed {
            Some(ModEvent::Set(working))
        } else {
            None
        }
    }
}

fn default_binding(spec: &ParamSpec, node: usize, current: ParamValue) -> Binding {
    let mut binding = Binding::new(node, spec.id, current, Modulator::lfo(LfoShape::Sine, 2.0));
    if let ParamKind::Float { min, max, .. } = spec.kind {
        binding.depth = (max - min).abs() * 0.25;
    }
    binding
}

fn shape_picker(ui: &mut Ui, id: &str, modulator: &mut Modulator) -> bool {
    let mut changed = false;
    if let Modulator::Lfo { shape, .. } = modulator {
        ui.horizontal(|ui| {
            ComboBox::from_id_salt((id, "shape"))
                .selected_text(shape.label())
                .show_ui(ui, |ui| {
                    for option in LfoShape::ALL {
                        changed |= ui.selectable_value(shape, option, option.label()).clicked();
                    }
                });
            ui.label("Shape");
        });
    }
    changed
}

fn rate_control(ui: &mut Ui, modulator: &mut Modulator) -> bool {
    let mut changed = false;
    if let Modulator::Lfo { rate, .. } = modulator {
        ui.horizontal(|ui| {
            let mut value = *rate;
            if ui
                .add(
                    DragValue::new(&mut value)
                        .speed(0.05)
                        .range(0.01..=40.0)
                        .suffix(" Hz"),
                )
                .changed()
            {
                *rate = value;
                changed = true;
            }
            ui.label("Rate");
        });
    }
    changed
}

fn mode_picker(ui: &mut Ui, id: &str, mode: &mut ModMode) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ComboBox::from_id_salt((id, "mode"))
            .selected_text(mode.label())
            .show_ui(ui, |ui| {
                for option in ModMode::ALL {
                    changed |= ui.selectable_value(mode, option, option.label()).clicked();
                }
            });
        ui.label("Mode");
    });
    changed
}
