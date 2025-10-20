use crate::cpu::Cpu;
use crate::mmu::Mmu;
use crate::ports::Ports;

pub struct State {
    pub cpu: Cpu,
    pub mmu: Mmu,
    pub ports: Ports,
}

impl State {
    pub fn new() -> Self {
        Self {
            cpu: Cpu::new(),
            mmu: Mmu::new(),
            ports: Ports::new(),
        }
    }
}