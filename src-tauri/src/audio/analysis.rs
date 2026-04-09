// SPDX-License-Identifier: Apache-2.0
//
// Copyright 2026 Victor Palade
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use rustfft::num_complex::Complex32;
use rustfft::{Fft, FftPlanner};
use std::sync::Arc;

const FFT_SIZE: usize = 1024;
const MEL_BAND_COUNT: usize = 24;
const MIN_ANALYSIS_HZ: f32 = 30.0;
const MAX_ANALYSIS_HZ: f32 = 12_000.0;

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
    previous_melbands: Vec<f32>,
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
            previous_melbands: vec![0.0; MEL_BAND_COUNT],
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
            &mut self.previous_melbands,
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
    previous_melbands: &mut [f32],
    sample_rate: u32,
) -> (f32, f32, f32, f32, f32) {
    let sample_rate = sample_rate as f32;
    let bin_hz = sample_rate / FFT_SIZE as f32;
    let half = spectrum.len() / 2;
    let filters = mel_filters(
        sample_rate,
        MEL_BAND_COUNT,
        MIN_ANALYSIS_HZ,
        MAX_ANALYSIS_HZ,
    );
    let mut melbands = vec![0.0_f32; MEL_BAND_COUNT];
    let mut melband_weight = vec![0.0_f32; MEL_BAND_COUNT];
    let mut flux_sum = 0.0_f32;
    let mut flux_weight_sum = 0.0_f32;

    for (index, value) in spectrum.iter().take(half).enumerate().skip(1) {
        let hz = index as f32 * bin_hz;
        if hz < MIN_ANALYSIS_HZ || hz > MAX_ANALYSIS_HZ {
            continue;
        }
        let magnitude = value.norm();

        for (band, filter) in filters.iter().enumerate() {
            let weight = triangular_weight(hz, *filter);
            if weight <= 0.0 {
                continue;
            }
            melbands[band] += magnitude * weight;
            melband_weight[band] += weight;
        }
    }

    for band in 0..MEL_BAND_COUNT {
        let energy = if melband_weight[band] > 0.0 {
            melbands[band] / melband_weight[band]
        } else {
            0.0
        };
        // Light high-band compensation avoids bass-only dominance from pink-spectrum mixes.
        let ratio = band as f32 / (MEL_BAND_COUNT.saturating_sub(1)) as f32;
        let compensated = energy * (0.90 + ratio * 0.36);
        melbands[band] = compensated;

        let previous = previous_melbands.get(band).copied().unwrap_or_default();
        let positive_flux = (compensated - previous).max(0.0);
        if let Some(slot) = previous_melbands.get_mut(band) {
            *slot = compensated;
        }

        // Mid/upper-mid transients should dominate pulse/flash behavior.
        let flux_weight = if ratio < 0.22 {
            0.70
        } else if ratio < 0.80 {
            1.35
        } else {
            1.00
        };
        flux_sum += positive_flux * flux_weight;
        flux_weight_sum += flux_weight;
    }

    let bass = mel_band_mean(&melbands, 0, 6);
    let mid = mel_band_mean(&melbands, 5, 15);
    let attack = mel_band_mean(&melbands, 10, 19);
    let treble = mel_band_mean(&melbands, 16, MEL_BAND_COUNT);
    let onset = if flux_weight_sum <= f32::EPSILON {
        0.0
    } else {
        normalize_onset_flux(flux_sum / flux_weight_sum)
    };

    (
        normalize_band_energy(bass),
        normalize_band_energy(mid),
        normalize_attack_energy(attack),
        normalize_band_energy(treble),
        onset,
    )
}

fn mel_filters(sample_rate: f32, count: usize, min_hz: f32, max_hz: f32) -> Vec<(f32, f32, f32)> {
    let upper = max_hz.min(sample_rate * 0.5 - 1.0).max(min_hz + 20.0);
    let min_mel = hz_to_mel(min_hz.max(1.0));
    let max_mel = hz_to_mel(upper);
    let step = (max_mel - min_mel) / (count + 1) as f32;
    (0..count)
        .map(|index| {
            let lower = mel_to_hz(min_mel + step * index as f32);
            let center = mel_to_hz(min_mel + step * (index + 1) as f32);
            let upper = mel_to_hz(min_mel + step * (index + 2) as f32);
            (lower, center, upper)
        })
        .collect()
}

fn triangular_weight(hz: f32, (lower, center, upper): (f32, f32, f32)) -> f32 {
    if hz <= lower || hz >= upper {
        return 0.0;
    }

    if hz <= center {
        let width = (center - lower).max(1e-6);
        ((hz - lower) / width).clamp(0.0, 1.0)
    } else {
        let width = (upper - center).max(1e-6);
        ((upper - hz) / width).clamp(0.0, 1.0)
    }
}

fn mel_band_mean(values: &[f32], start: usize, end: usize) -> f32 {
    let clamped_start = start.min(values.len());
    let clamped_end = end.min(values.len()).max(clamped_start + 1);
    let slice = &values[clamped_start..clamped_end];
    slice.iter().copied().sum::<f32>() / slice.len() as f32
}

fn hz_to_mel(hz: f32) -> f32 {
    2_595.0 * (1.0 + hz / 700.0).log10()
}

fn mel_to_hz(mel: f32) -> f32 {
    700.0 * (10_f32.powf(mel / 2_595.0) - 1.0)
}

fn normalize_band_energy(average: f32) -> f32 {
    (average.sqrt() * 0.16).clamp(0.0, 1.0)
}

fn normalize_onset_flux(average: f32) -> f32 {
    (average.sqrt() * 0.50).clamp(0.0, 1.0)
}

fn normalize_attack_energy(average: f32) -> f32 {
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
