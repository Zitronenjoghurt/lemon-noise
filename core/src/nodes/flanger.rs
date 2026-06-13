use crate::context::Context;
use crate::node::Node;
use crate::param::{ParamKind, ParamSpec, ParamValue};
use std::f32::consts::TAU;

const MAX_DELAY_MS: f32 = 10.0;

const PARAMS: &[ParamSpec] = &[
    ParamSpec {
        id: "rate",
        label: "Rate",
        description: "Speed of the sweep.",
        unit: Some(" Hz"),
        kind: ParamKind::Float {
            min: 0.05,
            max: 8.0,
            default: 0.3,
            logarithmic: true,
        },
    },
    ParamSpec {
        id: "depth",
        label: "Depth",
        description: "How far the short delay sweeps, widening the jet-like comb.",
        unit: None,
        kind: ParamKind::Float {
            min: 0.0,
            max: 1.0,
            default: 0.6,
            logarithmic: false,
        },
    },
    ParamSpec {
        id: "feedback",
        label: "Feedback",
        description: "How much of the swept signal is fed back, sharpening the resonance.",
        unit: None,
        kind: ParamKind::Float {
            min: 0.0,
            max: 0.95,
            default: 0.5,
            logarithmic: false,
        },
    },
    ParamSpec {
        id: "mix",
        label: "Mix",
        description: "Blend between the dry input and the flanged signal.",
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
pub struct Flanger {
    rate: f32,
    depth: f32,
    feedback: f32,
    mix: f32,
    buffer: Vec<f32>,
    write: usize,
    phase: f32,
}

impl Flanger {
    pub const ID: &'static str = "flanger";
    pub const LABEL: &'static str = "Flanger";
    pub const DESCRIPTION: &'static str =
        "A short swept delay with feedback, producing the classic jet-plane whoosh.";

    fn ensure_buffer(&mut self, sample_rate: u32) {
        let needed = (sample_rate as f32 * MAX_DELAY_MS / 1000.0).ceil() as usize + 2;
        if self.buffer.len() != needed {
            self.buffer = vec![0.0; needed.max(2)];
            self.write = 0;
        }
    }
}

impl Default for Flanger {
    fn default() -> Self {
        Self {
            rate: 0.3,
            depth: 0.6,
            feedback: 0.5,
            mix: 0.5,
            buffer: Vec::new(),
            write: 0,
            phase: 0.0,
        }
    }
}

impl Node for Flanger {
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
            "feedback" => Some(ParamValue::Float(self.feedback)),
            "mix" => Some(ParamValue::Float(self.mix)),
            _ => None,
        }
    }

    fn set_param(&mut self, id: &str, value: ParamValue) {
        match id {
            "rate" => self.rate = value.as_float().clamp(0.05, 8.0),
            "depth" => self.depth = value.as_float().clamp(0.0, 1.0),
            "feedback" => self.feedback = value.as_float().clamp(0.0, 0.95),
            "mix" => self.mix = value.as_float().clamp(0.0, 1.0),
            _ => {}
        }
    }

    fn process(&mut self, input: f32, ctx: &Context) -> f32 {
        self.ensure_buffer(ctx.sample_rate());
        let len = self.buffer.len();

        self.phase += self.rate * ctx.dt();
        if self.phase >= 1.0 {
            self.phase -= self.phase.floor();
        }
        let lfo = 0.5 + 0.5 * (TAU * self.phase).sin();
        let delay_ms = 1.0 + self.depth * 4.0 * lfo;
        let delay = (delay_ms / 1000.0 * ctx.sample_rate() as f32).clamp(1.0, (len - 2) as f32);

        let read = self.write as f32 - delay + len as f32;
        let base = read.floor();
        let frac = read - base;
        let i0 = base as usize % len;
        let i1 = (i0 + 1) % len;
        let delayed = self.buffer[i0] * (1.0 - frac) + self.buffer[i1] * frac;

        self.buffer[self.write] = (input + self.feedback * delayed).clamp(-4.0, 4.0);
        self.write = (self.write + 1) % len;

        input * (1.0 - self.mix) + delayed * self.mix
    }

    fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.write = 0;
        self.phase = 0.0;
    }
}
