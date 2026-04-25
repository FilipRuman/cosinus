use crate::emulator::thread::Thread;

impl Thread {
    pub fn addr(&mut self, rd: u8, rs1: u8, rs2: u8) {
        self.gpr[rd as usize] = self.gpr[rs1 as usize].wrapping_add(self.gpr[rs2 as usize]);
    }

    pub fn subr(&mut self, rd: u8, rs1: u8, rs2: u8) {
        self.gpr[rd as usize] = self.gpr[rs1 as usize].wrapping_sub(self.gpr[rs2 as usize]);
    }

    pub fn andr(&mut self, rd: u8, rs1: u8, rs2: u8) {
        self.gpr[rd as usize] = self.gpr[rs1 as usize] & self.gpr[rs2 as usize];
    }

    pub fn orr(&mut self, rd: u8, rs1: u8, rs2: u8) {
        self.gpr[rd as usize] = self.gpr[rs1 as usize] | self.gpr[rs2 as usize];
    }

    pub fn xorr(&mut self, rd: u8, rs1: u8, rs2: u8) {
        self.gpr[rd as usize] = self.gpr[rs1 as usize] ^ self.gpr[rs2 as usize];
    }

    pub fn mulr(&mut self, rd: u8, rs1: u8, rs2: u8) {
        self.gpr[rd as usize] = self.gpr[rs1 as usize].wrapping_mul(self.gpr[rs2 as usize]);
    }

    pub fn divr(&mut self, rd: u8, rs1: u8, rs2: u8) {
        let b = self.gpr[rs2 as usize];
        self.gpr[rd as usize] = if b == 0 {
            0
        } else {
            self.gpr[rs1 as usize] / b
        };
    }

    pub fn remr(&mut self, rd: u8, rs1: u8, rs2: u8) {
        let b = self.gpr[rs2 as usize];
        self.gpr[rd as usize] = if b == 0 {
            0
        } else {
            self.gpr[rs1 as usize] % b
        };
    }

    pub fn shlr(&mut self, rd: u8, rs1: u8, rs2: u8) {
        let sh = self.gpr[rs2 as usize] as u32 & 0x1F;
        self.gpr[rd as usize] = self.gpr[rs1 as usize].wrapping_shl(sh);
    }

    pub fn shrr(&mut self, rd: u8, rs1: u8, rs2: u8) {
        let sh = self.gpr[rs2 as usize] as u32 & 0x1F;
        self.gpr[rd as usize] = self.gpr[rs1 as usize].wrapping_shr(sh);
    }

    pub fn sarr(&mut self, rd: u8, rs1: u8, rs2: u8) {
        let sh = self.gpr[rs2 as usize] as u32 & 0x1F;
        self.gpr[rd as usize] = self.gpr[rs1 as usize] >> sh;
    }

    pub fn add(&mut self, rd: u8, rs1: u8, imm: i16) {
        let imm = imm as i32;
        self.gpr[rd as usize] = self.gpr[rs1 as usize].wrapping_add(imm);
    }

    pub fn sub(&mut self, rd: u8, rs1: u8, imm: i16) {
        let imm = imm as i32;
        self.gpr[rd as usize] = self.gpr[rs1 as usize].wrapping_sub(imm);
    }

    pub fn and(&mut self, rd: u8, rs1: u8, imm: i16) {
        let imm = imm as i32;
        self.gpr[rd as usize] = self.gpr[rs1 as usize] & imm;
    }

    pub fn or(&mut self, rd: u8, rs1: u8, imm: i16) {
        let imm = imm as i32;
        self.gpr[rd as usize] = self.gpr[rs1 as usize] | imm;
    }

    pub fn xor(&mut self, rd: u8, rs1: u8, imm: i16) {
        let imm = imm as i32;
        self.gpr[rd as usize] = self.gpr[rs1 as usize] ^ imm;
    }

    pub fn mul(&mut self, rd: u8, rs1: u8, imm: i16) {
        let imm = imm as i32;
        self.gpr[rd as usize] = self.gpr[rs1 as usize].wrapping_mul(imm);
    }

    pub fn div(&mut self, rd: u8, rs1: u8, imm: i16) {
        let imm = imm as i32;
        self.gpr[rd as usize] = if imm == 0 {
            0
        } else {
            self.gpr[rs1 as usize] / imm
        };
    }

    pub fn rem(&mut self, rd: u8, rs1: u8, imm: i16) {
        let imm = imm as i32;
        self.gpr[rd as usize] = if imm == 0 {
            0
        } else {
            self.gpr[rs1 as usize] % imm
        };
    }

    pub fn shl(&mut self, rd: u8, rs1: u8, imm: i16) {
        let sh = imm as u32 & 0x1F;
        self.gpr[rd as usize] = self.gpr[rs1 as usize].wrapping_shl(sh);
    }

    pub fn shr(&mut self, rd: u8, rs1: u8, imm: i16) {
        let sh = imm as u32 & 0x1F;
        self.gpr[rd as usize] = self.gpr[rs1 as usize].wrapping_shr(sh);
    }

    pub fn sar(&mut self, rd: u8, rs1: u8, imm: i16) {
        let sh = imm as u32 & 0x1F;
        self.gpr[rd as usize] = self.gpr[rs1 as usize] >> sh;
    }
    pub fn lui(&mut self, rd: u8, imm: i16) {
        let imm = (imm as i32) << 16i32;
        self.gpr[rd as usize] = imm;
    }
    pub fn not(&mut self, rd: u8, rs1: u8) {
        self.gpr[rd as usize] = !self.gpr[rs1 as usize];
    }
}
