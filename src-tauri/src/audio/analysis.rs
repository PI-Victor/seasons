use rustfft::num_complex::Complex32;
use rustfft::{Fft, FftPlanner};
use std::sync::Arc;

const FFT_SIZE: usize = 1024;

#[derive(Clone, Copy, Debug, Default)]
pub struct AudioFeatures {
    pub level: f32,
    pub bass: f32,
    pub mid: f32,
    pub treble: f32,
}

pub struct AudioAnalyzer {
    fft: Arc<dyn Fft<f32>>,
    window: Vec<f32>,
    spectrum: Vec<Complex32>,
    scratch: Vec<Complex32>,
}

impl AudioAnalyzer {
    pub fn new() -> Self {
        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(FFT_SIZE);
        let scratch = vec![Complex32::default(); fft.get_inplace_scratch_len()];
        let window = (0..FFT_SIZE)
            .map(|index| {
                let ratio = index as f32 / (FFT_SIZE.saturating_sub(1)) as f32;
                0.5 - 0.5 * (std::f32::consts::TAU * ratio).cos()
            })
            .collect::<Vec<_>>();

        Self {
            fft,
            window,
            spectrum: vec![Complex32::default(); FFT_SIZE],
            scratch,
        }
    }

    pub fn analyze_interleaved_f32(
        &mut self,
        samples: &[u8],
        channel_count: usize,
        sample_rate: u32,
    ) -> AudioFeatures {
        if channel_count == 0 || samples.len() < std::mem::size_of::<f32>() {
            return AudioFeatures::default();
        }

        let total_samples = samples.len() / std::mem::size_of::<f32>();
        let frame_count = total_samples / channel_count;
        if frame_count == 0 {
            return AudioFeatures::default();
        }

        let mut rms_sum = 0.0_f32;
        let mut mono_count = 0_usize;

        for index in 0..FFT_SIZE {
            let mono_sample = if index < frame_count {
                interleaved_mono_sample(samples, channel_count, index)
            } else {
                0.0
            };

            if index < frame_count {
                rms_sum += mono_sample * mono_sample;
                mono_count += 1;
            }

            self.spectrum[index] = Complex32::new(mono_sample * self.window[index], 0.0);
        }

        let level = if mono_count == 0 {
            0.0
        } else {
            (rms_sum / mono_count as f32).sqrt().clamp(0.0, 1.0)
        };

        self.fft
            .process_with_scratch(&mut self.spectrum, &mut self.scratch);

        let (bass, mid, treble) = band_levels(&self.spectrum, sample_rate.max(1));

        AudioFeatures {
            level,
            bass,
            mid,
            treble,
        }
    }
}

fn interleaved_mono_sample(samples: &[u8], channel_count: usize, frame_index: usize) -> f32 {
    let mut sum = 0.0_f32;
    let mut read_channels = 0_usize;

    for channel in 0..channel_count {
        let sample_index = frame_index * channel_count + channel;
        let start = sample_index * std::mem::size_of::<f32>();
        let end = start + std::mem::size_of::<f32>();
        if end > samples.len() {
            break;
        }

        let mut encoded = [0_u8; 4];
        encoded.copy_from_slice(&samples[start..end]);
        sum += f32::from_le_bytes(encoded);
        read_channels += 1;
    }

    if read_channels == 0 {
        0.0
    } else {
        sum / read_channels as f32
    }
}

fn band_levels(spectrum: &[Complex32], sample_rate: u32) -> (f32, f32, f32) {
    let bin_hz = sample_rate as f32 / FFT_SIZE as f32;
    let half = spectrum.len() / 2;

    let mut bass_sum = 0.0_f32;
    let mut bass_bins = 0_usize;
    let mut mid_sum = 0.0_f32;
    let mut mid_bins = 0_usize;
    let mut treble_sum = 0.0_f32;
    let mut treble_bins = 0_usize;

    for (index, value) in spectrum.iter().take(half).enumerate().skip(1) {
        let hz = index as f32 * bin_hz;
        let magnitude = value.norm();

        if (20.0..250.0).contains(&hz) {
            bass_sum += magnitude;
            bass_bins += 1;
        } else if (250.0..=2_000.0).contains(&hz) {
            mid_sum += magnitude;
            mid_bins += 1;
        } else if (2_000.0..=12_000.0).contains(&hz) {
            treble_sum += magnitude;
            treble_bins += 1;
        }
    }

    (
        normalize_band_energy(bass_sum, bass_bins),
        normalize_band_energy(mid_sum, mid_bins),
        normalize_band_energy(treble_sum, treble_bins),
    )
}

fn normalize_band_energy(sum: f32, bins: usize) -> f32 {
    if bins == 0 {
        return 0.0;
    }

    let average = sum / bins as f32;
    (average.sqrt() * 0.16).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::AudioAnalyzer;

    fn encode_interleaved(samples: &[f32]) -> Vec<u8> {
        samples
            .iter()
            .flat_map(|sample| sample.to_le_bytes())
            .collect::<Vec<_>>()
    }

    #[test]
    fn analyzer_reports_level_for_non_silent_signal() {
        let mut analyzer = AudioAnalyzer::new();
        let samples = vec![0.5_f32; 2048];
        let encoded = encode_interleaved(&samples);

        let features = analyzer.analyze_interleaved_f32(&encoded, 1, 48_000);

        assert!(features.level > 0.1);
    }
}
