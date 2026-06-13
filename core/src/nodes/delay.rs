use crate::context::Context;
use crate::node::Node;
use crate::param::{ParamKind, ParamSpec, ParamValue};

const MAX_TIME_MS: f32 = 2_000.0;

const PARAMS: &[ParamSpec] = &[
    ParamSpec {
        id: "time",
        label: "Time",
        description: "Delay length before each echo repeats.",
        unit: Some(" ms"),
        kind: ParamKind::Float {
            min: 1.0,
            max: MAX_TIME_MS,
            default: 250.0,
            logarithmic: true,
        },
    },
    ParamSpec {
        id: "feedback",
        label: "Feedback",
        description: "How much of each echo is fed back in. High values trail off slowly.",
        unit: None,
        kind: ParamKind::Float {
            min: 0.0,
            max: 0.95,
            default: 0.4,
            logarithmic: false,
        },
    },
    ParamSpec {
        id: "mix",
        label: "Mix",
        description: "Blend between the dry input and the delayed signal.",
        unit: None,
        kind: ParamKind::Float {
            min: 0.0,
            max: 1.0,
            default: 0.4,
            logarithmic: false,
        },
    },
];

#[derive(Debug, Clone)]
pub struct Delay {
    time_ms: f32,
    feedback: f32,
    mix: f32,
    buffer: Vec<f32>,
    write: usize,
}

impl Delay {
    pub const ID: &'static str = "delay";
    pub const LABEL: &'static str = "Delay";
    pub const DESCRIPTION: &'static str =
        "An echo line with feedback. Adds space, rhythm and slap to any source.";

    fn ensure_buffer(&mut self, sample_rate: u32) {
        let needed = (sample_rate as f32 * MAX_TIME_MS / 1000.0).ceil() as usize + 1;
        if self.buffer.len() != needed {
            self.buffer = vec![0.0; needed.max(1)];
            self.write = 0;
        }
    }
}

impl Default for Delay {
    fn default() -> Self {
        Self {
            time_ms: 250.0,
            feedback: 0.4,
            mix: 0.4,
            buffer: Vec::new(),
            write: 0,
        }
    }
}

impl Node for Delay {
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
            "time" => Some(ParamValue::Float(self.time_ms)),
            "feedback" => Some(ParamValue::Float(self.feedback)),
            "mix" => Some(ParamValue::Float(self.mix)),
            _ => None,
        }
    }

    fn set_param(&mut self, id: &str, value: ParamValue) {
        match id {
            "time" => self.time_ms = value.as_float().clamp(1.0, MAX_TIME_MS),
            "feedback" => self.feedback = value.as_float().clamp(0.0, 0.95),
            "mix" => self.mix = value.as_float().clamp(0.0, 1.0),
            _ => {}
        }
    }

    fn process(&mut self, input: f32, ctx: &Context) -> f32 {
        self.ensure_buffer(ctx.sample_rate());
        let len = self.buffer.len();
        if len == 0 {
            return input;
        }

        let delay = (ctx.sample_rate() as f32 * self.time_ms / 1000.0)
            .round()
            .clamp(1.0, (len - 1) as f32) as usize;
        let read = (self.write + len - delay) % len;
        let delayed = self.buffer[read];

        self.buffer[self.write] = (input + self.feedback * delayed).clamp(-4.0, 4.0);
        self.write = (self.write + 1) % len;

        input * (1.0 - self.mix) + delayed * self.mix
    }

    fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.write = 0;
    }
}
