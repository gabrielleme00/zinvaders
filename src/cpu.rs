use crate::bdos::Bdos;
use crate::mmu::Mmu;
use crate::ports::Ports;

const BDOS_CALL_ADDR: u16 = 0x0005;

const OPCODE_CYCLES: [u8; 256] = [
    04, 10, 07, 05, 05, 05, 07, 04, 04, 10, 07, 05, 05, 05, 07, 04, // 00..0F
    04, 10, 07, 05, 05, 05, 07, 04, 04, 10, 07, 05, 05, 05, 07, 04, // 00..1F
    04, 10, 16, 05, 05, 05, 07, 04, 04, 10, 16, 05, 05, 05, 07, 04, // 20..2F
    04, 10, 13, 05, 10, 10, 10, 04, 04, 10, 13, 05, 05, 05, 07, 04, // 30..3F
    05, 05, 05, 05, 05, 05, 07, 05, 05, 05, 05, 05, 05, 05, 07, 05, // 40..4F
    05, 05, 05, 05, 05, 05, 07, 05, 05, 05, 05, 05, 05, 05, 07, 05, // 50..5F
    05, 05, 05, 05, 05, 05, 07, 05, 05, 05, 05, 05, 05, 05, 07, 05, // 60..6F
    07, 07, 07, 07, 07, 07, 07, 07, 05, 05, 05, 05, 05, 05, 07, 05, // 70..7F
    04, 04, 04, 04, 04, 04, 07, 04, 04, 04, 04, 04, 04, 04, 07, 04, // 80..8F
    04, 04, 04, 04, 04, 04, 07, 04, 04, 04, 04, 04, 04, 04, 07, 04, // 90..9F
    04, 04, 04, 04, 04, 04, 07, 04, 04, 04, 04, 04, 04, 04, 07, 04, // A0..AF
    04, 04, 04, 04, 04, 04, 07, 04, 04, 04, 04, 04, 04, 04, 07, 04, // B0..BF
    05, 10, 10, 10, 11, 11, 07, 11, 05, 10, 10, 10, 11, 17, 07, 11, // C0..CF
    05, 10, 10, 10, 11, 11, 07, 11, 05, 10, 10, 10, 11, 17, 07, 11, // D0..DF
    05, 10, 10, 18, 11, 11, 07, 11, 05, 05, 10, 05, 11, 17, 07, 11, // E0..EF
    05, 10, 10, 04, 11, 11, 07, 11, 05, 05, 10, 04, 11, 17, 07, 11, // F0..FF
];

/// Represents the Intel 8080 CPU flags.
#[derive(Default)]
pub struct Flags {
    pub z: bool,  // Zero
    pub s: bool,  // Sign
    pub p: bool,  // Parity
    pub cy: bool, // Carry
    pub ac: bool, // Auxiliary Carry
}

impl Flags {
    /// Returns the flags as a single byte.
    pub fn to_byte(&self) -> u8 {
        let mut byte = 0u8;
        if self.s {
            byte |= 0x80;
        }
        if self.z {
            byte |= 0x40;
        }
        if self.ac {
            byte |= 0x10;
        }
        if self.p {
            byte |= 0x04;
        }
        if self.cy {
            byte |= 0x01;
        }
        byte
    }

    /// Sets the flags from a single byte.
    pub fn set_from_byte(&mut self, byte: u8) {
        self.s = (byte & 0x80) != 0;
        self.z = (byte & 0x40) != 0;
        self.ac = (byte & 0x10) != 0;
        self.p = (byte & 0x04) != 0;
        self.cy = (byte & 0x01) != 0;
    }
}

/// Represents the Intel 8080 CPU and provides methods to emulate CPU cycles.
#[derive(Default)]
pub struct Cpu {
    // 8-bit registers
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,

    // 16-bit registers
    pub sp: u16,
    pub pc: u16,

    // Flags (Intel 8080: Sign, Zero, Aux Carry, Parity, Carry)
    pub flags: Flags,
    pub ime: bool, // Interrupt Master Enable

    /// Total cycles executed by the CPU
    pub cycles: u64,

    /// Halted state
    pub halted: bool,
}

impl Cpu {
    /// Creates a new CPU instance with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Prints the CPU state.
    pub fn print_state(&self, mmu: &Mmu) {
        let next_bytes = [
            mmu.read_byte(self.pc),
            mmu.read_byte(self.pc.wrapping_add(1)),
            mmu.read_byte(self.pc.wrapping_add(2)),
            mmu.read_byte(self.pc.wrapping_add(3)),
        ];

        println!(
            "PC: {:04X}, AF: {:04X}, BC: {:04X}, DE: {:04X}, HL: {:04X}, SP: {:04X}, CYC: {}\t({:02X} {:02X} {:02X} {:02X})",
            self.pc,
            self.get_af(),
            self.get_bc(),
            self.get_de(),
            self.get_hl(),
            self.sp,
            self.cycles,
            next_bytes[0],
            next_bytes[1],
            next_bytes[2],
            next_bytes[3]
        );
    }

