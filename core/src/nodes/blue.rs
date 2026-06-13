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
pub struct Blue {
    amplitude: f32,
    seed: u64,
    rng: Rng,
    previous: f32,
}

impl Blue {
    pub const ID: &'static str = "blue_noise";
    pub const LABEL: &'static str = "Blue Noise";
    pub const DESCRIPTION: &'static str = "White noise tilted toward the highs. Brighter and airier than white, good for hiss and spray.";
}

impl Default for Blue {
    fn default() -> Self {
        Self {
            amplitude: 0.5,
            seed: 1,
            rng: Rng::new(1),
            previous: 0.0,
        }
    }
}

impl Node for Blue {
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
            "seed" => Some(ParamValue::Int(self.seed as i64)),
            _ => None,
        }
    }

    fn set_param(&mut self, id: &str, value: ParamValue) {
        match id {
            "amplitude" => self.amplitude = value.as_float().clamp(0.0, 1.0),
            "seed" => {
                self.seed = value.as_int().max(0) as u64;
                self.rng = Rng::new(self.seed);
                self.previous = 0.0;
            }
            _ => {}
        }
    }

    fn process(&mut self, _input: f32, _ctx: &Context) -> f32 {
        let white = self.rng.next_bipolar();
        let blue = (white - self.previous) * 0.5;
        self.previous = white;
        blue * self.amplitude
    }

    fn reset(&mut self) {
        self.rng = Rng::new(self.seed);
        self.previous = 0.0;
    }
}
