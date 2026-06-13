use crate::context::Context;
use crate::node::Node;
use crate::param::{ParamKind, ParamSpec, ParamValue};
use std::f32::consts::TAU;

const PARAMS: &[ParamSpec] = &[ParamSpec {
    id: "cutoff",
    label: "Cutoff",
    description: "Frequencies below this point are progressively attenuated.",
    unit: Some(" Hz"),
    kind: ParamKind::Float {
        min: 20.0,
        max: 20_000.0,
        default: 500.0,
        logarithmic: true,
    },
}];

#[derive(Debug, Clone)]
pub struct HighPass {
    cutoff: f32,
    prev_input: f32,
    prev_output: f32,
}

impl HighPass {
    pub const ID: &'static str = "high_pass";
    pub const LABEL: &'static str = "High Pass";
    pub const DESCRIPTION: &'static str =
        "One-pole high-pass filter. Removes lows, thinning the sound and cutting rumble.";
}

impl Default for HighPass {
    fn default() -> Self {
        Self {
            cutoff: 500.0,
            prev_input: 0.0,
            prev_output: 0.0,
        }
    }
}

impl Node for HighPass {
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
        let alpha = rc / (rc + dt);
        let output = alpha * (self.prev_output + input - self.prev_input);
        self.prev_input = input;
        self.prev_output = output;
        output
    }

    fn reset(&mut self) {
        self.prev_input = 0.0;
        self.prev_output = 0.0;
    }
}
