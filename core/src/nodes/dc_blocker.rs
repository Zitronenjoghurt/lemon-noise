use crate::context::Context;
use crate::node::Node;
use crate::param::{ParamKind, ParamSpec, ParamValue};

const PARAMS: &[ParamSpec] = &[ParamSpec {
    id: "strength",
    label: "Strength",
    description: "How aggressively low-frequency drift and offset are removed.",
    unit: None,
    kind: ParamKind::Float {
        min: 0.9,
        max: 0.9999,
        default: 0.995,
        logarithmic: false,
    },
}];

#[derive(Debug, Clone)]
pub struct DcBlocker {
    strength: f32,
    previous_input: f32,
    previous_output: f32,
}

impl DcBlocker {
    pub const ID: &'static str = "dc_blocker";
    pub const LABEL: &'static str = "DC Blocker";
    pub const DESCRIPTION: &'static str =
        "Removes constant offset and slow drift so the signal stays centered and clean.";
}

impl Default for DcBlocker {
    fn default() -> Self {
        Self {
            strength: 0.995,
            previous_input: 0.0,
            previous_output: 0.0,
        }
    }
}

impl Node for DcBlocker {
    fn id(&self) -> &'static str {
        Self::ID
    }

    fn label(&self) -> &'static str {
        Self::LABEL
    }

    fn description(&self) -> &'static str {
        Self::DESCRIPTION
    }

    fn params(&self) -> &'static [ParamSpec] {
        PARAMS
    }

    fn get_param(&self, id: &str) -> Option<ParamValue> {
        match id {
            "strength" => Some(ParamValue::Float(self.strength)),
            _ => None,
        }
    }

    fn set_param(&mut self, id: &str, value: ParamValue) {
        if id == "strength" {
            self.strength = value.as_float().clamp(0.9, 0.9999);
        }
    }

    fn process(&mut self, input: f32, _ctx: &Context) -> f32 {
        let output = input - self.previous_input + self.strength * self.previous_output;
        self.previous_input = input;
        self.previous_output = output;
        output
    }

    fn reset(&mut self) {
        self.previous_input = 0.0;
        self.previous_output = 0.0;
    }
}
