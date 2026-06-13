use crate::context::Context;
use crate::node::Node;
use crate::param::{ParamKind, ParamSpec, ParamValue};

const PARAMS: &[ParamSpec] = &[
    ParamSpec {
        id: "threshold",
        label: "Threshold",
        description: "Level above which the signal starts to be turned down.",
        unit: Some(" dB"),
        kind: ParamKind::Float {
            min: -60.0,
            max: 0.0,
            default: -18.0,
            logarithmic: false,
        },
    },
    ParamSpec {
        id: "ratio",
        label: "Ratio",
        description: "How hard signal above the threshold is reduced. 4 means 4 dB in becomes 1 dB out.",
        unit: Some(":1"),
        kind: ParamKind::Float {
            min: 1.0,
            max: 20.0,
            default: 4.0,
            logarithmic: false,
        },
    },
    ParamSpec {
        id: "attack",
        label: "Attack",
        description: "How quickly the compressor clamps down once the signal gets loud.",
        unit: Some(" ms"),
        kind: ParamKind::Float {
            min: 0.1,
            max: 100.0,
            default: 5.0,
            logarithmic: true,
        },
    },
    ParamSpec {
        id: "release",
        label: "Release",
        description: "How quickly the compressor lets go once the signal drops back down.",
        unit: Some(" ms"),
        kind: ParamKind::Float {
            min: 5.0,
            max: 1_000.0,
            default: 120.0,
            logarithmic: true,
        },
    },
    ParamSpec {
        id: "makeup",
        label: "Makeup",
        description: "Extra gain applied after compression to bring the level back up.",
        unit: Some(" dB"),
        kind: ParamKind::Float {
            min: 0.0,
            max: 24.0,
            default: 0.0,
            logarithmic: false,
        },
    },
];

#[derive(Debug, Clone)]
pub struct Compressor {
    threshold: f32,
    ratio: f32,
    attack: f32,
    release: f32,
    makeup: f32,
    envelope: f32,
}

impl Compressor {
    pub const ID: &'static str = "compressor";
    pub const LABEL: &'static str = "Compressor";
    pub const DESCRIPTION: &'static str =
        "Tames peaks and evens out dynamics, making the signal denser and more controlled.";
}

impl Default for Compressor {
    fn default() -> Self {
        Self {
            threshold: -18.0,
            ratio: 4.0,
            attack: 5.0,
            release: 120.0,
            makeup: 0.0,
            envelope: 0.0,
        }
    }
}

fn coefficient(time_ms: f32, sample_rate: f32) -> f32 {
    let samples = (time_ms * 0.001 * sample_rate).max(1.0);
    1.0 - (-1.0 / samples).exp()
}

impl Node for Compressor {
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
            "ratio" => Some(ParamValue::Float(self.ratio)),
            "attack" => Some(ParamValue::Float(self.attack)),
            "release" => Some(ParamValue::Float(self.release)),
            "makeup" => Some(ParamValue::Float(self.makeup)),
            _ => None,
        }
    }

    fn set_param(&mut self, id: &str, value: ParamValue) {
        match id {
            "threshold" => self.threshold = value.as_float().clamp(-60.0, 0.0),
            "ratio" => self.ratio = value.as_float().clamp(1.0, 20.0),
            "attack" => self.attack = value.as_float().clamp(0.1, 100.0),
            "release" => self.release = value.as_float().clamp(5.0, 1_000.0),
            "makeup" => self.makeup = value.as_float().clamp(0.0, 24.0),
            _ => {}
        }
    }

    fn process(&mut self, input: f32, ctx: &Context) -> f32 {
        let sample_rate = ctx.sample_rate() as f32;
        let level = input.abs();
        let coeff = if level > self.envelope {
            coefficient(self.attack, sample_rate)
        } else {
            coefficient(self.release, sample_rate)
        };
        self.envelope += coeff * (level - self.envelope);

        let level_db = 20.0 * self.envelope.max(1e-6).log10();
        let over = level_db - self.threshold;
        let reduction_db = if over > 0.0 {
            over * (1.0 / self.ratio - 1.0)
        } else {
            0.0
        };

        let gain = 10.0_f32.powf((reduction_db + self.makeup) / 20.0);
        input * gain
    }

    fn reset(&mut self) {
        self.envelope = 0.0;
    }
}
