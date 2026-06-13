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
pub struct Pink {
    amplitude: f32,
    seed: u64,
    rng: Rng,
    state: [f32; 7],
}

impl Pink {
    pub const ID: &'static str = "pink";
    pub const LABEL: &'static str = "Pink Noise";
    pub const DESCRIPTION: &'static str =
        "Equal energy per octave (-3 dB/octave). Warmer and more natural than white noise.";
}

impl Default for Pink {
    fn default() -> Self {
        Self {
            amplitude: 0.5,
            seed: 1,
            rng: Rng::new(1),
            state: [0.0; 7],
        }
    }
}

impl Node for Pink {
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
            }
            _ => {}
        }
    }

    fn process(&mut self, _input: f32, _ctx: &Context) -> f32 {
        let white = self.rng.next_bipolar();
        let s = &mut self.state;
        s[0] = 0.99886 * s[0] + white * 0.0555179;
        s[1] = 0.99332 * s[1] + white * 0.0750759;
        s[2] = 0.969 * s[2] + white * 0.153852;
        s[3] = 0.8665 * s[3] + white * 0.3104856;
        s[4] = 0.55 * s[4] + white * 0.5329522;
        s[5] = -0.7616 * s[5] - white * 0.016898;
        let pink = s[0] + s[1] + s[2] + s[3] + s[4] + s[5] + s[6] + white * 0.5362;
        s[6] = white * 0.115926;
        pink * 0.11 * self.amplitude
    }

    fn reset(&mut self) {
        self.rng = Rng::new(self.seed);
        self.state = [0.0; 7];
    }
}
