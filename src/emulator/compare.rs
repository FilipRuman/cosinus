use log::debug;

use crate::emulator::thread::Thread;
impl Thread {
    /// rd = (rs1 < rs2) signed
    pub fn ltr(&mut self, rd: u8, rs1: u8, rs2: u8) {
        self.gpr[rd as usize] = (self.gpr[rs1 as usize] < self.gpr[rs2 as usize]) as i32;
    }

    /// rd = (rs1 == rs2)
    pub fn eqr(&mut self, rd: u8, rs1: u8, rs2: u8) {
        self.gpr[rd as usize] = (self.gpr[rs1 as usize] == self.gpr[rs2 as usize]) as i32;
    }

    pub fn lt(&mut self, rd: u8, rs1: u8, imm: i16) {
        let b = imm as i32;
        self.gpr[rd as usize] = (self.gpr[rs1 as usize] < b) as i32;
    }

    pub fn eq(&mut self, rd: u8, rs1: u8, imm: i16) {
        let b = imm as i32;
        self.gpr[rd as usize] = (self.gpr[rs1 as usize] == b) as i32;
    }

    /// rd = rs3 ? rs1 : rs2   (TRUE if rs3 != 0)
    pub fn sel(&mut self, rd: u8, rs1: u8, rs2: u8, rs3: u8) {
        self.gpr[rd as usize] = if self.gpr[rs3 as usize] != 0 {
            self.gpr[rs1 as usize]
        } else {
            self.gpr[rs2 as usize]
        };
    }

    /// rd = trailing zeros of rs1
    pub fn ctz(&mut self, rd: u8, rs1: u8) {
        let val = self.gpr[rs1 as usize] as u32;
        self.gpr[rd as usize] = val.trailing_zeros() as i32;
    }

    /// rd = leading zeros of rs1
    pub fn clz(&mut self, rd: u8, rs1: u8) {
        let val = self.gpr[rs1 as usize] as u32;
        self.gpr[rd as usize] = val.leading_zeros() as i32;
    }
}
