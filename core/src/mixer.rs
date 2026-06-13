use crate::pipeline::Pipeline;

#[derive(Clone)]
pub struct Channel {
    pub name: String,
    pub pipeline: Pipeline,
    pub gain: f32,
    pub muted: bool,
    pub soloed: bool,
}

impl Channel {
    pub fn new(name: impl Into<String>, sample_rate: u32) -> Self {
        Self {
            name: name.into(),
            pipeline: Pipeline::new(sample_rate),
            gain: 1.0,
            muted: false,
            soloed: false,
        }
    }
}

#[derive(Clone)]
pub struct Mixer {
    channels: Vec<Channel>,
    sample_rate: u32,
}

impl Mixer {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            channels: Vec::new(),
            sample_rate,
        }
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate;
        for channel in &mut self.channels {
            channel.pipeline.set_sample_rate(sample_rate);
        }
    }

    pub fn len(&self) -> usize {
        self.channels.len()
    }

    pub fn is_empty(&self) -> bool {
        self.channels.is_empty()
    }

    pub fn channels(&self) -> &[Channel] {
        &self.channels
    }

    pub fn channel(&self, index: usize) -> Option<&Channel> {
        self.channels.get(index)
    }

    pub fn channel_mut(&mut self, index: usize) -> Option<&mut Channel> {
        self.channels.get_mut(index)
    }

    pub fn add_channel(&mut self, name: impl Into<String>) -> usize {
        self.channels.push(Channel::new(name, self.sample_rate));
        self.channels.len() - 1
    }

    pub fn push_channel(&mut self, mut channel: Channel) {
        channel.pipeline.set_sample_rate(self.sample_rate);
        self.channels.push(channel);
    }

    pub fn remove_channel(&mut self, index: usize) -> Option<Channel> {
        if index < self.channels.len() {
            Some(self.channels.remove(index))
        } else {
            None
        }
    }

    pub fn reset(&mut self) {
        for channel in &mut self.channels {
            channel.pipeline.reset();
        }
    }

    pub fn render(&mut self, count: usize) -> Vec<f32> {
        (0..count).map(|_| self.tick()).collect()
    }

    fn tick(&mut self) -> f32 {
        let any_solo = self.channels.iter().any(|channel| channel.soloed);
        let mut sum = 0.0;
        for channel in &mut self.channels {
            let sample = channel.pipeline.next().unwrap_or(0.0);
            let audible = if any_solo {
                channel.soloed
            } else {
                !channel.muted
            };
            if audible {
                sum += sample * channel.gain;
            }
        }
        sum
    }
}

impl Iterator for Mixer {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        Some(self.tick())
    }
}
