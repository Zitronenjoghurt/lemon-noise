use crate::context::Context;
use crate::node::Node;
use crate::param::{ParamKind, ParamSpec, ParamValue};

const PARAMS: &[ParamSpec] = &[
    ParamSpec {
        id: "threshold",
        label: "Threshold",
        description: "Signal quieter than this is silenced. Louder signal passes through.",
        unit: Some(" dB"),
        kind: ParamKind::Float {
            min: -80.0,
            max: 0.0,
            default: -40.0,
            logarithmic: false,
        },
    },
    ParamSpec {
        id: "attack",
        label: "Attack",
        description: "How quickly the gate opens once the signal rises above the threshold.",
        unit: Some(" ms"),
        kind: ParamKind::Float {
            min: 0.1,
            max: 100.0,
            default: 1.0,
            logarithmic: true,
        },
    },
    ParamSpec {
        id: "release",
        label: "Release",
        description: "How quickly the gate closes once the signal falls below the threshold.",
        unit: Some(" ms"),
        kind: ParamKind::Float {
            min: 1.0,
            max: 1_000.0,
            default: 80.0,
            logarithmic: true,
        },
    },
];

#[derive(Debug, Clone)]
pub struct Gate {
    threshold: f32,
    attack: f32,
    release: f32,
    envelope: f32,
    gain: f32,
}

impl Gate {
    pub const ID: &'static str = "gate";
    pub const LABEL: &'static str = "Gate";
    pub const DESCRIPTION: &'static str =
        "Mutes the signal while it sits below the threshold. Carves silence between louder events.";
}

impl Default for Gate {
    fn default() -> Self {
        Self {
            threshold: -40.0,
            attack: 1.0,
            release: 80.0,
            envelope: 0.0,
            gain: 0.0,
        }
    }
}

fn coefficient(time_ms: f32, sample_rate: f32) -> f32 {
    let samples = (time_ms * 0.001 * sample_rate).max(1.0);
    1.0 - (-1.0 / samples).exp()
}

impl Node for Gate {
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
            "threshold" => Some(ParamValue::Float(self.threshold)),
            "attack" => Some(ParamValue::Float(self.attack)),
            "release" => Some(ParamValue::Float(self.release)),
            _ => None,
        }
    }

    fn set_param(&mut self, id: &str, value: ParamValue) {
        match id {
            "threshold" => self.threshold = value.as_float().clamp(-80.0, 0.0),
            "attack" => self.attack = value.as_float().clamp(0.1, 100.0),
            "release" => self.release = value.as_float().clamp(1.0, 1_000.0),
            _ => {}
        }
    }

    fn process(&mut self, input: f32, ctx: &Context) -> f32 {
        let sample_rate = ctx.sample_rate() as f32;
        let level = input.abs();
        self.envelope = self.envelope.max(level);

        let level_db = 20.0 * self.envelope.max(1e-6).log10();
        let target = if level_db >= self.threshold { 1.0 } else { 0.0 };
        let coeff = if target > self.gain {
            coefficient(self.attack, sample_rate)
        } else {
            coefficient(self.release, sample_rate)
        };
        self.gain += coeff * (target - self.gain);

        let decay = coefficient(self.release, sample_rate);
        self.envelope -= decay * self.envelope;

        input * self.gain
    }

    fn reset(&mut self) {
        self.envelope = 0.0;
        self.gain = 0.0;
    }
}
