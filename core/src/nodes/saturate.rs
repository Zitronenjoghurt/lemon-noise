use crate::context::Context;
use crate::node::Node;
use crate::param::{ParamKind, ParamSpec, ParamValue};

const PARAMS: &[ParamSpec] = &[ParamSpec {
    id: "drive",
    label: "Drive",
    description: "How hard the signal is pushed into the soft-clipping curve. Higher values add warmth then crunch.",
    unit: Some("x"),
    kind: ParamKind::Float {
        min: 1.0,
        max: 20.0,
        default: 1.0,
        logarithmic: true,
    },
}];

#[derive(Debug, Clone)]
pub struct Saturate {
    drive: f32,
}

impl Saturate {
    pub const ID: &'static str = "saturate";
    pub const LABEL: &'static str = "Saturate";
    pub const DESCRIPTION: &'static str =
        "Soft-clips the signal with a tanh curve, rounding off peaks and adding harmonics.";
}

impl Default for Saturate {
    fn default() -> Self {
        Self { drive: 1.0 }
    }
}

impl Node for Saturate {
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
            _ => None,
        }
    }

    fn set_param(&mut self, id: &str, value: ParamValue) {
        if id == "drive" {
            self.drive = value.as_float().clamp(1.0, 20.0);
        }
    }

    fn process(&mut self, input: f32, _ctx: &Context) -> f32 {
        (input * self.drive).tanh()
    }
}
