use crate::context::Context;
use crate::node::Node;
use crate::param::{ParamKind, ParamSpec, ParamValue};

const PARAMS: &[ParamSpec] = &[
    ParamSpec {
        id: "rate",
        label: "Rate",
        description: "How many impulses fire per second.",
        unit: Some(" Hz"),
        kind: ParamKind::Float {
            min: 0.1,
            max: 1_000.0,
            default: 4.0,
            logarithmic: true,
        },
    },
    ParamSpec {
        id: "amplitude",
        label: "Amplitude",
        description: "Peak level of each impulse.",
        unit: None,
        kind: ParamKind::Float {
            min: 0.0,
            max: 1.0,
            default: 0.8,
            logarithmic: false,
        },
    },
];

#[derive(Debug, Clone)]
pub struct Pulse {
    rate: f32,
    amplitude: f32,
    phase: f32,
}

impl Pulse {
    pub const ID: &'static str = "pulse";
    pub const LABEL: &'static str = "Pulse";
    pub const DESCRIPTION: &'static str =
        "A steady train of single-sample impulses. Use it for clicks, ticks and rhythmic clocking.";
}

impl Default for Pulse {
    fn default() -> Self {
        Self {
            rate: 4.0,
            amplitude: 0.8,
            phase: 1.0,
        }
    }
}

impl Node for Pulse {
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

    fn is_source(&self) -> bool {
        true
    }

    fn get_param(&self, id: &str) -> Option<ParamValue> {
        match id {
            "rate" => Some(ParamValue::Float(self.rate)),
            "amplitude" => Some(ParamValue::Float(self.amplitude)),
            _ => None,
        }
    }

    fn set_param(&mut self, id: &str, value: ParamValue) {
        match id {
            "rate" => self.rate = value.as_float().clamp(0.1, 1_000.0),
            "amplitude" => self.amplitude = value.as_float().clamp(0.0, 1.0),
            _ => {}
        }
    }

    fn process(&mut self, _input: f32, ctx: &Context) -> f32 {
        self.phase += self.rate * ctx.dt();
        if self.phase >= 1.0 {
            self.phase -= self.phase.floor();
            self.amplitude
        } else {
            0.0
        }
    }

    fn reset(&mut self) {
        self.phase = 1.0;
    }
}
