use crate::context::Context;
use crate::node::Node;
use crate::param::{ParamKind, ParamSpec, ParamValue};

const COMB_TUNING: [usize; 8] = [1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617];
const ALLPASS_TUNING: [usize; 4] = [556, 441, 341, 225];
const REFERENCE_RATE: f32 = 44_100.0;
const FIXED_GAIN: f32 = 0.015;
const WET_SCALE: f32 = 3.0;

const PARAMS: &[ParamSpec] = &[
    ParamSpec {
        id: "size",
        label: "Size",
        description: "Length of the tail. Higher values sound like a larger space.",
        unit: None,
        kind: ParamKind::Float {
            min: 0.0,
            max: 1.0,
            default: 0.7,
            logarithmic: false,
        },
    },
    ParamSpec {
        id: "damping",
        label: "Damping",
        description: "How quickly the high frequencies fade from the tail.",
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
        description: "Blend between the dry input and the reverberated signal.",
        unit: None,
        kind: ParamKind::Float {
            min: 0.0,
            max: 1.0,
            default: 0.3,
            logarithmic: false,
        },
    },
];

#[derive(Debug, Clone)]
struct Comb {
    buffer: Vec<f32>,
    index: usize,
    store: f32,
}

impl Comb {
    fn new(len: usize) -> Self {
        Self {
            buffer: vec![0.0; len.max(1)],
            index: 0,
            store: 0.0,
        }
    }

    fn process(&mut self, input: f32, feedback: f32, damp1: f32, damp2: f32) -> f32 {
        let output = self.buffer[self.index];
        self.store = output * damp2 + self.store * damp1;
        self.buffer[self.index] = input + self.store * feedback;
        self.index = (self.index + 1) % self.buffer.len();
        output
    }
}

#[derive(Debug, Clone)]
struct Allpass {
    buffer: Vec<f32>,
    index: usize,
}

impl Allpass {
    fn new(len: usize) -> Self {
        Self {
            buffer: vec![0.0; len.max(1)],
            index: 0,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let buffered = self.buffer[self.index];
        let output = -input + buffered;
        self.buffer[self.index] = input + buffered * 0.5;
        self.index = (self.index + 1) % self.buffer.len();
        output
    }
}

#[derive(Debug, Clone)]
pub struct Reverb {
    size: f32,
    damping: f32,
    mix: f32,
    combs: Vec<Comb>,
    allpasses: Vec<Allpass>,
    sample_rate: u32,
}

impl Reverb {
    pub const ID: &'static str = "reverb";
    pub const LABEL: &'static str = "Reverb";
    pub const DESCRIPTION: &'static str = "A spacious tail built from parallel combs and allpass filters. Turns dry noise into a room.";

    fn ensure_buffers(&mut self, sample_rate: u32) {
        if self.sample_rate == sample_rate && !self.combs.is_empty() {
            return;
        }
        let scale = sample_rate as f32 / REFERENCE_RATE;
        self.combs = COMB_TUNING
            .iter()
            .map(|&tuning| Comb::new((tuning as f32 * scale) as usize))
            .collect();
        self.allpasses = ALLPASS_TUNING
            .iter()
            .map(|&tuning| Allpass::new((tuning as f32 * scale) as usize))
            .collect();
        self.sample_rate = sample_rate;
    }
}

impl Default for Reverb {
    fn default() -> Self {
        Self {
            size: 0.7,
            damping: 0.5,
            mix: 0.3,
            combs: Vec::new(),
            allpasses: Vec::new(),
            sample_rate: 0,
        }
    }
}

impl Node for Reverb {
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
            "size" => Some(ParamValue::Float(self.size)),
            "damping" => Some(ParamValue::Float(self.damping)),
            "mix" => Some(ParamValue::Float(self.mix)),
            _ => None,
        }
    }

    fn set_param(&mut self, id: &str, value: ParamValue) {
        match id {
            "size" => self.size = value.as_float().clamp(0.0, 1.0),
            "damping" => self.damping = value.as_float().clamp(0.0, 1.0),
            "mix" => self.mix = value.as_float().clamp(0.0, 1.0),
            _ => {}
        }
    }

    fn process(&mut self, input: f32, ctx: &Context) -> f32 {
        self.ensure_buffers(ctx.sample_rate());

        let feedback = self.size * 0.28 + 0.7;
        let damp1 = self.damping * 0.4;
        let damp2 = 1.0 - damp1;
        let fed = input * FIXED_GAIN;

        let mut wet = 0.0;
        for comb in &mut self.combs {
            wet += comb.process(fed, feedback, damp1, damp2);
        }
        for allpass in &mut self.allpasses {
            wet = allpass.process(wet);
        }

        let wet = (wet * WET_SCALE).clamp(-4.0, 4.0);
        input * (1.0 - self.mix) + wet * self.mix
    }

    fn reset(&mut self) {
        for comb in &mut self.combs {
            comb.buffer.fill(0.0);
            comb.index = 0;
            comb.store = 0.0;
        }
        for allpass in &mut self.allpasses {
            allpass.buffer.fill(0.0);
            allpass.index = 0;
        }
    }
}
