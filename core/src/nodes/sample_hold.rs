use crate::context::Context;
use crate::node::Node;
use crate::param::{ParamKind, ParamSpec, ParamValue};

const PARAMS: &[ParamSpec] = &[ParamSpec {
    id: "rate",
    label: "Rate",
    description: "How many times per second a new input value is captured. Lower rates sound slower and grainier.",
    unit: Some(" Hz"),
    kind: ParamKind::Float {
        min: 1.0,
        max: 20_000.0,
        default: 1_000.0,
        logarithmic: true,
    },
}];

#[derive(Debug, Clone)]
pub struct SampleHold {
    rate: f32,
    phase: f32,
    held: f32,
}

impl SampleHold {
    pub const ID: &'static str = "sample_hold";
    pub const LABEL: &'static str = "Sample & Hold";
    pub const DESCRIPTION: &'static str = "Freezes the input and only refreshes it at the chosen rate. A lower rate makes the texture coarser and slower.";
}

impl Default for SampleHold {
    fn default() -> Self {
        Self {
            rate: 1_000.0,
            phase: 1.0,
            held: 0.0,
        }
    }
}

impl Node for SampleHold {
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
            "rate" => Some(ParamValue::Float(self.rate)),
            _ => None,
        }
    }

    fn set_param(&mut self, id: &str, value: ParamValue) {
        if id == "rate" {
            self.rate = value.as_float().clamp(1.0, 20_000.0);
        }
    }

    fn process(&mut self, input: f32, ctx: &Context) -> f32 {
        self.phase += self.rate * ctx.dt();
        if self.phase >= 1.0 {
            self.phase -= self.phase.floor();
            self.held = input;
        }
        self.held
    }

    fn reset(&mut self) {
        self.phase = 1.0;
        self.held = 0.0;
    }
}
