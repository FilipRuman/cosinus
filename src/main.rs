use anyhow::{Context, Result};

use crate::assembler::{
    instruction::{Immediate, Instruction, Macro},
    instruction_parsing,
};

mod assembler;
pub mod emulator;
pub mod log;
mod tests;

#[tokio::main]
async fn main() -> Result<()> {
    colog::init();
    unsafe {
        let assembled = assembler::assemble_commands(vec![
            Instruction::ADD {
                rd: 1,
                rs1: 0,
                imm: Immediate::Direct('H' as u16),
            }
            .into(),
            Macro::Set32 {
                rd: 2,
                imm: Immediate::Direct(0xE0300000_u32),
            }
            .into(),
            Instruction::STORE {
                rs1: 2,
                rs2: 1,
                imm: Immediate::Direct(0),
            }
            .into(),
        ])
        .context("assembling the test print instruction")?;
        emulator::write_instructions_to_memory(0, assembled);
    }
    emulator::run().await;

    let instr = instruction_parsing::parse_program(
        "
jmp jump_label
or r5 r5 25
set32 r25 0x2222333
beq r10 r5 253
scall
sel 30 20 10 3
:jump_label
halt
",
    )?;

    loop {}
}