    /// Emulates a single CPU step and returns the number of cycles consumed.
    pub fn step(&mut self, mmu: &mut Mmu, ports: &mut Ports) -> u64 {
        if self.halted {
            return 4;
        }

        // Handle CP/M BDOS call at address 0x0005
        if self.pc == BDOS_CALL_ADDR {
            Bdos::handle_call(self.c, self.get_de(), mmu);
        }

        let opcode = self.fetch_byte(mmu);
        let cycles = self.execute_opcode(opcode, mmu, ports) as u64;
        self.cycles = self.cycles.wrapping_add(cycles);
        cycles
    }
    /// Fetches the next byte from memory and increments the program counter.
    fn fetch_byte(&mut self, mmu: &Mmu) -> u8 {
        let byte = mmu.read_byte(self.pc);
        self.pc = self.pc.wrapping_add(1);
        byte
    }

    /// Fetches the next word (2 bytes) from memory and increments the program counter.
    fn fetch_word(&mut self, mmu: &Mmu) -> u16 {
        let low = self.fetch_byte(mmu);
        let high = self.fetch_byte(mmu);
        merge_bytes(low, high)
    }

    /// Executes an opcode and returns the number of cycles consumed.
    fn execute_opcode(&mut self, opcode: u8, mmu: &mut Mmu, ports: &mut Ports) -> u8 {
        let cycles = self.get_opcode_cycles(opcode);

        match opcode {
            // # Misc/control instructions
            0x00 | 0x08 | 0x10 | 0x18 | 0x20 | 0x28 | 0x30 | 0x38 => {} // NOP
            0x76 => self.hlt(),                                         // HLT
            0xD3 => {
                // OUT D8
                let port = self.fetch_byte(mmu);
                ports.write(port, self.a);
            }
            0xDB => {
                // IN D8
                let port = self.fetch_byte(mmu);
                self.a = ports.read(port);
            }
            0xF3 => self.ime = false, // DI
            0xFB => self.ime = true,  // EI

            // # Jumps/calls
            0xC0 => {
                // RNZ
                if !self.flags.z {
                    self.ret(mmu);
                }
            }
            0xD0 => {
                // RNC
                if !self.flags.cy {
                    self.ret(mmu);
                }
            }
            0xE0 => {
                // RPO
                if !self.flags.p {
                    self.ret(mmu);
                }
            }
            0xE8 => {
                // RPE
                if self.flags.p {
                    self.ret(mmu);
                }
            }
            0xF0 => {
                // RP
                if !self.flags.s {
                    self.ret(mmu);
                }
            }
            0xC8 => {
                // RZ
                if self.flags.z {
                    self.ret(mmu);
                }
            }
            0xD8 => {
                // RC
                if self.flags.cy {
                    self.ret(mmu);
                }
            }
            0xF8 => {
                // RM
                if self.flags.s {
                    self.ret(mmu);
                }
            }

            0xC2 => {
                // JNZ A16
                let addr = self.fetch_word(mmu);
                if !self.flags.z {
                    self.pc = addr;
                }
            }
            0xD2 => {
                // JNC A16
                let addr = self.fetch_word(mmu);
                if !self.flags.cy {
                    self.pc = addr;
                }
            }
            0xC3 | 0xCB => {
                // JMP A16
                let addr = self.fetch_word(mmu);
                self.pc = addr;
            }

            0xCA => {
                // JZ A16
                let addr = self.fetch_word(mmu);
                if self.flags.z {
                    self.pc = addr;
                }
            }
            0xDA => {
                // JC A16
                let addr = self.fetch_word(mmu);
                if self.flags.cy {
                    self.pc = addr;
                }
            }

            0xCC => {
                // CZ A16
                let addr = self.fetch_word(mmu);
                if self.flags.z {
                    self.push(self.pc, mmu);
                    self.pc = addr;
                }
            }
            0xDC => {
                // CC A16
                let addr = self.fetch_word(mmu);
                if self.flags.cy {
                    self.push(self.pc, mmu);
                    self.pc = addr;
                }
            }

            0xE2 => {
                // JPO A16
                let addr = self.fetch_word(mmu);
                if !self.flags.p {
                    self.pc = addr;
                }
            }
            0xEA => {
                // JPE A16
                let addr = self.fetch_word(mmu);
                if self.flags.p {
                    self.pc = addr;
                }
            }
            0xF2 => {
                // JP A16
                let addr = self.fetch_word(mmu);
                if !self.flags.s {
                    self.pc = addr;
                }
            }

            0xC4 => {
                // CNZ A16
                let addr = self.fetch_word(mmu);
                if !self.flags.z {
                    self.push(self.pc, mmu);
                    self.pc = addr;
                }
            }
            0xD4 => {
                // CNC A16
                let addr = self.fetch_word(mmu);
                if !self.flags.cy {
                    self.push(self.pc, mmu);
                    self.pc = addr;
                }
            }

            0xE4 => {
                // CPO A16
                let addr = self.fetch_word(mmu);
                if !self.flags.p {
                    self.push(self.pc, mmu);
                    self.pc = addr;
                }
            }
            0xEC => {
                // CPE A16
                let addr = self.fetch_word(mmu);
                if self.flags.p {
                    self.push(self.pc, mmu);
                    self.pc = addr;
                }
            }
            0xF4 => {
                // CP A16
                let addr = self.fetch_word(mmu);
                if !self.flags.s {
                    self.push(self.pc, mmu);
                    self.pc = addr;
                }
            }
            0xFA => {
                // JM A16
                let addr = self.fetch_word(mmu);
                if self.flags.s {
                    self.pc = addr;
                }
            }
            0xFC => {
                // CM A16
                let addr = self.fetch_word(mmu);
                if self.flags.s {
                    self.push(self.pc, mmu);
                    self.pc = addr;
                }
            }

            0xE9 => {
                // PCHL
                self.pc = self.get_hl();
            }

            0xC7 => self.rst(0, mmu), // RST 0
            0xCF => self.rst(1, mmu), // RST 1
            0xD7 => self.rst(2, mmu), // RST 2
            0xDF => self.rst(3, mmu), // RST 3
            0xE7 => self.rst(4, mmu), // RST 4
            0xEF => self.rst(5, mmu), // RST 5
            0xF7 => self.rst(6, mmu), // RST 6
            0xFF => self.rst(7, mmu), // RST 7

            0xCD | 0xDD | 0xED | 0xFD => {
                // CALL A16
                let addr = self.fetch_word(mmu);
                self.push(self.pc, mmu);
                self.pc = addr;
            }

            0xC9 | 0xD9 => self.ret(mmu), // RET

            // # 8bit load/store/move instructions

            // Store Accumulator
            0x02 => mmu.write_byte(self.get_bc(), self.a), // STAX BC
            0x12 => mmu.write_byte(self.get_de(), self.a), // STAX DE

            // Load Accumulator
            0x0A => self.a = mmu.read_byte(self.get_bc()), // LDAX BC
            0x1A => self.a = mmu.read_byte(self.get_de()), // LDAX DE

            // Store/Load Accumulator Direct
            0x32 => {
                // STA D16
                let addr = self.fetch_word(mmu);
                mmu.write_byte(addr, self.a);
            }
            0x3A => {
                // LDA D16
                let addr = self.fetch_word(mmu);
                self.a = mmu.read_byte(addr);
            }

            // Move Immediate to Register
            0x06 => self.b = self.fetch_byte(mmu), // MVI B D8
            0x0E => self.c = self.fetch_byte(mmu), // MVI C D8
            0x16 => self.d = self.fetch_byte(mmu), // MVI D D8
            0x1E => self.e = self.fetch_byte(mmu), // MVI E D8
            0x26 => self.h = self.fetch_byte(mmu), // MVI H D8
            0x2E => self.l = self.fetch_byte(mmu), // MVI L D8
            0x36 => {
                // MVI M D8
                let value = self.fetch_byte(mmu);
                self.set_at_hl(value, mmu);
            }
            0x3E => self.a = self.fetch_byte(mmu), // MVI A D8

            // Move Register to Register
            0x40 => self.b = self.b,                       // MOV B,B
            0x41 => self.b = self.c,                       // MOV B,C
            0x42 => self.b = self.d,                       // MOV B,D
            0x43 => self.b = self.e,                       // MOV B,E
            0x44 => self.b = self.h,                       // MOV B,H
            0x45 => self.b = self.l,                       // MOV B,L
            0x46 => self.b = mmu.read_byte(self.get_hl()), // MOV B,[HL]
            0x47 => self.b = self.a,                       // MOV B,A
            0x48 => self.c = self.b,                       // MOV C,B
            0x49 => self.c = self.c,                       // MOV C,C
            0x4A => self.c = self.d,                       // MOV C,D
            0x4B => self.c = self.e,                       // MOV C,E
            0x4C => self.c = self.h,                       // MOV C,H
            0x4D => self.c = self.l,                       // MOV C,L
            0x4E => self.c = mmu.read_byte(self.get_hl()), // MOV C,[HL]
            0x4F => self.c = self.a,                       // MOV C,A
            0x50 => self.d = self.b,                       // MOV D,B
            0x51 => self.d = self.c,                       // MOV D,C
            0x52 => self.d = self.d,                       // MOV D,D
            0x53 => self.d = self.e,                       // MOV D,E
            0x54 => self.d = self.h,                       // MOV D,H
            0x55 => self.d = self.l,                       // MOV D,L
            0x56 => self.d = mmu.read_byte(self.get_hl()), // MOV D,[HL]
            0x57 => self.d = self.a,                       // MOV D,A
            0x58 => self.e = self.b,                       // MOV E,B
            0x59 => self.e = self.c,                       // MOV E,C
            0x5A => self.e = self.d,                       // MOV E,D
            0x5B => self.e = self.e,                       // MOV E,E
            0x5C => self.e = self.h,                       // MOV E,H
            0x5D => self.e = self.l,                       // MOV E,L
            0x5E => self.e = mmu.read_byte(self.get_hl()), // MOV E,[HL]
            0x5F => self.e = self.a,                       // MOV E,A
            0x60 => self.h = self.b,                       // MOV H,B
            0x61 => self.h = self.c,                       // MOV H,C
            0x62 => self.h = self.d,                       // MOV H,D
            0x63 => self.h = self.e,                       // MOV H,E
            0x64 => self.h = self.h,                       // MOV H,H
            0x65 => self.h = self.l,                       // MOV H,L
            0x66 => self.h = mmu.read_byte(self.get_hl()), // MOV H,[HL]
            0x67 => self.h = self.a,                       // MOV H,A
            0x68 => self.l = self.b,                       // MOV L,B
            0x69 => self.l = self.c,                       // MOV L,C
            0x6A => self.l = self.d,                       // MOV L,D
            0x6B => self.l = self.e,                       // MOV L,E
            0x6C => self.l = self.h,                       // MOV L,H
            0x6D => self.l = self.l,                       // MOV L,L
            0x6E => self.l = mmu.read_byte(self.get_hl()), // MOV L,[HL]
            0x6F => self.l = self.a,                       // MOV L,A
            0x70 => self.set_at_hl(self.b, mmu),           // MOV [HL],B
            0x71 => self.set_at_hl(self.c, mmu),           // MOV [HL],C
            0x72 => self.set_at_hl(self.d, mmu),           // MOV [HL],D
            0x73 => self.set_at_hl(self.e, mmu),           // MOV [HL],E
            0x74 => self.set_at_hl(self.h, mmu),           // MOV [HL],H
            0x75 => self.set_at_hl(self.l, mmu),           // MOV [HL],L
            0x77 => self.set_at_hl(self.a, mmu),           // MOV [HL],A
            0x78 => self.a = self.b,                       // MOV A,B
            0x79 => self.a = self.c,                       // MOV A,C
            0x7A => self.a = self.d,                       // MOV A,D
            0x7B => self.a = self.e,                       // MOV A,E
            0x7C => self.a = self.h,                       // MOV A,H
            0x7D => self.a = self.l,                       // MOV A,L
            0x7E => self.a = mmu.read_byte(self.get_hl()), // MOV A,[HL]
            0x7F => self.a = self.a,                       // MOV A,A

            // # 16-bit load/store/move instructions

            // Load Immediate to Register Pair
            0x01 => {
                // LXI BC,D16
                let word = self.fetch_word(mmu);
                self.set_bc(word);
            }
            0x11 => {
                // LXI DE,D16
                let word = self.fetch_word(mmu);
                self.set_de(word);
            }
            0x21 => {
                // LXI HL,D16
                let word = self.fetch_word(mmu);
                self.set_hl(word);
            }
            0x31 => {
                // LXI SP,D16
                let word = self.fetch_word(mmu);
                self.sp = word;
            }

            // Store/Load HL Direct
            0x22 => {
                // SHLD D16
                let addr = self.fetch_word(mmu);
                let value = self.get_hl();
                mmu.write_word(addr, value);
            }
            0x2A => {
                // LHLD D16
                let addr = self.fetch_word(mmu);
                let value = mmu.read_word(addr);
                self.set_hl(value);
            }

            // # Stack operations
            0xC1 => {
                // POP BC
                let value = self.pop(mmu);
                self.set_bc(value);
            }
            0xD1 => {
                // POP DE
                let value = self.pop(mmu);
                self.set_de(value);
            }
            0xE1 => {
                // POP HL
                let value = self.pop(mmu);
                self.set_hl(value);
            }
            0xF1 => {
                // POP PSW
                let value = self.pop(mmu);
                let [low, high] = value.to_le_bytes();
                self.a = high;
                self.flags.set_from_byte(low);
            }
            0xC5 => {
                // PUSH BC
                let value = self.get_bc();
                self.push(value, mmu);
            }
            0xD5 => {
                // PUSH DE
                let value = self.get_de();
                self.push(value, mmu);
            }
            0xE5 => {
                // PUSH HL
                let value = self.get_hl();
                self.push(value, mmu);
            }
            0xF5 => {
                // PUSH PSW
                let low = self.flags.to_byte();
                let high = self.a;
                let value = merge_bytes(low, high);
                self.push(value, mmu);
            }

            0xE3 => {
                // XTHL
                let sp = self.sp;
                let low = mmu.read_byte(sp);
                let high = mmu.read_byte(sp.wrapping_add(1));
                let temp_l = self.l;
                let temp_h = self.h;
                self.l = low;
                self.h = high;
                mmu.write_byte(sp, temp_l);
                mmu.write_byte(sp.wrapping_add(1), temp_h);
            }
            0xEB => {
                // XCHG
                let temp_d = self.d;
                let temp_e = self.e;
                self.d = self.h;
                self.e = self.l;
                self.h = temp_d;
                self.l = temp_e;
            }
            0xF9 => {
                // SPHL
                self.sp = self.get_hl();
            }

            // # 8bit arithmetic/logical instructions
            0x07 => self.a = self.rol(self.a), // RLC
            0x0F => self.a = self.ror(self.a), // RRC
            0x17 => self.a = self.ral(),       // RAL
            0x1F => self.a = self.rar(),       // RAR
            0x27 => self.a = self.daa(),       // DAA
            0x2F => self.a = self.cma(),       // CMA
            0x37 => self.stc(),                // STC
            0x3F => self.cmc(),                // CMC

            0x04 => self.b = self.inr(self.b), // INR B
            0x0C => self.c = self.inr(self.c), // INR C
            0x14 => self.d = self.inr(self.d), // INR D
            0x1C => self.e = self.inr(self.e), // INR E
            0x24 => self.h = self.inr(self.h), // INR H
            0x2C => self.l = self.inr(self.l), // INR L
            0x3C => self.a = self.inr(self.a), // INR A

            0x05 => self.b = self.dcr(self.b), // DCR B
            0x0D => self.c = self.dcr(self.c), // DCR C
            0x15 => self.d = self.dcr(self.d), // DCR D
            0x1D => self.e = self.dcr(self.e), // DCR E
            0x25 => self.h = self.dcr(self.h), // DCR H
            0x2D => self.l = self.dcr(self.l), // DCR L
            0x34 => {
                // INR [HL]
                let addr = self.get_hl();
                let value = self.inr(mmu.read_byte(addr));
                mmu.write_byte(addr, value);
            }
            0x35 => {
                // DCR [HL]
                let addr = self.get_hl();
                let value = self.dcr(mmu.read_byte(addr));
                mmu.write_byte(addr, value);
            }
            0x3D => self.a = self.dcr(self.a), // DCR A

            0x80 => self.add(self.b),                       // ADD B
            0x81 => self.add(self.c),                       // ADD C
            0x82 => self.add(self.d),                       // ADD D
            0x83 => self.add(self.e),                       // ADD E
            0x84 => self.add(self.h),                       // ADD H
            0x85 => self.add(self.l),                       // ADD L
            0x86 => self.add(mmu.read_byte(self.get_hl())), // ADD [HL]
            0x87 => self.add(self.a),                       // ADD A

            0x88 => self.adc(self.b),                       // ADC B
            0x89 => self.adc(self.c),                       // ADC C
            0x8A => self.adc(self.d),                       // ADC D
            0x8B => self.adc(self.e),                       // ADC E
            0x8C => self.adc(self.h),                       // ADC H
            0x8D => self.adc(self.l),                       // ADC L
            0x8E => self.adc(mmu.read_byte(self.get_hl())), // ADC [HL]
            0x8F => self.adc(self.a),                       // ADC A

            0x90 => self.sub(self.b),                       // SUB B
            0x91 => self.sub(self.c),                       // SUB C
            0x92 => self.sub(self.d),                       // SUB D
            0x93 => self.sub(self.e),                       // SUB E
            0x94 => self.sub(self.h),                       // SUB H
            0x95 => self.sub(self.l),                       // SUB L
            0x96 => self.sub(mmu.read_byte(self.get_hl())), // SUB [HL]
            0x97 => self.sub(self.a),                       // SUB A

            0x98 => self.sbb(self.b),                       // SBB B
            0x99 => self.sbb(self.c),                       // SBB C
            0x9A => self.sbb(self.d),                       // SBB D
            0x9B => self.sbb(self.e),                       // SBB E
            0x9C => self.sbb(self.h),                       // SBB H
            0x9D => self.sbb(self.l),                       // SBB L
            0x9E => self.sbb(mmu.read_byte(self.get_hl())), // SBB [HL]
            0x9F => self.sbb(self.a),                       // SBB A

            0xA0 => self.ana(self.b),                       // ANA B
            0xA1 => self.ana(self.c),                       // ANA C
            0xA2 => self.ana(self.d),                       // ANA D
            0xA3 => self.ana(self.e),                       // ANA E
            0xA4 => self.ana(self.h),                       // ANA H
            0xA5 => self.ana(self.l),                       // ANA L
            0xA6 => self.ana(mmu.read_byte(self.get_hl())), // ANA [HL]
            0xA7 => self.ana(self.a),                       // ANA A

            0xA8 => self.xra(self.b),                       // XRA B
            0xA9 => self.xra(self.c),                       // XRA C
            0xAA => self.xra(self.d),                       // XRA D
            0xAB => self.xra(self.e),                       // XRA E
            0xAC => self.xra(self.h),                       // XRA H
            0xAD => self.xra(self.l),                       // XRA L
            0xAE => self.xra(mmu.read_byte(self.get_hl())), // XRA [HL]
            0xAF => self.xra(self.a),                       // XRA A

            0xB0 => self.ora(self.b),                       // ORA B
            0xB1 => self.ora(self.c),                       // ORA C
            0xB2 => self.ora(self.d),                       // ORA D
            0xB3 => self.ora(self.e),                       // ORA E
            0xB4 => self.ora(self.h),                       // ORA H
            0xB5 => self.ora(self.l),                       // ORA L
            0xB6 => self.ora(mmu.read_byte(self.get_hl())), // ORA [HL]
            0xB7 => self.ora(self.a),                       // ORA A

            0xB8 => self.cmp(self.b),                       // CMP B
            0xB9 => self.cmp(self.c),                       // CMP C
            0xBA => self.cmp(self.d),                       // CMP D
            0xBB => self.cmp(self.e),                       // CMP E
            0xBC => self.cmp(self.h),                       // CMP H
            0xBD => self.cmp(self.l),                       // CMP L
            0xBE => self.cmp(mmu.read_byte(self.get_hl())), // CMP [HL]
            0xBF => self.cmp(self.a),                       // CMP A

            0xC6 => {
                // ADI D8
                let value = self.fetch_byte(mmu);
                self.add(value);
            }
            0xCE => {
                // ACI D8
                let value = self.fetch_byte(mmu);
                self.adc(value);
            }
            0xD6 => {
                // SUI D8
                let value = self.fetch_byte(mmu);
                self.sub(value);
            }
            0xDE => {
                // SBI D8
                let value = self.fetch_byte(mmu);
                self.sbb(value);
            }
            0xE6 => {
                // ANI D8
                let value = self.fetch_byte(mmu);
                self.ana(value);
            }
            0xEE => {
                // XRI D8
                let value = self.fetch_byte(mmu);
                self.xra(value);
            }
            0xF6 => {
                // ORI D8
                let value = self.fetch_byte(mmu);
                self.ora(value);
            }
            0xFE => {
                // CPI D8
                let value = self.fetch_byte(mmu);
                self.cmp(value);
            }

            // # 16-bit arithmetic/logical instructions
            0x03 => {
                // INX BC
                let bc = self.get_bc().wrapping_add(1);
                self.set_bc(bc);
            }
            0x13 => {
                // INX DE
                let de = self.get_de().wrapping_add(1);
                self.set_de(de);
            }
            0x23 => {
                // INX HL
                let hl = self.get_hl().wrapping_add(1);
                self.set_hl(hl);
            }
            0x33 => {
                // INX SP
                self.sp = self.sp.wrapping_add(1);
            }

            0x0B => {
                // DCX BC
                let bc = self.get_bc().wrapping_sub(1);
                self.set_bc(bc);
            }
            0x1B => {
                // DCX DE
                let de = self.get_de().wrapping_sub(1);
                self.set_de(de);
            }
            0x2B => {
                // DCX HL
                let hl = self.get_hl().wrapping_sub(1);
                self.set_hl(hl);
            }
            0x3B => {
                // DCX SP
                self.sp = self.sp.wrapping_sub(1);
            }

            0x09 => {
                // DAD BC
                let bc = self.get_bc();
                self.dad(bc);
            }
            0x19 => {
                // DAD DE
                let de = self.get_de();
                self.dad(de);
            }
            0x29 => {
                // DAD HL
                let hl = self.get_hl();
                self.dad(hl);
            }
            0x39 => {
                // DAD SP
                let sp = self.sp;
                self.dad(sp);
            }
        };

        cycles
    }

