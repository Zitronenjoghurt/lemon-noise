use crate::context::Context;
use crate::param::ParamValue;
use crate::rng::Rng;
use std::f32::consts::TAU;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LfoShape {
    Sine,
    Triangle,
    Saw,
    Square,
    Random,
}

impl LfoShape {
    pub const ALL: [LfoShape; 5] = [
        LfoShape::Sine,
        LfoShape::Triangle,
        LfoShape::Saw,
        LfoShape::Square,
        LfoShape::Random,
    ];

    pub fn label(self) -> &'static str {
        match self {
            LfoShape::Sine => "Sine",
            LfoShape::Triangle => "Triangle",
            LfoShape::Saw => "Saw",
            LfoShape::Square => "Square",
            LfoShape::Random => "Random",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ModMode {
    Add,
    Multiply,
}

impl ModMode {
    pub const ALL: [ModMode; 2] = [ModMode::Add, ModMode::Multiply];

    pub fn label(self) -> &'static str {
        match self {
            ModMode::Add => "Add",
            ModMode::Multiply => "Multiply",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Interp {
    Step,
    Linear,
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AutomationPoint {
    pub time: f32,
    pub value: f32,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Modulator {
    Lfo {
        shape: LfoShape,
        rate: f32,
        seed: u64,
    },
    Automation {
        points: Vec<AutomationPoint>,
        duration: f32,
        looping: bool,
        interp: Interp,
    },
}

impl Modulator {
    pub fn lfo(shape: LfoShape, rate: f32) -> Self {
        Modulator::Lfo {
            shape,
            rate: rate.max(0.0),
            seed: 1,
        }
    }

    pub fn automation(points: Vec<AutomationPoint>, duration: f32, looping: bool) -> Self {
        Modulator::Automation {
            points,
            duration: duration.max(0.0),
            looping,
            interp: Interp::Linear,
        }
    }

    pub fn value(&self, ctx: &Context) -> f32 {
        match self {
            Modulator::Lfo { shape, rate, seed } => lfo_value(*shape, *rate, *seed, ctx.time()),
            Modulator::Automation {
                points,
                duration,
                looping,
                interp,
            } => automation_value(points, *duration, *looping, *interp, ctx.time()),
        }
    }
}

fn lfo_value(shape: LfoShape, rate: f32, seed: u64, time: f32) -> f32 {
    let position = time * rate;
    match shape {
        LfoShape::Sine => (TAU * position).sin(),
        LfoShape::Triangle => 1.0 - 4.0 * (position.rem_euclid(1.0) - 0.5).abs(),
        LfoShape::Saw => 2.0 * position.rem_euclid(1.0) - 1.0,
        LfoShape::Square => {
            if position.rem_euclid(1.0) < 0.5 {
                1.0
            } else {
                -1.0
            }
        }
        LfoShape::Random => {
            let cycle = position.floor();
            let frac = position - cycle;
            let current = random_step(seed, cycle as i64);
            let next = random_step(seed, cycle as i64 + 1);
            let blend = frac * frac * (3.0 - 2.0 * frac);
            current + (next - current) * blend
        }
    }
}

fn random_step(seed: u64, cycle: i64) -> f32 {
    Rng::new(seed ^ (cycle as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)).next_bipolar()
}

fn automation_value(
    points: &[AutomationPoint],
    duration: f32,
    looping: bool,
    interp: Interp,
    time: f32,
) -> f32 {
    if points.is_empty() {
        return 0.0;
    }

    let position = if looping && duration > 0.0 {
        time.rem_euclid(duration)
    } else {
        time.min(duration)
    };

    if position <= points[0].time {
        return points[0].value;
    }
    let last = points[points.len() - 1];
    if position >= last.time {
        return last.value;
    }

    let upper = points
        .iter()
        .position(|point| point.time >= position)
        .unwrap_or(points.len() - 1);
    let high = points[upper];
    let low = points[upper - 1];

    match interp {
        Interp::Step => low.value,
        Interp::Linear => {
            let span = (high.time - low.time).max(f32::EPSILON);
            let t = (position - low.time) / span;
            low.value + (high.value - low.value) * t
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Binding {
    pub node: usize,
    pub param: &'static str,
    pub base: ParamValue,
    pub depth: f32,
    pub mode: ModMode,
    pub modulator: Modulator,
}

impl Binding {
    pub fn new(node: usize, param: &'static str, base: ParamValue, modulator: Modulator) -> Self {
        Self {
            node,
            param,
            base,
            depth: 1.0,
            mode: ModMode::Add,
            modulator,
        }
    }

    pub fn resolve(&self, ctx: &Context) -> ParamValue {
        let signal = self.modulator.value(ctx);
        let base = self.base.as_float();
        let value = match self.mode {
            ModMode::Add => base + self.depth * signal,
            ModMode::Multiply => base * (1.0 + self.depth * signal),
        };
        ParamValue::Float(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lfo_shapes_stay_in_range() {
        for shape in LfoShape::ALL {
            let modulator = Modulator::lfo(shape, 2.0);
            let mut ctx = Context::new(1_000);
            for _ in 0..5_000 {
                let value = modulator.value(&ctx);
                assert!(
                    (-1.001..=1.001).contains(&value),
                    "{shape:?} produced {value}"
                );
                ctx.advance();
            }
        }
    }

    #[test]
    fn random_lfo_is_deterministic() {
        let modulator = Modulator::lfo(LfoShape::Random, 5.0);
        let mut a = Context::new(1_000);
        let mut b = Context::new(1_000);
        for _ in 0..200 {
            assert_eq!(modulator.value(&a), modulator.value(&b));
            a.advance();
            b.advance();
        }
    }

    #[test]
    fn automation_interpolates_linearly() {
        let points = vec![
            AutomationPoint {
                time: 0.0,
                value: -1.0,
            },
            AutomationPoint {
                time: 1.0,
                value: 1.0,
            },
        ];
        let modulator = Modulator::automation(points, 1.0, false);

        let start = Context::new(1_000);
        assert!((modulator.value(&start) + 1.0).abs() < 1e-6);

        let mut middle = Context::new(1_000);
        for _ in 0..500 {
            middle.advance();
        }
        assert!(modulator.value(&middle).abs() < 1e-2);
    }

    #[test]
    fn add_mode_offsets_base() {
        let binding = Binding {
            node: 0,
            param: "x",
            base: ParamValue::Float(0.5),
            depth: 0.5,
            mode: ModMode::Add,
            modulator: Modulator::Automation {
                points: vec![AutomationPoint {
                    time: 0.0,
                    value: 1.0,
                }],
                duration: 1.0,
                looping: false,
                interp: Interp::Step,
            },
        };
        let ctx = Context::new(1_000);
        assert!((binding.resolve(&ctx).as_float() - 1.0).abs() < 1e-6);
    }
}
