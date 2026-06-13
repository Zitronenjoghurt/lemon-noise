use crate::context::Context;
use crate::node::Node;
use crate::param::{ParamKind, ParamSpec, ParamValue};

const PARAMS: &[ParamSpec] = &[ParamSpec {
    id: "bits",
    label: "Bits",
    description: "Bit depth the signal is quantized to. Fewer bits add gritty digital distortion.",
    unit: None,
    kind: ParamKind::Int {
        min: 1,
        max: 16,
        default: 8,
    },
}];

#[derive(Debug, Clone)]
pub struct Bitcrush {
    bits: u32,
}

impl Bitcrush {
    pub const ID: &'static str = "bitcrush";
    pub const LABEL: &'static str = "Bitcrush";
    pub const DESCRIPTION: &'static str = "Reduces the bit depth, quantizing the signal into coarse steps for a lo-fi, crunchy sound.";
}

impl Default for Bitcrush {
    fn default() -> Self {
        Self { bits: 8 }
    }
}

impl Node for Bitcrush {
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
            "bits" => Some(ParamValue::Int(self.bits as i64)),
            _ => None,
        }
    }

    fn set_param(&mut self, id: &str, value: ParamValue) {
        if id == "bits" {
            self.bits = value.as_int().clamp(1, 16) as u32;
        }
    }

    fn process(&mut self, input: f32, _ctx: &Context) -> f32 {
        let levels = (1u32 << self.bits) as f32;
        let step = 2.0 / (levels - 1.0).max(1.0);
        (input / step).round() * step
    }
}