    /// Triggers an interrupt if interrupts are enabled.
    pub fn interrupt(&mut self, vector: u8, mmu: &mut Mmu) {
        if !self.ime {
            return;
        }

        self.ime = false;
        self.halted = false;
        self.rst(vector, mmu);
    }

    fn get_opcode_cycles(&self, opcode: u8) -> u8 {
        OPCODE_CYCLES[opcode as usize]
    }

    /// Halt the CPU.
    fn hlt(&mut self) {
        self.halted = true;
    }

    /// Calls a subroutine at the specified address.
    fn rst(&mut self, n: u8, mmu: &mut Mmu) {
        let addr = (n as u16) * 8;
        self.push(self.pc, mmu);
        self.pc = addr;
    }

    /// Adds a value to the HL register pair and updates the carry flag.
    fn dad(&mut self, value: u16) {
        let (result, carry) = self.get_hl().overflowing_add(value);
        self.set_hl(result);
        self.flags.cy = carry;
    }

    /// Rotate Left through Carry.
    fn rol(&mut self, value: u8) -> u8 {
        let carry = (value & 0x80) != 0;
        let result = (value << 1) | if carry { 1 } else { 0 };
        self.flags.cy = carry;
        result
    }

    /// Rotate Right through Carry.
    fn ror(&mut self, value: u8) -> u8 {
        let carry = (value & 0x01) != 0;
        let result = (value >> 1) | if carry { 0x80 } else { 0 };
        self.flags.cy = carry;
        result
    }

