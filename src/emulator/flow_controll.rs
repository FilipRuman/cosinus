use log::{debug, info};

use crate::emulator::core::Core;

impl Core {
    pub fn jmp(&mut self, imm26: i32) {
        let sign_extended = ((imm26 << 6) as i32) >> 6;
        self.pc = self.pc.wrapping_add(sign_extended);
    }
    pub fn call(&mut self, imm26: i32) {
        let sign_extended = ((imm26 << 6) as i32) >> 6;
        self.gpr[Core::RA] = self.pc; // byte space address -> 4 bytes
        info!(
            "call, pc:'{}' imm:'{sign_extended:#b}', ra:'{}'",
            self.pc,
            self.gpr[Core::RA]
        );
        self.pc = self.pc.wrapping_add(sign_extended);
    }

    pub fn ret(&mut self) {
        info!("ret- retrun addr:'{}'", self.gpr[Core::RA]);
        self.pc = self.gpr[Core::RA];
    }

    pub fn jmpr(&mut self, rs: u8, imm: i16) {
        debug!("jmpr rs:{rs} rs_val:{} imm:{imm}", self.gpr[rs as usize]);
        let target = self.gpr[rs as usize].wrapping_add(imm as i32);
        self.pc = target;
    }

    pub fn apc(&mut self, rd: u8, imm: i16) {
        self.gpr[rd as usize] = self.pc.wrapping_add(imm as i32);
        debug!("apc rd:{rd} rd_val:{}", self.gpr[rd as usize]);
    }
}
