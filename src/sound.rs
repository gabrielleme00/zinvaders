use rodio::{OutputStream, OutputStreamHandle, Sink, Source};
use std::sync::Arc;
use std::time::Duration;

/// Sound effects for Space Invaders
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum SoundEffect {
    Ufo = 0,           // Port 3, bit 0 - UFO (repeating)
    Shot = 1,          // Port 3, bit 1 - Player shot
    PlayerDie = 2,     // Port 3, bit 2 - Player explosion
    InvaderDie = 3,    // Port 3, bit 3 - Invader explosion
    ExtendedPlay = 4,  // Port 3, bit 4 - Extended play (extra ship)
    // Port 5 sounds
    FleetMove1 = 5,    // Port 5, bit 0 - Fleet movement 1
    FleetMove2 = 6,    // Port 5, bit 1 - Fleet movement 2
    FleetMove3 = 7,    // Port 5, bit 2 - Fleet movement 3
    FleetMove4 = 8,    // Port 5, bit 3 - Fleet movement 4
    UfoHit = 9,        // Port 5, bit 4 - UFO hit
}

pub struct SoundSystem {
    _stream: OutputStream,
    stream_handle: Arc<OutputStreamHandle>,
    last_port3: u8,
    last_port5: u8,
}

impl SoundSystem {
    pub fn new() -> Result<Self, String> {
        let (stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| format!("Failed to initialize audio: {}", e))?;
        
        Ok(Self {
            _stream: stream,
            stream_handle: Arc::new(stream_handle),
            last_port3: 0,
            last_port5: 0,
        })
    }

    pub fn update(&mut self, port3: u8, port5: u8) {
        // Check port 3 bits
        for bit in 0..5 {
            let mask = 1 << bit;
            let was_set = (self.last_port3 & mask) != 0;
            let is_set = (port3 & mask) != 0;

            // Trigger on rising edge
            if !was_set && is_set {
                self.play_sound(bit);
            }
        }

        // Check port 5 bits
        for bit in 0..5 {
            let mask = 1 << bit;
            let was_set = (self.last_port5 & mask) != 0;
            let is_set = (port5 & mask) != 0;

            // Trigger on rising edge
            if !was_set && is_set {
                self.play_sound(bit + 5);
            }
        }

        self.last_port3 = port3;
        self.last_port5 = port5;
    }

    fn play_sound(&self, sound_id: usize) {
        if let Ok(sink) = Sink::try_new(&self.stream_handle) {
            match sound_id {
                0 => self.generate_ufo(&sink),           // UFO
                1 => self.generate_shot(&sink),          // Player shot
                2 => self.generate_player_die(&sink),    // Player die
                3 => self.generate_invader_die(&sink),   // Invader die
                4 => self.generate_extended_play(&sink), // Extended play
                5 => self.generate_fleet_move1(&sink),   // Fleet move 1
                6 => self.generate_fleet_move2(&sink),   // Fleet move 2
                7 => self.generate_fleet_move3(&sink),   // Fleet move 3
                8 => self.generate_fleet_move4(&sink),   // Fleet move 4
                9 => self.generate_ufo_hit(&sink),       // UFO hit
                _ => return,
            }
            // Detach the sink so it plays in the background without blocking
            sink.detach();
        }
    }

    // Generate sound effects using square waves
    fn generate_ufo(&self, sink: &Sink) {
        // UFO has a warbling/pulsing effect
        for _ in 0..4 {
            let source = SquareWave::new(200.0, Duration::from_millis(50));
            sink.append(source);
            let source = SquareWave::new(240.0, Duration::from_millis(50));
            sink.append(source);
        }
    }

    fn generate_shot(&self, sink: &Sink) {
        // Sharp descending "pew" sound
        for i in 0..8 {
            let freq = 1200.0 - (i as f32 * 140.0);
            let source = SquareWave::new(freq, Duration::from_millis(10));
            sink.append(source);
        }
    }

    fn generate_player_die(&self, sink: &Sink) {
        // Distinctive "waaah" descending explosion
        for i in 0..40 {
            let freq = 400.0 - (i as f32 * 8.0);
            let duration = if i < 10 { 15 } else { 12 };
            let source = SquareWave::new(freq.max(50.0), Duration::from_millis(duration));
            sink.append(source);
        }
    }

    fn generate_invader_die(&self, sink: &Sink) {
        // Quick metallic "clank" sound - shorter descending tone
        for i in 0..12 {
            let freq = 180.0 - (i as f32 * 12.0);
            let source = SquareWave::new(freq.max(40.0), Duration::from_millis(15));
            sink.append(source);
        }
    }

    fn generate_extended_play(&self, sink: &Sink) {
        // Celebratory rising then steady tone
        for i in 0..5 {
            let freq = 400.0 + (i as f32 * 30.0);
            let source = SquareWave::new(freq, Duration::from_millis(40));
            sink.append(source);
        }
        let source = SquareWave::new(550.0, Duration::from_millis(200));
        sink.append(source);
    }

    fn generate_fleet_move1(&self, sink: &Sink) {
        // Deep thump - lowest note in the 4-note sequence
        let source = SquareWave::new(98.0, Duration::from_millis(120));
        sink.append(source);
    }

    fn generate_fleet_move2(&self, sink: &Sink) {
        // Second note - slightly higher
        let source = SquareWave::new(110.0, Duration::from_millis(120));
        sink.append(source);
    }

    fn generate_fleet_move3(&self, sink: &Sink) {
        // Third note
        let source = SquareWave::new(123.0, Duration::from_millis(120));
        sink.append(source);
    }

    fn generate_fleet_move4(&self, sink: &Sink) {
        // Highest note - tension building
        let source = SquareWave::new(139.0, Duration::from_millis(120));
        sink.append(source);
    }

    fn generate_ufo_hit(&self, sink: &Sink) {
        // Dramatic explosion with rapid descending pitch and some noise-like quality
        for i in 0..30 {
            let freq = 800.0 - (i as f32 * 24.0);
            let duration = if i < 15 { 12 } else { 10 };
            let source = SquareWave::new(freq.max(60.0), Duration::from_millis(duration));
            sink.append(source);
        }
    }
}

// Square wave generator for more authentic arcade sounds
struct SquareWave {
    freq: f32,
    num_samples: usize,
    current_sample: usize,
}

impl SquareWave {
    fn new(freq: f32, duration: Duration) -> Self {
        let sample_rate = 48000;
        let num_samples = (duration.as_secs_f32() * sample_rate as f32) as usize;
        Self {
            freq,
            num_samples,
            current_sample: 0,
        }
    }
}

impl Iterator for SquareWave {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_sample >= self.num_samples {
            return None;
        }

        let sample_rate = 48000.0;
        let t = self.current_sample as f32 / sample_rate;
        // Square wave: alternates between -0.2 and 0.2
        let phase = (t * self.freq) % 1.0;
        let value = if phase < 0.5 { 0.2 } else { -0.2 };

        self.current_sample += 1;
        Some(value)
    }
}

impl Source for SquareWave {
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.num_samples - self.current_sample)
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        48000
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs_f32(self.num_samples as f32 / 48000.0))
    }
}
