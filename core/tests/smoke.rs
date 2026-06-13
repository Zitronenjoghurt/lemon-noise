use lemon_noise::{Mixer, Pipeline, Registry};

#[test]
fn every_builtin_node_renders_finite_and_bounded() {
    let registry = Registry::with_builtins();
    for descriptor in registry.descriptors() {
        let mut pipeline = Pipeline::new(44_100);
        pipeline.push(registry.create("white_noise").expect("white noise builtin"));
        pipeline.push(
            registry
                .create(descriptor.id)
                .unwrap_or_else(|| panic!("registry could not create {}", descriptor.id)),
        );

        for sample in pipeline.render(4_000) {
            assert!(
                sample.is_finite(),
                "{} produced a non-finite sample",
                descriptor.id
            );
            assert!(
                sample.abs() <= 8.0,
                "{} produced a runaway sample: {sample}",
                descriptor.id
            );
        }
    }
}

#[test]
fn registry_exposes_new_nodes() {
    let registry = Registry::with_builtins();
    for id in ["violet_noise", "pulse", "compressor", "gate"] {
        assert!(
            registry.get(id).is_some(),
            "registry is missing the {id} node"
        );
    }
}

#[test]
fn descriptors_sort_generators_first_then_by_label() {
    let registry = Registry::with_builtins();
    let descriptors = registry.descriptors();

    let first_processor = descriptors
        .iter()
        .position(|d| !d.is_source)
        .expect("expected at least one processor");
    assert!(
        descriptors[first_processor..].iter().all(|d| !d.is_source),
        "generators must come before processors"
    );

    let labels: Vec<&str> = descriptors.iter().map(|d| d.label).collect();
    let sources_sorted = descriptors[..first_processor]
        .windows(2)
        .all(|w| w[0].label <= w[1].label);
    let processors_sorted = descriptors[first_processor..]
        .windows(2)
        .all(|w| w[0].label <= w[1].label);
    assert!(
        sources_sorted,
        "generators must be sorted by label: {labels:?}"
    );
    assert!(
        processors_sorted,
        "processors must be sorted by label: {labels:?}"
    );
}

#[test]
fn mixer_respects_mute_and_solo() {
    let registry = Registry::with_builtins();

    let mut mixer = Mixer::new(44_100);
    let a = mixer.add_channel("a");
    mixer
        .channel_mut(a)
        .unwrap()
        .pipeline
        .push(registry.create("white_noise").unwrap());
    let b = mixer.add_channel("b");
    mixer
        .channel_mut(b)
        .unwrap()
        .pipeline
        .push(registry.create("white_noise").unwrap());

    mixer.channel_mut(a).unwrap().muted = true;
    let muted = mixer.clone().render(64);
    let only_b: Vec<f32> = {
        let mut reference = Mixer::new(44_100);
        let index = reference.add_channel("b");
        reference
            .channel_mut(index)
            .unwrap()
            .pipeline
            .push(registry.create("white_noise").unwrap());
        reference.render(64)
    };
    assert_eq!(muted, only_b);

    mixer.channel_mut(a).unwrap().muted = false;
    mixer.channel_mut(b).unwrap().soloed = true;
    assert_eq!(mixer.render(64), only_b);
}

#[cfg(feature = "serde")]
#[test]
fn capture_apply_reproduces_audio() {
    use lemon_noise::{Binding, LfoShape, Modulator, ParamValue, ProjectState};

    let registry = Registry::with_builtins();
    let mut mixer = Mixer::new(44_100);
    let index = mixer.add_channel("layer");
    let channel = mixer.channel_mut(index).unwrap();
    channel.pipeline.push(registry.create("pink").unwrap());
    channel.pipeline.push(registry.create("svf").unwrap());
    channel.pipeline.set_binding(Binding::new(
        1,
        "cutoff",
        ParamValue::Float(1_000.0),
        Modulator::lfo(LfoShape::Sine, 0.5),
    ));

    let restored = ProjectState::capture(&mixer).apply(&registry, 44_100);

    mixer.reset();
    let mut restored = restored;
    restored.reset();
    assert_eq!(mixer.render(512), restored.render(512));
}

#[cfg(feature = "persistence")]
#[test]
fn persistence_roundtrip_through_bytes() {
    use lemon_noise::ProjectState;
    use lemon_noise::persistence::{decode, encode};

    let registry = Registry::with_builtins();
    let mut mixer = Mixer::new(44_100);
    let index = mixer.add_channel("layer");
    mixer
        .channel_mut(index)
        .unwrap()
        .pipeline
        .push(registry.create("brown").unwrap());

    let bytes = encode(&ProjectState::capture(&mixer)).expect("encode");
    let decoded: ProjectState = decode(&bytes).expect("decode");
    let restored = decoded.apply(&registry, 44_100);

    assert_eq!(restored.len(), 1);
    assert_eq!(restored.channel(0).unwrap().pipeline.len(), 1);
}
