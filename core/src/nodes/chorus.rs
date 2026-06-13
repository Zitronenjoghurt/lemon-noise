use crate::context::Context;
use crate::node::Node;
use crate::param::{ParamKind, ParamSpec, ParamValue};
use std::f32::consts::TAU;

const BASE_DELAY_MS: f32 = 15.0;
const MAX_DELAY_MS: f32 = 40.0;

const PARAMS: &[ParamSpec] = &[
    ParamSpec {
        id: "rate",
        label: "Rate",
        description: "Speed of the pitch wobble.",
        unit: Some(" Hz"),
        kind: ParamKind::Float {
            min: 0.05,
            max: 8.0,
            default: 0.6,
            logarithmic: true,
        },
    },
    ParamSpec {
        id: "depth",
        label: "Depth",
        description: "How far the delay time sweeps, deepening the shimmer.",
        unit: None,
        kind: ParamKind::Float {
            min: 0.0,
            max: 1.0,
            default: 0.5,
            logarithmic: false,
        },
    },
    ParamSpec {
        id: "mix",
        label: "Mix",
        description: "Blend between the dry input and the chorused signal.",
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
pub struct Chorus {
    rate: f32,
    depth: f32,
    mix: f32,
    buffer: Vec<f32>,
    write: usize,
    phase: f32,
}

impl Chorus {
    pub const ID: &'static str = "chorus";
    pub const LABEL: &'static str = "Chorus";
    pub const DESCRIPTION: &'static str =
        "A modulated delay that thickens and widens the sound into a shimmering ensemble.";

    fn ensure_buffer(&mut self, sample_rate: u32) {
        let needed = (sample_rate as f32 * MAX_DELAY_MS / 1000.0).ceil() as usize + 2;
        if self.buffer.len() != needed {
            self.buffer = vec![0.0; needed.max(2)];
            self.write = 0;
        }
    }
}

impl Default for Chorus {
    fn default() -> Self {
        Self {
            rate: 0.6,
            depth: 0.5,
            mix: 0.5,
            buffer: Vec::new(),
            write: 0,
            phase: 0.0,
        }
    }
}

impl Node for Chorus {
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
            "mix" => Some(ParamValue::Float(self.mix)),
            _ => None,
        }
    }

    fn set_param(&mut self, id: &str, value: ParamValue) {
        match id {
            "rate" => self.rate = value.as_float().clamp(0.05, 8.0),
            "depth" => self.depth = value.as_float().clamp(0.0, 1.0),
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
        let lfo = (TAU * self.phase).sin();
        let delay_ms = BASE_DELAY_MS + self.depth * 7.0 * lfo;
        let delay = (delay_ms / 1000.0 * ctx.sample_rate() as f32).clamp(1.0, (len - 2) as f32);

        let read = self.write as f32 - delay + len as f32;
        let base = read.floor();
        let frac = read - base;
        let i0 = base as usize % len;
        let i1 = (i0 + 1) % len;
        let delayed = self.buffer[i0] * (1.0 - frac) + self.buffer[i1] * frac;

        self.buffer[self.write] = input;
        self.write = (self.write + 1) % len;

        input * (1.0 - self.mix) + delayed * self.mix
    }

    fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.write = 0;
        self.phase = 0.0;
    }
}