    /// Rotate Accumulator Left through Carry.
    fn ral(&mut self) -> u8 {
        let carry = if self.flags.cy { 1 } else { 0 };
        let new_carry = (self.a & 0x80) != 0;
        let result = (self.a << 1) | carry;
        self.flags.cy = new_carry;
        result
    }

    /// Rotate Accumulator Right through Carry.
    fn rar(&mut self) -> u8 {
        let carry = if self.flags.cy { 0x80 } else { 0 };
        let new_carry = (self.a & 0x01) != 0;
        let result = (self.a >> 1) | carry;
        self.flags.cy = new_carry;
        result
    }

    fn daa(&mut self) -> u8 {
        let mut correction = 0;
        if (self.a & 0x0F) > 9 || self.flags.ac {
            correction += 0x06;
            self.flags.ac = true;
        } else {
            self.flags.ac = false;
        }
        if (self.a >> 4) > 9 || self.flags.cy || ((self.a + correction) > 0x99) {
            correction += 0x60;
            self.flags.cy = true;
        } else {
            self.flags.cy = false;
        }
        self.a = self.a.wrapping_add(correction);
        self.flags.z = self.a == 0;
        self.flags.s = (self.a & 0x80) != 0;
        self.flags.p = self.a.count_ones() % 2 == 0;
        self.a
    }

