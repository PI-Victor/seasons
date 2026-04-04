use rustfft::num_complex::Complex32;
use rustfft::{Fft, FftPlanner};
use std::sync::Arc;

const FFT_SIZE: usize = 1024;

#[derive(Clone, Copy, Debug, Default)]
pub struct AudioFeatures {
    pub level: f32,
    pub bass: f32,
    pub mid: f32,
    pub attack: f32,
    pub treble: f32,
    pub onset: f32,
}

pub struct AudioAnalyzer {
    fft: Arc<dyn Fft<f32>>,
    window: Vec<f32>,
    spectrum: Vec<Complex32>,
    scratch: Vec<Complex32>,
    previous_magnitudes: Vec<f32>,
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
            previous_magnitudes: vec![0.0; FFT_SIZE / 2],
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

        let (bass, mid, attack, treble, onset) = spectral_features(
            &self.spectrum,
            &mut self.previous_magnitudes,
            sample_rate.max(1),
        );

        AudioFeatures {
            level,
            bass,
            mid,
            attack,
            treble,
            onset,
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

fn spectral_features(
    spectrum: &[Complex32],
    previous_magnitudes: &mut [f32],
    sample_rate: u32,
) -> (f32, f32, f32, f32, f32) {
    let bin_hz = sample_rate as f32 / FFT_SIZE as f32;
    let half = spectrum.len() / 2;

    let mut bass_sum = 0.0_f32;
    let mut bass_bins = 0_usize;
    let mut mid_sum = 0.0_f32;
    let mut mid_bins = 0_usize;
    let mut attack_sum = 0.0_f32;
    let mut attack_bins = 0_usize;
    let mut treble_sum = 0.0_f32;
    let mut treble_bins = 0_usize;
    let mut onset_sum = 0.0_f32;
    let mut onset_bins = 0_usize;

    for (index, value) in spectrum.iter().take(half).enumerate().skip(1) {
        let hz = index as f32 * bin_hz;
        let magnitude = value.norm();
        let previous = previous_magnitudes.get(index).copied().unwrap_or_default();
        let positive_flux = (magnitude - previous).max(0.0);
        if let Some(slot) = previous_magnitudes.get_mut(index) {
            *slot = magnitude;
        }

        if (20.0..150.0).contains(&hz) {
            bass_sum += magnitude;
            bass_bins += 1;
        } else if (150.0..1_200.0).contains(&hz) {
            mid_sum += magnitude;
            mid_bins += 1;
        } else if (1_200.0..=4_500.0).contains(&hz) {
            attack_sum += magnitude;
            attack_bins += 1;
        } else if (4_500.0..=12_000.0).contains(&hz) {
            treble_sum += magnitude;
            treble_bins += 1;
        }

        if (30.0..=6_000.0).contains(&hz) {
            onset_sum += positive_flux;
            onset_bins += 1;
        }
    }

    (
        normalize_band_energy(bass_sum, bass_bins),
        normalize_band_energy(mid_sum, mid_bins),
        normalize_attack_energy(attack_sum, attack_bins),
        normalize_band_energy(treble_sum, treble_bins),
        normalize_onset_flux(onset_sum, onset_bins),
    )
}

fn normalize_band_energy(sum: f32, bins: usize) -> f32 {
    if bins == 0 {
        return 0.0;
    }

    let average = sum / bins as f32;
    (average.sqrt() * 0.16).clamp(0.0, 1.0)
}

fn normalize_onset_flux(sum: f32, bins: usize) -> f32 {
    if bins == 0 {
        return 0.0;
    }

    let average = sum / bins as f32;
    (average.sqrt() * 0.34).clamp(0.0, 1.0)
}

fn normalize_attack_energy(sum: f32, bins: usize) -> f32 {
    if bins == 0 {
        return 0.0;
    }

    let average = sum / bins as f32;
    (average.sqrt() * 0.22).clamp(0.0, 1.0)
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

    #[test]
    fn analyzer_reports_onset_for_a_sudden_attack() {
        let mut analyzer = AudioAnalyzer::new();
        let silence = encode_interleaved(&vec![0.0_f32; 2048]);
        let attack = encode_interleaved(&vec![0.9_f32; 2048]);

        let _ = analyzer.analyze_interleaved_f32(&silence, 1, 48_000);
        let features = analyzer.analyze_interleaved_f32(&attack, 1, 48_000);

        assert!(features.onset > 0.05);
    }

    #[test]
    fn analyzer_separates_attack_band_from_bass() {
        let mut analyzer = AudioAnalyzer::new();
        let sample_rate = 48_000.0_f32;
        let samples = (0..2048)
            .map(|index| {
                let time = index as f32 / sample_rate;
                (std::f32::consts::TAU * 2_200.0 * time).sin() * 0.8
            })
            .collect::<Vec<_>>();
        let encoded = encode_interleaved(&samples);

        let features = analyzer.analyze_interleaved_f32(&encoded, 1, 48_000);

        assert!(features.attack > features.bass);
        assert!(features.attack > 0.05);
    }
}
