#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum WavFormat {
    Pcm16,
    Pcm24,
    Float32,
}

impl WavFormat {
    pub const ALL: [WavFormat; 3] = [WavFormat::Pcm16, WavFormat::Pcm24, WavFormat::Float32];

    pub fn label(self) -> &'static str {
        match self {
            WavFormat::Pcm16 => "WAV 16-bit PCM",
            WavFormat::Pcm24 => "WAV 24-bit PCM",
            WavFormat::Float32 => "WAV 32-bit float",
        }
    }

    fn bits(self) -> u16 {
        match self {
            WavFormat::Pcm16 => 16,
            WavFormat::Pcm24 => 24,
            WavFormat::Float32 => 32,
        }
    }

    fn tag(self) -> u16 {
        match self {
            WavFormat::Pcm16 | WavFormat::Pcm24 => 1,
            WavFormat::Float32 => 3,
        }
    }
}

pub fn encode_wav(samples: &[f32], sample_rate: u32, format: WavFormat) -> Vec<u8> {
    const CHANNELS: u16 = 1;
    let bits = format.bits();
    let bytes_per_sample = (bits / 8) as usize;
    let block_align = CHANNELS * (bits / 8);
    let byte_rate = sample_rate * block_align as u32;
    let data_len = (samples.len() * bytes_per_sample) as u32;

    let mut out = Vec::with_capacity(44 + data_len as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&format.tag().to_le_bytes());
    out.extend_from_slice(&CHANNELS.to_le_bytes());
    out.extend_from_slice(&sample_rate.to_le_bytes());
    out.extend_from_slice(&byte_rate.to_le_bytes());
    out.extend_from_slice(&block_align.to_le_bytes());
    out.extend_from_slice(&bits.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());

    for &sample in samples {
        let clamped = sample.clamp(-1.0, 1.0);
        match format {
            WavFormat::Pcm16 => {
                let value = (clamped * i16::MAX as f32) as i16;
                out.extend_from_slice(&value.to_le_bytes());
            }
            WavFormat::Pcm24 => {
                let value = (clamped * 8_388_607.0) as i32;
                out.extend_from_slice(&value.to_le_bytes()[..3]);
            }
            WavFormat::Float32 => {
                out.extend_from_slice(&clamped.to_le_bytes());
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_and_size_per_format() {
        let samples = vec![0.0f32; 10];
        assert_eq!(
            encode_wav(&samples, 44_100, WavFormat::Pcm16).len(),
            44 + 10 * 2
        );
        assert_eq!(
            encode_wav(&samples, 44_100, WavFormat::Pcm24).len(),
            44 + 10 * 3
        );
        assert_eq!(
            encode_wav(&samples, 44_100, WavFormat::Float32).len(),
            44 + 10 * 4
        );

        let wav = encode_wav(&samples, 44_100, WavFormat::Float32);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(u16::from_le_bytes([wav[20], wav[21]]), 3);
    }

    #[test]
    fn samples_are_clamped() {
        let wav = encode_wav(&[2.0], 44_100, WavFormat::Pcm16);
        let value = i16::from_le_bytes([wav[44], wav[45]]);
        assert_eq!(value, i16::MAX);
    }
}