    /// Complement Accumulator.
    fn cma(&mut self) -> u8 {
        !self.a
    }

    /// Set Carry Flag.
    fn stc(&mut self) {
        self.flags.cy = true;
    }

    /// Complement Carry Flag.
    fn cmc(&mut self) {
        self.flags.cy = !self.flags.cy;
    }

    /// Increments a value and updates flags.
    fn inr(&mut self, value: u8) -> u8 {
        let result = value.wrapping_add(1);
        self.flags.z = result == 0;
        self.flags.s = (result & 0x80) != 0;
        self.flags.p = result.count_ones() % 2 == 0;
        self.flags.ac = (value & 0x0F) + 1 > 0x0F;
        result
    }

    /// Decrements a value and updates flags.
    fn dcr(&mut self, value: u8) -> u8 {
        let result = value.wrapping_sub(1);
        self.flags.z = result == 0;
        self.flags.s = (result & 0x80) != 0;
        self.flags.p = result.count_ones() % 2 == 0;
        self.flags.ac = (value & 0x0F) == 0;
        result
    }

    /// Adds a value to the accumulator and updates flags.
    fn add(&mut self, value: u8) {
        let (result, carry) = self.a.overflowing_add(value);
        self.a = result;
        self.flags.cy = carry;
        self.flags.z = self.a == 0;
        self.flags.s = (self.a & 0x80) != 0;
        self.flags.p = self.a.count_ones() % 2 == 0;
        self.flags.ac = (self.a & 0x10) != 0;
    }

