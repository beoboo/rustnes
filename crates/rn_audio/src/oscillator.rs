#[derive(Debug, Clone, Default)]
pub enum Waveform {
    #[default]
    Sine,
    Square(f32), // Duty cycle
    Saw,
    Triangle,
}

pub struct Oscillator {
    pub sample_rate: f32,
    pub waveform: Waveform,
    pub current_sample_index: f32,
    pub frequency: f32,
}

impl Oscillator {
    pub fn new(sample_rate: f32, waveform: Waveform, frequency: f32) -> Self {
        Self {
            sample_rate,
            waveform,
            current_sample_index: 0.0,
            frequency,
        }
    }

    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
    }

    pub fn tick(&mut self) -> f32 {
        match self.waveform {
            Waveform::Sine => self.sine_wave(),
            Waveform::Square(duty_cycle) => self.square_wave(duty_cycle),
            Waveform::Saw => self.saw_wave(),
            Waveform::Triangle => self.triangle_wave(),
        }
    }

    fn sine_wave(&mut self) -> f32 {
        self.advance_sample();
        self.calculate_sine_output_from_freq(self.frequency)
    }

    fn square_wave(&mut self, duty_cycle: f32) -> f32 {
        self.advance_sample();
        
        let cycle_position = (self.current_sample_index * self.frequency / self.sample_rate) % 1.0;
        
        if cycle_position < duty_cycle {
            1.0
        } else {
            -1.0
        }
    }

    fn saw_wave(&mut self) -> f32 {
        self.generative_waveform(1, 1.0)
    }

    fn triangle_wave(&mut self) -> f32 {
        self.generative_waveform(2, 2.0)
    }

    fn generative_waveform(&mut self, harmonic_index_increment: i32, gain_exponent: f32) -> f32 {
        self.advance_sample();
        let mut output = 0.0;
        let mut i = 1;
        
        while !self.is_multiple_of_freq_above_nyquist(i as f32) {
            let gain = 1.0 / (i as f32).powf(gain_exponent);
            output += gain * self.calculate_sine_output_from_freq(self.frequency * i as f32);
            i += harmonic_index_increment;
        }
        output
    }

    fn advance_sample(&mut self) {
        self.current_sample_index = (self.current_sample_index + 1.0) % self.sample_rate;
    }

    fn is_multiple_of_freq_above_nyquist(&self, multiple: f32) -> bool {
        self.frequency * multiple > self.sample_rate / 2.0
    }

    fn calculate_sine_output_from_freq(&self, freq: f32) -> f32 {
        let two_pi = 2.0 * std::f32::consts::PI;
        (self.current_sample_index * freq * two_pi / self.sample_rate).sin()
    }
}
