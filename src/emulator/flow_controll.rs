use log::debug;

use crate::emulator::thread::Thread;

impl Thread {
    pub fn jmp(&mut self, imm: i32) {
        self.pc += imm;
    }
    pub fn call(&mut self, imm: i32) {
        debug!("call {imm}");
        self.gpr[Thread::RA] = self.pc; // byte space address -> 4 bytes
        self.pc = self.pc.wrapping_add(imm);
    }

    pub fn ret(&mut self) {
        self.pc = self.gpr[Thread::RA];
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