    /// Adds a value and the carry flag to the accumulator and updates flags.
    fn adc(&mut self, value: u8) {
        let carry = if self.flags.cy { 1 } else { 0 };
        let (intermediate, carry1) = self.a.overflowing_add(value);
        let (result, carry2) = intermediate.overflowing_add(carry);
        self.a = result;
        self.flags.cy = carry1 || carry2;
        self.flags.z = self.a == 0;
        self.flags.s = (self.a & 0x80) != 0;
        self.flags.p = self.a.count_ones() % 2 == 0;
        self.flags.ac = ((self.a & 0x0F) + (value & 0x0F) + carry) > 0x0F;
    }

    /// Subtracts a value from the accumulator and updates flags.
    fn sub(&mut self, value: u8) {
        let (result, borrow) = self.a.overflowing_sub(value);
        self.a = result;
        self.flags.cy = borrow;
        self.flags.z = self.a == 0;
        self.flags.s = (self.a & 0x80) != 0;
        self.flags.p = self.a.count_ones() % 2 == 0;
        self.flags.ac = (self.a & 0x0F) < (value & 0x0F);
    }

    /// Subtracts a value and the carry flag from the accumulator and updates flags.
    fn sbb(&mut self, value: u8) {
        let carry = if self.flags.cy { 1 } else { 0 };
        let (intermediate, borrow1) = self.a.overflowing_sub(value);
        let (result, borrow2) = intermediate.overflowing_sub(carry);
        self.a = result;
        self.flags.cy = borrow1 || borrow2;
        self.flags.z = self.a == 0;
        self.flags.s = (self.a & 0x80) != 0;
        self.flags.p = self.a.count_ones() % 2 == 0;
        self.flags.ac = (self.a & 0x0F) < (value & 0x0F);
    }

