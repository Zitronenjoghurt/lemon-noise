use crate::context::Context;
use crate::node::Node;
use crate::param::{ParamKind, ParamSpec, ParamValue};
use std::f32::consts::PI;

const MODES: &[&str] = &["Low Pass", "High Pass", "Band Pass", "Notch"];

const PARAMS: &[ParamSpec] = &[
    ParamSpec {
        id: "cutoff",
        label: "Cutoff",
        description: "Center/corner frequency of the filter.",
        unit: Some(" Hz"),
        kind: ParamKind::Float {
            min: 20.0,
            max: 12_000.0,
            default: 1_000.0,
            logarithmic: true,
        },
    },
    ParamSpec {
        id: "resonance",
        label: "Resonance",
        description: "Emphasis around the cutoff. High values ring and can nearly self-oscillate.",
        unit: None,
        kind: ParamKind::Float {
            min: 0.0,
            max: 1.0,
            default: 0.2,
            logarithmic: false,
        },
    },
    ParamSpec {
        id: "mode",
        label: "Mode",
        description: "Which slice of the spectrum to keep.",
        unit: None,
        kind: ParamKind::Choice {
            options: MODES,
            default: 0,
        },
    },
];

#[derive(Debug, Clone)]
pub struct Svf {
    cutoff: f32,
    resonance: f32,
    mode: usize,
    low: f32,
    band: f32,
}

impl Svf {
    pub const ID: &'static str = "svf";
    pub const LABEL: &'static str = "Resonant Filter";
    pub const DESCRIPTION: &'static str = "A state-variable filter with adjustable resonance and selectable low/high/band/notch response.";
}

impl Default for Svf {
    fn default() -> Self {
        Self {
            cutoff: 1_000.0,
            resonance: 0.2,
            mode: 0,
            low: 0.0,
            band: 0.0,
        }
    }
}

impl Node for Svf {
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
            "cutoff" => Some(ParamValue::Float(self.cutoff)),
            "resonance" => Some(ParamValue::Float(self.resonance)),
            "mode" => Some(ParamValue::Int(self.mode as i64)),
            _ => None,
        }
    }

    fn set_param(&mut self, id: &str, value: ParamValue) {
        match id {
            "cutoff" => self.cutoff = value.as_float().clamp(20.0, 12_000.0),
            "resonance" => self.resonance = value.as_float().clamp(0.0, 1.0),
            "mode" => self.mode = (value.as_int().max(0) as usize).min(MODES.len() - 1),
            _ => {}
        }
    }

    fn process(&mut self, input: f32, ctx: &Context) -> f32 {
        let nyquist = ctx.sample_rate() as f32 * 0.5;
        let cutoff = self.cutoff.min(nyquist / 3.0);
        let f = 2.0 * (PI * cutoff / ctx.sample_rate() as f32).sin();
        let damping = (1.0 - self.resonance).clamp(0.02, 1.0);

        self.low += f * self.band;
        let high = input - self.low - damping * self.band;
        self.band += f * high;
        let notch = high + self.low;

        if !self.low.is_finite() || !self.band.is_finite() {
            self.low = 0.0;
            self.band = 0.0;
        }

        let output = match self.mode {
            0 => self.low,
            1 => high,
            2 => self.band,
            _ => notch,
        };
        output.clamp(-1.5, 1.5)
    }

    fn reset(&mut self) {
        self.low = 0.0;
        self.band = 0.0;
    }
}
