use crate::context::Context;
use crate::node::Node;
use crate::param::{ParamKind, ParamSpec, ParamValue};
use std::f32::consts::TAU;

const PARAMS: &[ParamSpec] = &[ParamSpec {
    id: "cutoff",
    label: "Cutoff",
    description: "Frequencies above this point are progressively attenuated.",
    unit: Some(" Hz"),
    kind: ParamKind::Float {
        min: 20.0,
        max: 20_000.0,
        default: 1_000.0,
        logarithmic: true,
    },
}];

#[derive(Debug, Clone)]
pub struct LowPass {
    cutoff: f32,
    previous: f32,
}

impl LowPass {
    pub const ID: &'static str = "low_pass";
    pub const LABEL: &'static str = "Low Pass";
    pub const DESCRIPTION: &'static str =
        "One-pole low-pass filter. Removes highs, making the sound darker and smoother.";
}

impl Default for LowPass {
    fn default() -> Self {
        Self {
            cutoff: 1_000.0,
            previous: 0.0,
        }
    }
}

impl Node for LowPass {
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
            "cutoff" => Some(ParamValue::Float(self.cutoff)),
            _ => None,
        }
    }

    fn set_param(&mut self, id: &str, value: ParamValue) {
        if id == "cutoff" {
            self.cutoff = value.as_float().clamp(20.0, 20_000.0);
        }
    }

    fn process(&mut self, input: f32, ctx: &Context) -> f32 {
        let dt = ctx.dt();
        let rc = 1.0 / (TAU * self.cutoff);
        let alpha = dt / (rc + dt);
        self.previous += alpha * (input - self.previous);
        self.previous
    }

    fn reset(&mut self) {
        self.previous = 0.0;
    }
}
