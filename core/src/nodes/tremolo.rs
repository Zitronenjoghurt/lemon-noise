use crate::context::Context;
use crate::node::Node;
use crate::param::{ParamKind, ParamSpec, ParamValue};
use std::f32::consts::TAU;

const PARAMS: &[ParamSpec] = &[
    ParamSpec {
        id: "rate",
        label: "Rate",
        description: "Speed of the volume pulsing.",
        unit: Some(" Hz"),
        kind: ParamKind::Float {
            min: 0.05,
            max: 40.0,
            default: 4.0,
            logarithmic: true,
        },
    },
    ParamSpec {
        id: "depth",
        label: "Depth",
        description: "How much the volume drops at the bottom of each cycle.",
        unit: None,
        kind: ParamKind::Float {
            min: 0.0,
            max: 1.0,
            default: 0.5,
            logarithmic: false,
        },
    },
];

#[derive(Debug, Clone)]
pub struct Tremolo {
    rate: f32,
    depth: f32,
    phase: f32,
}

impl Tremolo {
    pub const ID: &'static str = "tremolo";
    pub const LABEL: &'static str = "Tremolo";
    pub const DESCRIPTION: &'static str =
        "Modulates the volume with a sine LFO, adding a rhythmic pulsing or breathing motion.";
}

impl Default for Tremolo {
    fn default() -> Self {
        Self {
            rate: 4.0,
            depth: 0.5,
            phase: 0.0,
        }
    }
}

impl Node for Tremolo {
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
            "depth" => Some(ParamValue::Float(self.depth)),
            _ => None,
        }
    }

    fn set_param(&mut self, id: &str, value: ParamValue) {
        match id {
            "rate" => self.rate = value.as_float().clamp(0.05, 40.0),
            "depth" => self.depth = value.as_float().clamp(0.0, 1.0),
            _ => {}
        }
    }

    fn process(&mut self, input: f32, ctx: &Context) -> f32 {
        self.phase += self.rate * ctx.dt();
        if self.phase >= 1.0 {
            self.phase -= self.phase.floor();
        }
        let lfo = 0.5 + 0.5 * (TAU * self.phase).sin();
        let gain = 1.0 - self.depth + self.depth * lfo;
        input * gain
    }

    fn reset(&mut self) {
        self.phase = 0.0;
    }
}
