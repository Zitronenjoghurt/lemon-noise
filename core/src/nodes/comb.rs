use crate::context::Context;
use crate::node::Node;
use crate::param::{ParamKind, ParamSpec, ParamValue};

const MIN_FREQUENCY: f32 = 20.0;

const PARAMS: &[ParamSpec] = &[
    ParamSpec {
        id: "frequency",
        label: "Frequency",
        description: "Pitch the comb resonates at. Turns broadband noise into a tuned tone.",
        unit: Some(" Hz"),
        kind: ParamKind::Float {
            min: MIN_FREQUENCY,
            max: 4_000.0,
            default: 220.0,
            logarithmic: true,
        },
    },
    ParamSpec {
        id: "feedback",
        label: "Resonance",
        description: "How strongly the comb rings. High values give a sharp, whistling pitch.",
        unit: None,
        kind: ParamKind::Float {
            min: 0.0,
            max: 0.98,
            default: 0.8,
            logarithmic: false,
        },
    },
    ParamSpec {
        id: "mix",
        label: "Mix",
        description: "Blend between the dry input and the resonated signal.",
        unit: None,
        kind: ParamKind::Float {
            min: 0.0,
            max: 1.0,
            default: 1.0,
            logarithmic: false,
        },
    },
];

#[derive(Debug, Clone)]
pub struct Comb {
    frequency: f32,
    feedback: f32,
    mix: f32,
    buffer: Vec<f32>,
    write: usize,
}

impl Comb {
    pub const ID: &'static str = "comb";
    pub const LABEL: &'static str = "Comb Resonator";
    pub const DESCRIPTION: &'static str =
        "A tuned feedback comb filter. Feed it noise to pull a pitched tone out of the texture.";

    fn ensure_buffer(&mut self, sample_rate: u32) {
        let needed = (sample_rate as f32 / MIN_FREQUENCY).ceil() as usize + 1;
        if self.buffer.len() != needed {
            self.buffer = vec![0.0; needed.max(1)];
            self.write = 0;
        }
    }
}

impl Default for Comb {
    fn default() -> Self {
        Self {
            frequency: 220.0,
            feedback: 0.8,
            mix: 1.0,
            buffer: Vec::new(),
            write: 0,
        }
    }
}

impl Node for Comb {
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
            "frequency" => Some(ParamValue::Float(self.frequency)),
            "feedback" => Some(ParamValue::Float(self.feedback)),
            "mix" => Some(ParamValue::Float(self.mix)),
            _ => None,
        }
    }

    fn set_param(&mut self, id: &str, value: ParamValue) {
        match id {
            "frequency" => self.frequency = value.as_float().clamp(MIN_FREQUENCY, 4_000.0),
            "feedback" => self.feedback = value.as_float().clamp(0.0, 0.98),
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

        let delay = (ctx.sample_rate() as f32 / self.frequency)
            .round()
            .clamp(1.0, (len - 1) as f32) as usize;
        let read = (self.write + len - delay) % len;
        let delayed = self.buffer[read];

        let resonated = input + self.feedback * delayed;
        let resonated = resonated.clamp(-4.0, 4.0);
        self.buffer[self.write] = resonated;
        self.write = (self.write + 1) % len;

        let wet = resonated * (1.0 - 0.5 * self.feedback);
        input * (1.0 - self.mix) + wet * self.mix
    }

    fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.write = 0;
    }
}
