use crate::emulator::thread::Thread;

impl Thread {
    pub fn beq(&mut self, rs1: u8, rs2: u8, imm: i16) {
        if self.reg(rs1) == self.reg(rs2) {
            self.jmp(imm as i32);
        }
    }

    pub fn bne(&mut self, rs1: u8, rs2: u8, imm: i16) {
        if self.reg(rs1) != self.reg(rs2) {
            self.jmp(imm as i32);
        }
    }

    pub fn blt(&mut self, rs1: u8, rs2: u8, imm: i16) {
        if (self.reg(rs1) as i32) < (self.reg(rs2) as i32) {
            self.jmp(imm as i32);
        }
    }

    pub fn bgt(&mut self, rs1: u8, rs2: u8, imm: i16) {
        if (self.reg(rs1) as i32) > (self.reg(rs2) as i32) {
            self.jmp(imm as i32);
        }
    }

    pub fn ble(&mut self, rs1: u8, rs2: u8, imm: i16) {
        if (self.reg(rs1) as i32) <= (self.reg(rs2) as i32) {
            self.jmp(imm as i32);
        }
    }

    pub fn bge(&mut self, rs1: u8, rs2: u8, imm: i16) {
        if (self.reg(rs1) as i32) >= (self.reg(rs2) as i32) {
            self.jmp(imm as i32);
        }
    }
}
