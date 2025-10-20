pub struct Ports {
    /// Shift register for reading (port 3)
    shift_register: u16,
    /// Shift amount (port 2)
    shift_amount: u8,
    /// Input port 1 value
    pub port1: u8,
    /// Input port 2 value
    pub port2: u8,
    /// Sound port 3 value (written by CPU)
    pub port3: u8,
    /// Sound port 5 value (written by CPU)
    pub port5: u8,
}

impl Ports {
    pub fn new() -> Self {
        Self {
            shift_register: 0,
            shift_amount: 0,
            port1: 0x08, // Bit 3 is always 1
            port2: 0,
            port3: 0,
            port5: 0,
        }
    }

    pub fn read(&self, port: u8) -> u8 {
        match port {
            1 => self.port1,
            2 => self.port2,
            3 => {
                // Read shift register result
                ((self.shift_register >> (8 - self.shift_amount)) & 0xFF) as u8
            }
            _ => 0,
        }
    }

    pub fn write(&mut self, port: u8, value: u8) {
        match port {
            2 => {
                // Set shift amount (bits 0-2)
                self.shift_amount = value & 0x07;
            }
            4 => {
                // Shift register data
                self.shift_register = (self.shift_register >> 8) | ((value as u16) << 8);
            }
            3 => {
                // Sound port 3
                self.port3 = value;
            }
            5 => {
                // Sound port 5
                self.port5 = value;
            }
            6 => {
                // Watchdog (can be ignored)
            }
            _ => {}
        }
    }
}
