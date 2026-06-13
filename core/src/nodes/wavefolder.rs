use crate::context::Context;
use crate::node::Node;
use crate::param::{ParamKind, ParamSpec, ParamValue};

const PARAMS: &[ParamSpec] = &[
    ParamSpec {
        id: "drive",
        label: "Drive",
        description: "How hard the signal is pushed into the folds. More drive means more harmonics.",
        unit: None,
        kind: ParamKind::Float {
            min: 1.0,
            max: 10.0,
            default: 2.0,
            logarithmic: false,
        },
    },
    ParamSpec {
        id: "mix",
        label: "Mix",
        description: "Blend between the dry input and the folded signal.",
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
pub struct Wavefolder {
    drive: f32,
    mix: f32,
}

impl Wavefolder {
    pub const ID: &'static str = "wavefolder";
    pub const LABEL: &'static str = "Wavefolder";
    pub const DESCRIPTION: &'static str =
        "Folds peaks back on themselves instead of clipping, adding bright metallic harmonics.";
}

impl Default for Wavefolder {
    fn default() -> Self {
        Self {
            drive: 2.0,
            mix: 1.0,
        }
    }
}

fn fold(mut value: f32) -> f32 {
    for _ in 0..8 {
        if value > 1.0 {
            value = 2.0 - value;
        } else if value < -1.0 {
            value = -2.0 - value;
        } else {
            break;
        }
    }
    value.clamp(-1.0, 1.0)
}

impl Node for Wavefolder {
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
            "drive" => Some(ParamValue::Float(self.drive)),
            "mix" => Some(ParamValue::Float(self.mix)),
            _ => None,
        }
    }

    fn set_param(&mut self, id: &str, value: ParamValue) {
        match id {
            "drive" => self.drive = value.as_float().clamp(1.0, 10.0),
            "mix" => self.mix = value.as_float().clamp(0.0, 1.0),
            _ => {}
        }
    }

    fn process(&mut self, input: f32, _ctx: &Context) -> f32 {
        let folded = fold(input * self.drive);
        input * (1.0 - self.mix) + folded * self.mix
    }
}
