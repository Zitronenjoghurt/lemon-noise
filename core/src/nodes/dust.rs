use crate::context::Context;
use crate::node::Node;
use crate::param::{ParamKind, ParamSpec, ParamValue};
use crate::rng::Rng;

const PARAMS: &[ParamSpec] = &[
    ParamSpec {
        id: "density",
        label: "Density",
        description: "Average number of impulses per second. Low values crackle, high values fizz.",
        unit: Some(" Hz"),
        kind: ParamKind::Float {
            min: 1.0,
            max: 5_000.0,
            default: 200.0,
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
pub struct Dust {
    density: f32,
    amplitude: f32,
    seed: u64,
    rng: Rng,
}

impl Dust {
    pub const ID: &'static str = "dust";
    pub const LABEL: &'static str = "Dust";
    pub const DESCRIPTION: &'static str =
        "Sparse random impulses. The raw material for rain, crackle, geiger ticks and vinyl dust.";
}

impl Default for Dust {
    fn default() -> Self {
        Self {
            density: 200.0,
            amplitude: 0.8,
            seed: 1,
            rng: Rng::new(1),
        }
    }
}

impl Node for Dust {
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
            "density" => Some(ParamValue::Float(self.density)),
            "amplitude" => Some(ParamValue::Float(self.amplitude)),
            "seed" => Some(ParamValue::Int(self.seed as i64)),
            _ => None,
        }
    }

    fn set_param(&mut self, id: &str, value: ParamValue) {
        match id {
            "density" => self.density = value.as_float().clamp(1.0, 5_000.0),
            "amplitude" => self.amplitude = value.as_float().clamp(0.0, 1.0),
            "seed" => {
                self.seed = value.as_int().max(0) as u64;
                self.rng = Rng::new(self.seed);
            }
            _ => {}
        }
    }

    fn process(&mut self, _input: f32, ctx: &Context) -> f32 {
        let probability = (self.density * ctx.dt()).min(1.0);
        if self.rng.next_unipolar() < probability {
            self.rng.next_bipolar() * self.amplitude
        } else {
            0.0
        }
    }

    fn reset(&mut self) {
        self.rng = Rng::new(self.seed);
    }
}
