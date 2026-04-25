use log::{debug, info};

use crate::emulator::{interrupts::ExceptionType, memory::MEMORY, thread::Thread};

const OPTCODE_MASK: i32 = ((1u32 << 6) - 1) as i32; // 6 bits
const R_MASK: u8 = ((1u32 << 5) - 1) as u8; // 5 bits
const IMM16_MASK: i16 = ((1u32 << 16) - 1) as i16; // 16 bits
const IMM26_MASK: i32 = ((1u32 << 26) - 1) as i32; // 26 bits
impl Thread {
    pub fn run_current_instruction(&mut self) {
        let addr = self.pc as usize;
        let instruction = unsafe { MEMORY.read(addr) };
        debug!(
            "run_current_instruction: {:#x} {:#b}",
            instruction as u32, instruction as u32
        );
        self.handle_instruction(instruction);
        self.pc += 4;
    }

    pub fn test_parse_instruction(instruction: i32) {
        let optcode = ((instruction >> 26) & OPTCODE_MASK) as u8;
        let r0 = ((instruction >> 21) as u8) & R_MASK;
        let r1 = ((instruction >> 16) as u8) & R_MASK;
        let r2 = ((instruction >> 11) as u8) & R_MASK;
        let r3 = ((instruction >> 6) as u8) & R_MASK;
        let r2_imm16 = (instruction as i16) & IMM16_MASK;
        let r1_imm16 = ((instruction >> 5) as i16) & IMM16_MASK;
        let imm26 = (instruction as i32) & IMM26_MASK;
        debug!(
            "handle_instruction optcode:{:#x};{:032b} r0:{:#x} r1:{:#x} r2:{:#x} r3:{:#x} r2_imm16:{:#x} r1_imm16:{:#x} imm26:{:#x} ",
            optcode as u32,
            optcode as u32,
            r0 as u32,
            r1 as u32,
            r2 as u32,
            r3 as u32,
            r2_imm16 as u32,
            r1_imm16 as u32,
            imm26 as u32
        );
    }
    pub fn handle_instruction(&mut self, instruction: i32) {
        let optcode = ((instruction >> 26) & OPTCODE_MASK) as u8;
        let r0 = ((instruction >> 21) as u8) & R_MASK;
        let r1 = ((instruction >> 16) as u8) & R_MASK;
        let r2 = ((instruction >> 11) as u8) & R_MASK;
        let r3 = ((instruction >> 6) as u8) & R_MASK;
        let r2_imm16 = (instruction as i16) & IMM16_MASK;
        let r1_imm16 = ((instruction >> 5) as i16) & IMM16_MASK;
        let imm26 = (instruction as i32) & IMM26_MASK;
        debug!(
            "handle_instruction optcode:{:#x};{:032b} r0:{:#x} r1:{:#x} r2:{:#x} r3:{:#x} r2_imm16:{:#x} r1_imm16:{:#x} imm26:{:#x} ",
            optcode as u32,
            optcode as u32,
            r0 as u32,
            r1 as u32,
            r2 as u32,
            r3 as u32,
            r2_imm16 as u32,
            r1_imm16 as u32,
            imm26 as u32
        );
        match optcode {
            // --- Register ---
            0x00 => self.addr(r0, r1, r2),
            0x01 => self.subr(r0, r1, r2),
            0x02 => self.andr(r0, r1, r2),
            0x03 => self.orr(r0, r1, r2),
            0x04 => self.xorr(r0, r1, r2),
            0x05 => self.mulr(r0, r1, r2),
            0x06 => self.divr(r0, r1, r2),
            0x07 => self.remr(r0, r1, r2),
            0x08 => self.shlr(r0, r1, r2),
            0x09 => self.shrr(r0, r1, r2),
            0x0A => self.sarr(r0, r1, r2),

            // --- Immediate ---
            0x0B => self.add(r0, r1, r2_imm16),
            0x0C => self.sub(r0, r1, r2_imm16),
            0x0D => self.and(r0, r1, r2_imm16),
            0x0E => self.or(r0, r1, r2_imm16),
            0x0F => self.xor(r0, r1, r2_imm16),
            0x10 => self.mul(r0, r1, r2_imm16),
            0x11 => self.div(r0, r1, r2_imm16),
            0x12 => self.rem(r0, r1, r2_imm16),
            0x13 => self.shl(r0, r1, r2_imm16),
            0x14 => self.shr(r0, r1, r2_imm16),
            0x15 => self.sar(r0, r1, r2_imm16),

            // --- LUI ---
            0x16 => self.lui(r0, r1_imm16),

            // --- Memory ---
            0x17 => self.load(r0, r1, r2_imm16),
            0x18 => self.store(r2, r1, r2_imm16),
            0x19 => self.loadb(r0, r1, r2_imm16),
            0x1A => self.storeb(r2, r1, r2_imm16),
            0x1B => self.loadh(r0, r1, r2_imm16),
            0x1C => self.storeh(r2, r1, r2_imm16),
            0x1D => self.loadpc(r0, r1_imm16),

            // --- Control Flow ---
            0x1E => self.jmp(imm26),
            0x1F => self.call(imm26),
            0x20 => self.ret(),
            0x21 => self.jmpr(r0, r1_imm16),
            0x22 => self.apc(r0, r1_imm16),

            // --- Branch ---
            0x23 => self.beq(r0, r1, r2_imm16),
            0x24 => self.bne(r0, r1, r2_imm16),
            0x25 => self.blt(r0, r1, r2_imm16),
            0x26 => self.bgt(r0, r1, r2_imm16),
            0x27 => self.ble(r0, r1, r2_imm16),
            0x28 => self.bge(r0, r1, r2_imm16),

            // --- Sys ---
            0x29 => self.scall(),
            0x2A => self.sret(),
            0x2B => self.sysr(r0, r1_imm16),
            0x2C => self.sysw(r1, r1_imm16),

            // --- Atomic ---
            0x2D => self.lr(r0, r1),
            0x2E => self.sc(r0, r1, r2),

            // --- Misc ---
            0x2F => {} // nop
            0x30 => self.halt(),

            // --- Compare / Utility ---
            0x31 => self.ltr(r0, r1, r2),
            0x32 => self.eqr(r0, r1, r2),
            0x33 => self.ltu(r0, r1, r2_imm16),
            0x34 => self.equ(r0, r1, r2_imm16),
            0x35 => self.lts(r0, r1, r2_imm16),
            0x36 => self.eqs(r0, r1, r2_imm16),
            0x37 => self.sel(r0, r1, r2, r3),
            0x38 => self.ctz(r0, r1),
            0x39 => self.clz(r0, r1),
            0x3A => self.not(r0, r1),

            _ => {
                self.trigger_exception(ExceptionType::UnknownInstructionOptcode);
            }
        };
    }
}
