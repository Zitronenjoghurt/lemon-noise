use crate::ui::icons;
use crate::ui::node_type_icon;
use crate::ui::widgets::{ModControl, ModEvent, ParamControl};
use egui::{Align, Button, Color32, Frame, Layout, Ui};
use lemon_noise::{Binding, Node, ParamKind, ParamValue, Registry};

const CARD_WIDTH: f32 = 224.0;
const WARN_COLOR: Color32 = Color32::from_rgb(230, 160, 30);

pub enum CardAction {
    Remove,
    MoveLeft,
    MoveRight,
    SetEnabled(bool),
    ChangeType(&'static str),
}

pub struct CardResponse {
    pub changes: Vec<(&'static str, ParamValue)>,
    pub mods: Vec<ModEvent>,
    pub action: Option<CardAction>,
}

pub struct NodeCard<'a> {
    node: &'a dyn Node,
    index: usize,
    count: usize,
    enabled: bool,
    bindings: &'a [Binding],
    registry: &'a Registry,
}

impl<'a> NodeCard<'a> {
    pub fn new(
        node: &'a dyn Node,
        index: usize,
        count: usize,
        enabled: bool,
        bindings: &'a [Binding],
        registry: &'a Registry,
    ) -> Self {
        Self {
            node,
            index,
            count,
            enabled,
            bindings,
            registry,
        }
    }

    pub fn show(self, ui: &mut Ui) -> CardResponse {
        let mut changes = Vec::new();
        let mut mods = Vec::new();
        let mut action = None;

        Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(CARD_WIDTH);
            ui.set_max_width(CARD_WIDTH);
            ui.push_id(self.index, |ui| {
                ui.vertical(|ui| {
                    self.header(ui, &mut action);
                    ui.separator();
                    self.params(ui, &mut changes, &mut mods);
                });
            });
        });

        CardResponse {
            changes,
            mods,
            action,
        }
    }

    fn header(&self, ui: &mut Ui, action: &mut Option<CardAction>) {
        let is_source = self.node.is_source();
        let badge_hint = if is_source {
            "Generator — creates sound and ignores the incoming signal."
        } else {
            "Processor — shapes the signal coming from the nodes before it."
        };
        let power_hint = if self.enabled {
            "Enabled — click to bypass"
        } else {
            "Bypassed — click to enable"
        };

        ui.horizontal(|ui| {
            if ui
                .selectable_label(self.enabled, node_type_icon(is_source))
                .on_hover_text(format!("{badge_hint}\n{power_hint}"))
                .clicked()
            {
                *action = Some(CardAction::SetEnabled(!self.enabled));
            }

            let title = format!("{} {}", self.node.label(), icons::CARET_DOWN);
            ui.menu_button(title, |ui| {
                for descriptor in self.registry.descriptors() {
                    let icon = node_type_icon(descriptor.is_source);
                    if ui
                        .button(format!("{icon}  {}", descriptor.label))
                        .on_hover_text(descriptor.description)
                        .clicked()
                    {
                        if descriptor.id != self.node.id() {
                            *action = Some(CardAction::ChangeType(descriptor.id));
                        }
                        ui.close();
                    }
                }
            })
            .response
            .on_hover_text(self.node.description());

            if is_source && self.index > 0 {
                ui.colored_label(WARN_COLOR, icons::WARNING).on_hover_text(
                    "This generator replaces the signal from the nodes before it. Move it to the front or onto its own channel.",
                );
            }

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.small_button(icons::X).on_hover_text("Remove").clicked() {
                    *action = Some(CardAction::Remove);
                }
                if ui
                    .add_enabled(
                        self.index + 1 < self.count,
                        Button::new(icons::ARROW_RIGHT).small(),
                    )
                    .on_hover_text("Move right")
                    .clicked()
                {
                    *action = Some(CardAction::MoveRight);
                }
                if ui
                    .add_enabled(self.index > 0, Button::new(icons::ARROW_LEFT).small())
                    .on_hover_text("Move left")
                    .clicked()
                {
                    *action = Some(CardAction::MoveLeft);
                }
            });
        });
    }

    fn params(
        &self,
        ui: &mut Ui,
        changes: &mut Vec<(&'static str, ParamValue)>,
        mods: &mut Vec<ModEvent>,
    ) {
        ui.scope(|ui| {
            if !self.enabled {
                ui.set_opacity(0.4);
            }

            for spec in self.node.params() {
                let current = self
                    .node
                    .get_param(spec.id)
                    .unwrap_or_else(|| spec.kind.default_value());
                let binding = self
                    .bindings
                    .iter()
                    .find(|b| b.node == self.index && b.param == spec.id);
                let is_float = matches!(spec.kind, ParamKind::Float { .. });

                if is_float && binding.is_some() {
                    if let Some(event) =
                        ModControl::new(spec, self.index, binding, current).show(ui)
                    {
                        mods.push(event);
                    }
                } else if is_float {
                    ui.horizontal(|ui| {
                        ui.label(spec.label).on_hover_text(spec.description);
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if let Some(event) =
                                ModControl::new(spec, self.index, None, current).show(ui)
                            {
                                mods.push(event);
                            }
                        });
                    });
                    if let Some(value) = ParamControl::new(spec, current).show(ui) {
                        changes.push((spec.id, value));
                    }
                } else if let Some(value) = ParamControl::new(spec, current).show(ui) {
                    changes.push((spec.id, value));
                }
            }
        });
    }
}
