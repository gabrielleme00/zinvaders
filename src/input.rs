use minifb::Key;

/// Represents the input state for Space Invaders controls
#[derive(Default)]
pub struct Input {
    // Port 1 bits (read from port 1)
    pub coin: bool,     // Bit 0
    pub p2_start: bool, // Bit 1
    pub p1_start: bool, // Bit 2
    pub p1_shoot: bool, // Bit 4
    pub p1_left: bool,  // Bit 5
    pub p1_right: bool, // Bit 6

    // Port 2 bits (read from port 2) - DIP switches
    pub dip3: bool,     // Bit 0 - Ships per game
    pub dip5: bool,     // Bit 1 - Extra ship at
    pub tilt: bool,     // Bit 2
    pub dip6: bool,     // Bit 3 - Extra ship at
    pub p2_shoot: bool, // Bit 4
    pub p2_left: bool,  // Bit 5
    pub p2_right: bool, // Bit 6
    pub dip7: bool,     // Bit 7 - Coin info
}

impl Input {
    pub fn new() -> Self {
        Self {
            // Default DIP switch settings (3 ships, extra ship at 1500)
            dip3: true,
            dip5: false,
            dip6: false,
            dip7: false,
            ..Default::default()
        }
    }

    /// Update input state from keyboard
    pub fn update(&mut self, keys: &[Key]) {
        // Player 1 controls
        self.p1_left = keys.contains(&Key::Left) || keys.contains(&Key::A);
        self.p1_right = keys.contains(&Key::Right) || keys.contains(&Key::D);
        self.p1_shoot = keys.contains(&Key::Space) || keys.contains(&Key::W);

        // Player 2 controls (optional)
        self.p2_left = keys.contains(&Key::J);
        self.p2_right = keys.contains(&Key::L);
        self.p2_shoot = keys.contains(&Key::I);

        // Game controls
        self.coin = keys.contains(&Key::Key3);
        self.p1_start = keys.contains(&Key::Key1);
        self.p2_start = keys.contains(&Key::Key2);

        // Tilt (cheat detection)
        self.tilt = keys.contains(&Key::T);
    }

    /// Get port 1 byte value (player inputs)
    pub fn get_port1(&self) -> u8 {
        let mut value = 0u8;
        if self.coin {
            value |= 0x01;
        }
        if self.p2_start {
            value |= 0x02;
        }
        if self.p1_start {
            value |= 0x04;
        }
        // Bit 3 is always 1
        value |= 0x08;
        if self.p1_shoot {
            value |= 0x10;
        }
        if self.p1_left {
            value |= 0x20;
        }
        if self.p1_right {
            value |= 0x40;
        }
        value
    }

    /// Get port 2 byte value (DIP switches and player 2)
    pub fn get_port2(&self) -> u8 {
        let mut value = 0u8;
        if self.dip3 {
            value |= 0x01;
        }
        if self.dip5 {
            value |= 0x02;
        }
        if self.tilt {
            value |= 0x04;
        }
        if self.dip6 {
            value |= 0x08;
        }
        if self.p2_shoot {
            value |= 0x10;
        }
        if self.p2_left {
            value |= 0x20;
        }
        if self.p2_right {
            value |= 0x40;
        }
        if self.dip7 {
            value |= 0x80;
        }
        value
    }
}
