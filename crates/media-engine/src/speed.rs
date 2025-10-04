// crates/media-engine/src/speed.rs
/// Time-stretching for speed control WITHOUT pitch change
pub struct TimeStretcher {
    sonic: sonic::Stream,
    speed: f32,
}

impl TimeStretcher {
    pub fn new(sample_rate: u32, channels: u8) -> Self {
        let mut sonic = sonic::Stream::new(sample_rate as i32, channels as i32);
        sonic.set_speed(1.0);
        Self { sonic, speed: 1.0 }
    }

    pub fn set_speed(&mut self, speed: f32) -> EngineResult<()> {
        // Validate speed range (0.5x to 3.0x)
        if !(0.5..=3.0).contains(&speed) {
            return Err(EngineError::InvalidSpeed(speed));
        }

        self.sonic.set_speed(speed);
        self.speed = speed;
        Ok(())
    }

    /// Process audio with time-stretching
    /// Quality is maintained even at extreme speeds
    pub fn process(&mut self, input: &[i16]) -> Vec<i16> {
        self.sonic.write_i16_to_stream(input);

        let output_size = self.sonic.samples_available();
        let mut output = vec![0i16; output_size];
        self.sonic.read_i16_from_stream(&mut output);

        output
    }
}