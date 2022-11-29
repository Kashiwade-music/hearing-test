use rodio::source::Source;

use std::f32::consts::PI;
use std::time::Duration;

/// A source that generates a clock like sine wave.
/// Always has a rate of 48kHz and one channel.

#[derive(Clone, Debug)]
pub struct SineWave {
    freq: f32,
    num_sample: usize,
    volume_vec: Vec<f32>,
    on_channel: u16,
    current_channel: u16,
}

impl SineWave {
    /// Builds a new `SineWave` with the given frequency.
    #[inline]
    pub fn new(freq: f32, on_sec: f32, off_sec: f32, on_channel: u16) -> SineWave {
        let mut volume_vec = Vec::new();
        let fade_in_sec = 0.025;
        let fade_out_sec = 0.025;

        // build volume vector
        let fade_in_samples = (fade_in_sec * 48000.0) as usize;
        let fade_out_samples = (fade_out_sec * 48000.0) as usize;
        let on_samples = (on_sec * 48000.0) as usize;
        let off_samples = (off_sec * 48000.0) as usize;

        for i in 0..fade_in_samples {
            volume_vec.push((i as f32 / fade_in_samples as f32).powf(2.0));
        }
        for _ in 0..on_samples - fade_in_samples - fade_out_samples {
            volume_vec.push(1.0);
        }
        for i in 0..fade_out_samples {
            volume_vec.push(1.0 - (i as f32 / fade_out_samples as f32).powf(2.0));
        }
        for _ in 0..off_samples {
            volume_vec.push(0.0);
        }

        SineWave {
            freq,
            num_sample: 0,
            volume_vec,
            on_channel,
            current_channel: 0,
        }
    }
}

impl Iterator for SineWave {
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<f32> {
        if self.current_channel == self.on_channel {
            self.current_channel = if self.current_channel == 0 { 1 } else { 0 };
            self.num_sample = self.num_sample.wrapping_add(1);

            let current_pos_per_loop = self.num_sample % (self.volume_vec.len()) as usize;

            Some(if self.volume_vec[current_pos_per_loop] > 0.0 {
                self.volume_vec[current_pos_per_loop]
                    * (self.freq * 2.0 * PI * self.num_sample as f32 / 48000.0).sin()
            } else {
                0.0
            })
        } else {
            self.current_channel = if self.current_channel == 0 { 1 } else { 0 };

            Some(0.0)
        }
    }
}

impl Source for SineWave {
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    #[inline]
    fn channels(&self) -> u16 {
        2
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        48000
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
