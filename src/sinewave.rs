use rodio::source::Source;

use std::f32::consts::PI;
use std::time::Duration;

/// A source that generates a clock like sine wave.
/// Always has a rate of 48kHz and one channel.

#[derive(Clone, Debug)]
pub struct SineWave {
    freq: f32,
    num_sample: usize,
    on_sec: f32,
    off_sec: f32,
    fade_in_sec: f32,
    fade_out_sec: f32,
    on_channel: u16,
    current_channel: u16,
}

impl SineWave {
    /// Builds a new `SineWave` with the given frequency.
    #[inline]
    pub fn new(freq: f32, on_sec: f32, off_sec: f32, on_channel: u16) -> SineWave {
        SineWave {
            freq,
            num_sample: 0,
            on_sec,
            off_sec,
            fade_in_sec: 0.025,
            fade_out_sec: 0.025,
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

            let current_pos_per_loop =
                self.num_sample % ((self.on_sec + self.off_sec) * 48000.0) as usize;

            let value = if (current_pos_per_loop as f32) < self.fade_in_sec * 48000.0 {
                let fade_in = current_pos_per_loop as f32 / (self.fade_in_sec * 48000.0);
                (self.freq * 2.0 * PI * (self.num_sample as f32 / 48000.0)).sin() * fade_in
            } else if (current_pos_per_loop as f32) < (self.on_sec - self.fade_out_sec) * 48000.0 {
                (self.freq * 2.0 * PI * (self.num_sample as f32 / 48000.0)).sin()
            } else if (current_pos_per_loop as f32) < self.on_sec * 48000.0 {
                let fade_out = 1.0
                    - (current_pos_per_loop as f32
                        - (self.on_sec - self.fade_out_sec) as f32 * 48000.0)
                        / (self.fade_out_sec * 48000.0);
                (self.freq * 2.0 * PI * (self.num_sample as f32 / 48000.0)).sin() * fade_out
            } else {
                0.0
            };
            Some(value)
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
