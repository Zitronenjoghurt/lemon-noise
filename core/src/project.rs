use crate::mixer::Mixer;
use crate::modulation::{Binding, ModMode, Modulator};
use crate::node::Node;
use crate::param::ParamValue;
use crate::registry::Registry;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectState {
    pub channels: Vec<ChannelState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelState {
    pub name: String,
    pub gain: f32,
    pub muted: bool,
    pub soloed: bool,
    pub nodes: Vec<NodeState>,
    pub bindings: Vec<BindingState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeState {
    pub id: String,
    pub params: Vec<(String, ParamValue)>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BindingState {
    pub node: usize,
    pub param: String,
    pub base: ParamValue,
    pub depth: f32,
    pub mode: ModMode,
    pub modulator: Modulator,
}

impl ProjectState {
    pub fn capture(mixer: &Mixer) -> Self {
        let channels = mixer
            .channels()
            .iter()
            .map(|channel| ChannelState {
                name: channel.name.clone(),
                gain: channel.gain,
                muted: channel.muted,
                soloed: channel.soloed,
                nodes: (0..channel.pipeline.len())
                    .map(|index| {
                        node_state(
                            channel.pipeline.node(index).unwrap(),
                            channel.pipeline.is_enabled(index),
                        )
                    })
                    .collect(),
                bindings: channel
                    .pipeline
                    .bindings()
                    .iter()
                    .map(binding_state)
                    .collect(),
            })
            .collect();
        Self { channels }
    }

    pub fn apply(&self, registry: &Registry, sample_rate: u32) -> Mixer {
        let mut mixer = Mixer::new(sample_rate);
        for channel_state in &self.channels {
            let index = mixer.add_channel(channel_state.name.clone());
            let Some(channel) = mixer.channel_mut(index) else {
                continue;
            };
            channel.gain = channel_state.gain;
            channel.muted = channel_state.muted;
            channel.soloed = channel_state.soloed;

            for node_state in &channel_state.nodes {
                if let Some(mut node) = registry.create(&node_state.id) {
                    for (id, value) in &node_state.params {
                        node.set_param(id, *value);
                    }
                    channel.pipeline.push_with(node, node_state.enabled);
                }
            }

            for binding_state in &channel_state.bindings {
                let param = channel
                    .pipeline
                    .node(binding_state.node)
                    .and_then(|node| resolve_param(node, &binding_state.param));
                if let Some(param) = param {
                    channel.pipeline.set_binding(Binding {
                        node: binding_state.node,
                        param,
                        base: binding_state.base,
                        depth: binding_state.depth,
                        mode: binding_state.mode,
                        modulator: binding_state.modulator.clone(),
                    });
                }
            }
        }
        mixer
    }
}

fn node_state(node: &dyn Node, enabled: bool) -> NodeState {
    let params = node
        .params()
        .iter()
        .filter_map(|spec| {
            node.get_param(spec.id)
                .map(|value| (spec.id.to_string(), value))
        })
        .collect();
    NodeState {
        id: node.id().to_string(),
        params,
        enabled,
    }
}

fn binding_state(binding: &Binding) -> BindingState {
    BindingState {
        node: binding.node,
        param: binding.param.to_string(),
        base: binding.base,
        depth: binding.depth,
        mode: binding.mode,
        modulator: binding.modulator.clone(),
    }
}

fn resolve_param(node: &dyn Node, name: &str) -> Option<&'static str> {
    node.params()
        .iter()
        .find(|spec| spec.id == name)
        .map(|spec| spec.id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modulation::{Binding, LfoShape, Modulator};

    #[test]
    fn roundtrip_preserves_structure() {
        let registry = Registry::with_builtins();
        let mut mixer = Mixer::new(44_100);
        let index = mixer.add_channel("Layer");
        let channel = mixer.channel_mut(index).unwrap();
        channel.gain = 0.7;
        channel.muted = true;
        let mut osc = registry.create("oscillator").unwrap();
        osc.set_param("frequency", ParamValue::Float(440.0));
        channel.pipeline.push(osc);
        channel.pipeline.push(registry.create("delay").unwrap());
        channel.pipeline.set_binding(Binding::new(
            0,
            "frequency",
            ParamValue::Float(440.0),
            Modulator::lfo(LfoShape::Sine, 3.0),
        ));

        let restored = ProjectState::capture(&mixer).apply(&registry, 48_000);

        assert_eq!(restored.sample_rate(), 48_000);
        let channel = restored.channel(0).unwrap();
        assert_eq!(channel.name, "Layer");
        assert!((channel.gain - 0.7).abs() < 1e-6);
        assert!(channel.muted);
        assert_eq!(channel.pipeline.len(), 2);
        assert_eq!(channel.pipeline.bindings().len(), 1);
        let frequency = channel
            .pipeline
            .node(0)
            .unwrap()
            .get_param("frequency")
            .unwrap();
        assert!((frequency.as_float() - 440.0).abs() < 1e-3);
    }
}
