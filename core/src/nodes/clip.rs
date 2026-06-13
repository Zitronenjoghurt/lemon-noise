use crate::context::Context;
use crate::node::Node;
use crate::param::{ParamKind, ParamSpec, ParamValue};

const PARAMS: &[ParamSpec] = &[ParamSpec {
    id: "threshold",
    label: "Threshold",
    description: "Level at which the signal is hard-clipped. Lower values clip more aggressively.",
    unit: None,
    kind: ParamKind::Float {
        min: 0.05,
        max: 1.0,
        default: 0.5,
        logarithmic: false,
    },
}];

#[derive(Debug, Clone)]
pub struct Clip {
    threshold: f32,
}

impl Clip {
    pub const ID: &'static str = "clip";
    pub const LABEL: &'static str = "Hard Clip";
    pub const DESCRIPTION: &'static str =
        "Clamps the signal to a threshold, squaring off peaks for harsh, aggressive distortion.";
}

impl Default for Clip {
    fn default() -> Self {
        Self { threshold: 0.5 }
    }
}

impl Node for Clip {
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
            "threshold" => Some(ParamValue::Float(self.threshold)),
            _ => None,
        }
    }

    fn set_param(&mut self, id: &str, value: ParamValue) {
        if id == "threshold" {
            self.threshold = value.as_float().clamp(0.05, 1.0);
        }
    }

    fn process(&mut self, input: f32, _ctx: &Context) -> f32 {
        input.clamp(-self.threshold, self.threshold)
    }
}