    /// Logical AND between accumulator and value, updates flags.
    fn ana(&mut self, value: u8) {
        self.a &= value;
        self.flags.cy = false;
        self.flags.z = self.a == 0;
        self.flags.s = (self.a & 0x80) != 0;
        self.flags.p = self.a.count_ones() % 2 == 0;
        self.flags.ac = true;
    }

    /// Logical XOR between accumulator and value, updates flags.
    fn xra(&mut self, value: u8) {
        self.a ^= value;
        self.flags.cy = false;
        self.flags.z = self.a == 0;
        self.flags.s = (self.a & 0x80) != 0;
        self.flags.p = self.a.count_ones() % 2 == 0;
        self.flags.ac = false;
    }

    /// Logical OR between accumulator and value, updates flags.
    fn ora(&mut self, value: u8) {
        self.a |= value;
        self.flags.cy = false;
        self.flags.z = self.a == 0;
        self.flags.s = (self.a & 0x80) != 0;
        self.flags.p = self.a.count_ones() % 2 == 0;
        self.flags.ac = false;
    }

    /// Compares a value with the accumulator and updates flags.
    fn cmp(&mut self, value: u8) {
        let (result, borrow) = self.a.overflowing_sub(value);
        self.flags.cy = borrow;
        self.flags.z = result == 0;
        self.flags.s = (result & 0x80) != 0;
        self.flags.p = result.count_ones() % 2 == 0;
        self.flags.ac = (self.a & 0x0F) < (value & 0x0F);
    }

