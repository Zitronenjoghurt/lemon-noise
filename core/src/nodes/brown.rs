use crate::context::Context;
use crate::node::Node;
use crate::param::{ParamKind, ParamSpec, ParamValue};
use crate::rng::Rng;

const PARAMS: &[ParamSpec] = &[
    ParamSpec {
        id: "amplitude",
        label: "Amplitude",
        description: "Output level of the generated noise.",
        unit: None,
        kind: ParamKind::Float {
            min: 0.0,
            max: 1.0,
            default: 0.5,
            logarithmic: false,
        },
    },
    ParamSpec {
        id: "step",
        label: "Step",
        description: "How far each sample can wander from the last. Larger values sound brighter and rougher.",
        unit: None,
        kind: ParamKind::Float {
            min: 0.001,
            max: 0.5,
            default: 0.05,
            logarithmic: true,
        },
    },
    ParamSpec {
        id: "seed",
        label: "Seed",
        description: "Seed for the random generator. The same seed always produces the same sequence.",
        unit: None,
        kind: ParamKind::Int {
            min: 0,
            max: i64::MAX,
            default: 1,
        },
    },
];

#[derive(Debug, Clone)]
pub struct Brown {
    amplitude: f32,
    step: f32,
    seed: u64,
    rng: Rng,
    state: f32,
}

impl Brown {
    pub const ID: &'static str = "brown";
    pub const LABEL: &'static str = "Brown Noise";
    pub const DESCRIPTION: &'static str =
        "A random walk with strong low-frequency energy. Deep and rumbling, like distant surf.";
}

impl Default for Brown {
    fn default() -> Self {
        Self {
            amplitude: 0.5,
            step: 0.05,
            seed: 1,
            rng: Rng::new(1),
            state: 0.0,
        }
    }
}

impl Node for Brown {
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
            "amplitude" => Some(ParamValue::Float(self.amplitude)),
            "step" => Some(ParamValue::Float(self.step)),
            "seed" => Some(ParamValue::Int(self.seed as i64)),
            _ => None,
        }
    }

    fn set_param(&mut self, id: &str, value: ParamValue) {
        match id {
            "amplitude" => self.amplitude = value.as_float().clamp(0.0, 1.0),
            "step" => self.step = value.as_float().clamp(0.001, 0.5),
            "seed" => {
                self.seed = value.as_int().max(0) as u64;
                self.rng = Rng::new(self.seed);
            }
            _ => {}
        }
    }

    fn process(&mut self, _input: f32, _ctx: &Context) -> f32 {
        self.state += self.rng.next_bipolar() * self.step;
        self.state = self.state.clamp(-1.0, 1.0);
        self.state * self.amplitude
    }

    fn reset(&mut self) {
        self.rng = Rng::new(self.seed);
        self.state = 0.0;
    }
}
