#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ParamValue {
    Float(f32),
    Int(i64),
    Bool(bool),
}

impl ParamValue {
    pub fn as_float(self) -> f32 {
        match self {
            ParamValue::Float(v) => v,
            ParamValue::Int(v) => v as f32,
            ParamValue::Bool(v) => v as i64 as f32,
        }
    }

    pub fn as_int(self) -> i64 {
        match self {
            ParamValue::Float(v) => v as i64,
            ParamValue::Int(v) => v,
            ParamValue::Bool(v) => v as i64,
        }
    }

    pub fn as_bool(self) -> bool {
        match self {
            ParamValue::Float(v) => v != 0.0,
            ParamValue::Int(v) => v != 0,
            ParamValue::Bool(v) => v,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParamKind {
    Float {
        min: f32,
        max: f32,
        default: f32,
        logarithmic: bool,
    },
    Int {
        min: i64,
        max: i64,
        default: i64,
    },
    Bool {
        default: bool,
    },
    Choice {
        options: &'static [&'static str],
        default: usize,
    },
}

impl ParamKind {
    pub fn default_value(self) -> ParamValue {
        match self {
            ParamKind::Float { default, .. } => ParamValue::Float(default),
            ParamKind::Int { default, .. } => ParamValue::Int(default),
            ParamKind::Bool { default } => ParamValue::Bool(default),
            ParamKind::Choice { default, .. } => ParamValue::Int(default as i64),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ParamSpec {
    pub id: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub unit: Option<&'static str>,
    pub kind: ParamKind,
}
