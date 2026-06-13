use crate::context::Context;
use crate::modulation::Binding;
use crate::node::Node;

#[derive(Clone)]
struct NodeEntry {
    node: Box<dyn Node>,
    enabled: bool,
}

#[derive(Clone)]
pub struct Pipeline {
    nodes: Vec<NodeEntry>,
    bindings: Vec<Binding>,
    ctx: Context,
}

impl Pipeline {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            nodes: Vec::new(),
            bindings: Vec::new(),
            ctx: Context::new(sample_rate),
        }
    }

    pub fn sample_rate(&self) -> u32 {
        self.ctx.sample_rate()
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.ctx.set_sample_rate(sample_rate);
    }

    pub fn context(&self) -> &Context {
        &self.ctx
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn node(&self, index: usize) -> Option<&dyn Node> {
        self.nodes.get(index).map(|entry| entry.node.as_ref())
    }

    pub fn node_mut(&mut self, index: usize) -> Option<&mut Box<dyn Node>> {
        self.nodes.get_mut(index).map(|entry| &mut entry.node)
    }

    pub fn is_enabled(&self, index: usize) -> bool {
        self.nodes.get(index).is_none_or(|entry| entry.enabled)
    }

    pub fn set_enabled(&mut self, index: usize, enabled: bool) {
        if let Some(entry) = self.nodes.get_mut(index) {
            entry.enabled = enabled;
        }
    }

    pub fn push(&mut self, node: Box<dyn Node>) {
        self.push_with(node, true);
    }

    pub fn push_with(&mut self, node: Box<dyn Node>, enabled: bool) {
        self.nodes.push(NodeEntry { node, enabled });
    }

    pub fn replace(&mut self, index: usize, node: Box<dyn Node>) -> Option<Box<dyn Node>> {
        let entry = self.nodes.get_mut(index)?;
        let previous = std::mem::replace(&mut entry.node, node);
        self.bindings.retain(|binding| binding.node != index);
        Some(previous)
    }

    pub fn insert(&mut self, index: usize, node: Box<dyn Node>) {
        let index = index.min(self.nodes.len());
        self.nodes.insert(
            index,
            NodeEntry {
                node,
                enabled: true,
            },
        );
        for binding in &mut self.bindings {
            if binding.node >= index {
                binding.node += 1;
            }
        }
    }

    pub fn remove(&mut self, index: usize) -> Option<Box<dyn Node>> {
        if index < self.nodes.len() {
            let entry = self.nodes.remove(index);
            self.bindings.retain(|binding| binding.node != index);
            for binding in &mut self.bindings {
                if binding.node > index {
                    binding.node -= 1;
                }
            }
            Some(entry.node)
        } else {
            None
        }
    }

    pub fn swap(&mut self, a: usize, b: usize) {
        if a < self.nodes.len() && b < self.nodes.len() {
            self.nodes.swap(a, b);
            for binding in &mut self.bindings {
                if binding.node == a {
                    binding.node = b;
                } else if binding.node == b {
                    binding.node = a;
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.bindings.clear();
        self.ctx.reset();
    }

    pub fn bindings(&self) -> &[Binding] {
        &self.bindings
    }

    pub fn binding(&self, node: usize, param: &str) -> Option<&Binding> {
        self.bindings
            .iter()
            .find(|binding| binding.node == node && binding.param == param)
    }

    pub fn set_binding(&mut self, binding: Binding) {
        if let Some(existing) = self
            .bindings
            .iter_mut()
            .find(|item| item.node == binding.node && item.param == binding.param)
        {
            *existing = binding;
        } else {
            self.bindings.push(binding);
        }
    }

    pub fn remove_binding(&mut self, node: usize, param: &str) -> Option<Binding> {
        let position = self
            .bindings
            .iter()
            .position(|binding| binding.node == node && binding.param == param)?;
        Some(self.bindings.remove(position))
    }

    pub fn reset(&mut self) {
        for entry in &mut self.nodes {
            entry.node.reset();
        }
        self.ctx.reset();
    }

    pub fn render(&mut self, count: usize) -> Vec<f32> {
        (0..count).map(|_| self.tick()).collect()
    }

    fn tick(&mut self) -> f32 {
        for binding in &self.bindings {
            if let Some(entry) = self.nodes.get_mut(binding.node)
                && entry.enabled
            {
                let value = binding.resolve(&self.ctx);
                entry.node.set_param(binding.param, value);
            }
        }

        let mut sample = 0.0;
        for entry in &mut self.nodes {
            if entry.enabled {
                sample = entry.node.process(sample, &self.ctx);
            }
        }
        self.ctx.advance();
        sample
    }
}

impl Iterator for Pipeline {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        Some(self.tick())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::{Gain, WhiteNoise};
    use crate::param::ParamValue;

    #[test]
    fn empty_pipeline_is_silent() {
        let mut pipeline = Pipeline::new(44_100);
        assert_eq!(pipeline.render(8), vec![0.0; 8]);
    }

    #[test]
    fn white_noise_stays_within_amplitude() {
        let mut pipeline = Pipeline::new(44_100);
        pipeline.push(Box::new(WhiteNoise::default()));
        for sample in pipeline.render(1_000) {
            assert!(sample.abs() <= 0.5 + f32::EPSILON);
        }
    }

    #[test]
    fn gain_scales_input() {
        let mut pipeline = Pipeline::new(44_100);
        pipeline.push(Box::new(WhiteNoise::default()));
        let mut gain: Box<dyn Node> = Box::new(Gain::default());
        gain.set_param("gain", ParamValue::Float(0.0));
        pipeline.push(gain);
        assert_eq!(pipeline.render(16), vec![0.0; 16]);
    }

    #[test]
    fn reset_makes_output_reproducible() {
        let mut pipeline = Pipeline::new(44_100);
        pipeline.push(Box::new(WhiteNoise::default()));
        let first = pipeline.render(64);
        pipeline.reset();
        let second = pipeline.render(64);
        assert_eq!(first, second);
    }

    #[test]
    fn modulation_drives_param_over_time() {
        use crate::modulation::{Binding, LfoShape, ModMode, Modulator};

        let mut pipeline = Pipeline::new(8);
        pipeline.push(Box::new(WhiteNoise::default()));
        let mut binding = Binding::new(
            0,
            "amplitude",
            ParamValue::Float(0.5),
            Modulator::lfo(LfoShape::Saw, 1.0),
        );
        binding.depth = 0.5;
        binding.mode = ModMode::Add;
        pipeline.set_binding(binding);

        pipeline.render(8);
        let amplitude = pipeline.node(0).unwrap().get_param("amplitude").unwrap();
        assert!((amplitude.as_float() - 0.5).abs() > f32::EPSILON);
    }

    #[test]
    fn removing_node_remaps_bindings() {
        use crate::modulation::{Binding, LfoShape, Modulator};

        let mut pipeline = Pipeline::new(44_100);
        pipeline.push(Box::new(WhiteNoise::default()));
        pipeline.push(Box::new(Gain::default()));
        pipeline.set_binding(Binding::new(
            1,
            "gain",
            ParamValue::Float(1.0),
            Modulator::lfo(LfoShape::Sine, 1.0),
        ));
        pipeline.remove(0);
        assert_eq!(pipeline.bindings().len(), 1);
        assert_eq!(pipeline.bindings()[0].node, 0);
    }

    #[test]
    fn disabled_node_is_bypassed() {
        let mut pipeline = Pipeline::new(44_100);
        pipeline.push(Box::new(WhiteNoise::default()));
        let mut gain: Box<dyn Node> = Box::new(Gain::default());
        gain.set_param("gain", ParamValue::Float(0.0));
        pipeline.push(gain);

        assert_eq!(pipeline.render(16), vec![0.0; 16]);

        pipeline.reset();
        pipeline.set_enabled(1, false);
        assert!(pipeline.render(64).iter().any(|sample| *sample != 0.0));
    }

    #[test]
    fn replace_swaps_node_and_drops_bindings() {
        use crate::modulation::{LfoShape, Modulator};

        let mut pipeline = Pipeline::new(44_100);
        pipeline.push(Box::new(WhiteNoise::default()));
        pipeline.set_binding(Binding::new(
            0,
            "amplitude",
            ParamValue::Float(0.5),
            Modulator::lfo(LfoShape::Sine, 1.0),
        ));

        pipeline.replace(0, Box::new(Gain::default()));
        assert_eq!(pipeline.node(0).unwrap().id(), Gain::default().id());
        assert!(pipeline.bindings().is_empty());
    }

    #[test]
    fn cloned_pipeline_is_independent() {
        let mut pipeline = Pipeline::new(44_100);
        pipeline.push(Box::new(WhiteNoise::default()));
        let mut clone = pipeline.clone();
        assert_eq!(pipeline.render(64), clone.render(64));
    }
}