    /// Returns from a subroutine.
    fn ret(&mut self, mmu: &Mmu) {
        self.pc = self.pop(mmu)
    }

    /// Pops a 16-bit value from the stack.
    fn pop(&mut self, mmu: &Mmu) -> u16 {
        let word = mmu.read_word(self.sp);
        self.sp = self.sp.wrapping_add(2);
        word
    }

    /// Pushes a 16-bit value onto the stack.
    fn push(&mut self, value: u16, mmu: &mut Mmu) {
        self.sp = self.sp.wrapping_sub(2);
        mmu.write_word(self.sp, value);
    }

    /// Returns the AF register pair (A as high byte, flags as low byte).
    pub fn get_af(&self) -> u16 {
        let a = self.a as u16;
        let f = self.flags.to_byte() as u16;
        (a << 8) | f
    }

    /// Returns the combined value of registers B and C as a 16-bit value.
    fn get_bc(&self) -> u16 {
        merge_bytes(self.c, self.b)
    }

    /// Sets the combined value of registers B and C from a 16-bit value.
    fn set_bc(&mut self, word: u16) {
        let [low, high] = word.to_le_bytes();
        self.b = high;
        self.c = low;
    }

    /// Returns the combined value of registers D and E as a 16-bit value.
    pub fn get_de(&self) -> u16 {
        merge_bytes(self.e, self.d)
    }

    /// Sets the combined value of registers D and E from a 16-bit value.
    fn set_de(&mut self, word: u16) {
        let [low, high] = word.to_le_bytes();
        self.d = high;
        self.e = low;
    }

    /// Returns the combined value of registers H and L as a 16-bit value.
    fn get_hl(&self) -> u16 {
        merge_bytes(self.l, self.h)
    }

    /// Sets the combined value of registers H and L from a 16-bit value.
    fn set_hl(&mut self, word: u16) {
        let [low, high] = word.to_le_bytes();
        self.h = high;
        self.l = low;
    }

    /// Writes a byte to the memory address pointed to by the HL register pair.
    fn set_at_hl(&mut self, value: u8, mmu: &mut Mmu) {
        let addr = self.get_hl();
        mmu.write_byte(addr, value);
    }
}

/// Auxiliary function that takes 2 bytes and returns a word
fn merge_bytes(low: u8, high: u8) -> u16 {
    ((high as u16) << 8) | (low as u16)
}
