use crate::node::Node;
use crate::nodes::{
    Bitcrush, Blue, Brown, Chorus, Clip, Comb, Compressor, DcBlocker, Delay, Dust, Flanger, Gain,
    Gate, HighPass, LowPass, Oscillator, Pink, Pulse, Reverb, RingMod, SampleHold, Saturate, Svf,
    Tremolo, Violet, Wavefolder, WhiteNoise,
};

#[derive(Clone, Copy)]
pub struct NodeDescriptor {
    pub id: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub is_source: bool,
    pub make: fn() -> Box<dyn Node>,
}

#[derive(Clone, Default)]
pub struct Registry {
    descriptors: Vec<NodeDescriptor>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            descriptors: Vec::new(),
        }
    }

    pub fn with_builtins() -> Self {
        let mut registry = Self::new();
        for descriptor in BUILTINS {
            registry.register(*descriptor);
        }
        registry
            .descriptors
            .sort_by(|a, b| b.is_source.cmp(&a.is_source).then(a.label.cmp(b.label)));
        registry
    }

    pub fn register(&mut self, descriptor: NodeDescriptor) {
        if !self.descriptors.iter().any(|d| d.id == descriptor.id) {
            self.descriptors.push(descriptor);
        }
    }

    pub fn descriptors(&self) -> &[NodeDescriptor] {
        &self.descriptors
    }

    pub fn get(&self, id: &str) -> Option<&NodeDescriptor> {
        self.descriptors.iter().find(|d| d.id == id)
    }

    pub fn create(&self, id: &str) -> Option<Box<dyn Node>> {
        self.get(id).map(|descriptor| (descriptor.make)())
    }
}

macro_rules! descriptor {
    ($ty:ty, $source:expr) => {
        NodeDescriptor {
            id: <$ty>::ID,
            label: <$ty>::LABEL,
            description: <$ty>::DESCRIPTION,
            is_source: $source,
            make: || Box::new(<$ty>::default()),
        }
    };
}

const BUILTINS: &[NodeDescriptor] = &[
    descriptor!(WhiteNoise, true),
    descriptor!(Pink, true),
    descriptor!(Brown, true),
    descriptor!(Blue, true),
    descriptor!(Violet, true),
    descriptor!(Dust, true),
    descriptor!(Pulse, true),
    descriptor!(Oscillator, true),
    descriptor!(LowPass, false),
    descriptor!(HighPass, false),
    descriptor!(Svf, false),
    descriptor!(Comb, false),
    descriptor!(SampleHold, false),
    descriptor!(Bitcrush, false),
    descriptor!(Wavefolder, false),
    descriptor!(Saturate, false),
    descriptor!(Clip, false),
    descriptor!(DcBlocker, false),
    descriptor!(Compressor, false),
    descriptor!(Gate, false),
    descriptor!(Tremolo, false),
    descriptor!(RingMod, false),
    descriptor!(Chorus, false),
    descriptor!(Flanger, false),
    descriptor!(Delay, false),
    descriptor!(Reverb, false),
    descriptor!(Gain, false),
];
