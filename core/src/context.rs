#[derive(Debug, Clone)]
pub struct Context {
    sample_rate: u32,
    sample_index: u64,
}

impl Context {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate: sample_rate.max(1),
            sample_index: 0,
        }
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate.max(1);
    }

    pub fn sample_index(&self) -> u64 {
        self.sample_index
    }

    pub fn time(&self) -> f32 {
        self.sample_index as f32 / self.sample_rate as f32
    }

    pub fn dt(&self) -> f32 {
        1.0 / self.sample_rate as f32
    }

    pub fn advance(&mut self) {
        self.sample_index = self.sample_index.wrapping_add(1);
    }

    pub fn reset(&mut self) {
        self.sample_index = 0;
    }
}
