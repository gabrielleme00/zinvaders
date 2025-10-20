use crate::mmu::Mmu;

/// CP/M BDOS (Basic Disk Operating System) emulation for TST8080.COM
pub struct Bdos;

impl Bdos {
    /// Handles CP/M BDOS call emulation
    /// This is called when the CPU executes a CALL to address 0x0005
    pub fn handle_call(c_reg: u8, de_reg: u16, mmu: &Mmu) {
        match c_reg {
            2 => {
                // Function 2: Console output (print character in E)
                let ch = (de_reg & 0xFF) as u8;
                print!("{}", ch as char);
            }
            9 => {
                // Function 9: Print string (address in DE, terminated by '$')
                let mut addr = de_reg;
                loop {
                    let ch = mmu.read_byte(addr);
                    if ch == b'$' {
                        break;
                    }
                    print!("{}", ch as char);
                    addr = addr.wrapping_add(1);
                }
            }
            _ => {
                // Other functions not implemented for TST8080.COM
            }
        }
    }
}
