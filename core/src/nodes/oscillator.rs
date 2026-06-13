use crate::context::Context;
use crate::node::Node;
use crate::param::{ParamKind, ParamSpec, ParamValue};
use std::f32::consts::TAU;

const SHAPES: &[&str] = &["Sine", "Triangle", "Saw", "Square"];

const PARAMS: &[ParamSpec] = &[
    ParamSpec {
        id: "frequency",
        label: "Frequency",
        description: "Pitch of the tone.",
        unit: Some(" Hz"),
        kind: ParamKind::Float {
            min: 20.0,
            max: 12_000.0,
            default: 220.0,
            logarithmic: true,
        },
    },
    ParamSpec {
        id: "amplitude",
        label: "Amplitude",
        description: "Output level of the tone.",
        unit: None,
        kind: ParamKind::Float {
            min: 0.0,
            max: 1.0,
            default: 0.5,
            logarithmic: false,
        },
    },
    ParamSpec {
        id: "shape",
        label: "Shape",
        description: "Waveform of the oscillator.",
        unit: None,
        kind: ParamKind::Choice {
            options: SHAPES,
            default: 0,
        },
    },
];

#[derive(Debug, Clone)]
pub struct Oscillator {
    frequency: f32,
    amplitude: f32,
    shape: usize,
    phase: f32,
}

impl Oscillator {
    pub const ID: &'static str = "oscillator";
    pub const LABEL: &'static str = "Oscillator";
    pub const DESCRIPTION: &'static str =
        "A tonal waveform generator. Layer it under noise for drones, hums and pitched beds.";
}

impl Default for Oscillator {
    fn default() -> Self {
        Self {
            frequency: 220.0,
            amplitude: 0.5,
            shape: 0,
            phase: 0.0,
        }
    }
}

impl Node for Oscillator {
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
            "frequency" => Some(ParamValue::Float(self.frequency)),
            "amplitude" => Some(ParamValue::Float(self.amplitude)),
            "shape" => Some(ParamValue::Int(self.shape as i64)),
            _ => None,
        }
    }

    fn set_param(&mut self, id: &str, value: ParamValue) {
        match id {
            "frequency" => self.frequency = value.as_float().clamp(20.0, 12_000.0),
            "amplitude" => self.amplitude = value.as_float().clamp(0.0, 1.0),
            "shape" => self.shape = (value.as_int().max(0) as usize).min(SHAPES.len() - 1),
            _ => {}
        }
    }

    fn process(&mut self, _input: f32, ctx: &Context) -> f32 {
        self.phase += self.frequency * ctx.dt();
        if self.phase >= 1.0 {
            self.phase -= self.phase.floor();
        }

        let wave = match self.shape {
            0 => (TAU * self.phase).sin(),
            1 => 1.0 - 4.0 * (self.phase - 0.5).abs(),
            2 => 2.0 * self.phase - 1.0,
            _ => {
                if self.phase < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
        };
        wave * self.amplitude
    }

    fn reset(&mut self) {
        self.phase = 0.0;
    }
}
