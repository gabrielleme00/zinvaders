pub struct Mmu {
    memory: [u8; 0x10000],
}

impl Mmu {
    pub fn new() -> Self {
        Self {
            memory: [0; 0x10000],
        }
    }

    pub fn load_rom(&mut self, rom: &[u8], base_addr: usize) {
        let start = base_addr;
        let end = start + rom.len();
        if end > self.memory.len() {
            panic!("ROM size exceeds memory limits");
        }
        self.memory[start..end].copy_from_slice(rom);

    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    pub fn read_word(&self, addr: u16) -> u16 {
        let low = self.read_byte(addr) as u16;
        let high = self.read_byte(addr.wrapping_add(1)) as u16;
        (high << 8) | low
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        self.memory[addr as usize] = value;
    }

    pub fn write_word(&mut self, addr: u16, value: u16) {
        let [low, high] = value.to_le_bytes();
        self.write_byte(addr, low);
        self.write_byte(addr.wrapping_add(1), high);
    }
}
