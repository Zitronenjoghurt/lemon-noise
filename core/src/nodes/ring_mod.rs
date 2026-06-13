use crate::context::Context;
use crate::node::Node;
use crate::param::{ParamKind, ParamSpec, ParamValue};
use std::f32::consts::TAU;

const PARAMS: &[ParamSpec] = &[
    ParamSpec {
        id: "frequency",
        label: "Frequency",
        description: "Pitch of the carrier the input is multiplied with.",
        unit: Some(" Hz"),
        kind: ParamKind::Float {
            min: 1.0,
            max: 5_000.0,
            default: 200.0,
            logarithmic: true,
        },
    },
    ParamSpec {
        id: "mix",
        label: "Mix",
        description: "Blend between the dry input and the ring-modulated signal.",
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
pub struct RingMod {
    frequency: f32,
    mix: f32,
    phase: f32,
}

impl RingMod {
    pub const ID: &'static str = "ring_mod";
    pub const LABEL: &'static str = "Ring Modulator";
    pub const DESCRIPTION: &'static str = "Multiplies the signal with a sine carrier, adding clangorous metallic and bell-like tones.";
}

impl Default for RingMod {
    fn default() -> Self {
        Self {
            frequency: 200.0,
            mix: 1.0,
            phase: 0.0,
        }
    }
}

impl Node for RingMod {
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
            "mix" => Some(ParamValue::Float(self.mix)),
            _ => None,
        }
    }

    fn set_param(&mut self, id: &str, value: ParamValue) {
        match id {
            "frequency" => self.frequency = value.as_float().clamp(1.0, 5_000.0),
            "mix" => self.mix = value.as_float().clamp(0.0, 1.0),
            _ => {}
        }
    }

    fn process(&mut self, input: f32, ctx: &Context) -> f32 {
        self.phase += self.frequency * ctx.dt();
        if self.phase >= 1.0 {
            self.phase -= self.phase.floor();
        }
        let carrier = (TAU * self.phase).sin();
        input * (1.0 - self.mix) + input * carrier * self.mix
    }

    fn reset(&mut self) {
        self.phase = 0.0;
    }
}
