use crate::context::Context;
use crate::node::Node;
use crate::param::{ParamKind, ParamSpec, ParamValue};

const PARAMS: &[ParamSpec] = &[ParamSpec {
    id: "gain",
    label: "Gain",
    description: "Linear multiplier applied to the incoming signal.",
    unit: Some("x"),
    kind: ParamKind::Float {
        min: 0.0,
        max: 4.0,
        default: 1.0,
        logarithmic: false,
    },
}];

#[derive(Debug, Clone)]
pub struct Gain {
    gain: f32,
}

impl Gain {
    pub const ID: &'static str = "gain";
    pub const LABEL: &'static str = "Gain";
    pub const DESCRIPTION: &'static str = "Scales the signal level up or down.";
}

impl Default for Gain {
    fn default() -> Self {
        Self { gain: 1.0 }
    }
}

impl Node for Gain {
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
            "gain" => Some(ParamValue::Float(self.gain)),
            _ => None,
        }
    }

    fn set_param(&mut self, id: &str, value: ParamValue) {
        if id == "gain" {
            self.gain = value.as_float().clamp(0.0, 4.0);
        }
    }

    fn process(&mut self, input: f32, _ctx: &Context) -> f32 {
        input * self.gain
    }
}
