#[cfg(test)]
mod test {
    use crate::{
        assembler::{
            self,
            instruction::{Immediate, Instruction},
        },
        emulator::thread::Thread,
    };
    use anyhow::Result;
    use log::info;

    #[test]
    fn test_instruction_conversion_for_asembler() -> Result<()> {
        let r = |x: &u8| (*x as u32) & 0x1F;
        let imm16 = |x: u16| (x as u16 as u32) & 0xFFFF;
        let imm26 = |x: u32| (x as u32) & 0x03FF_FFFF;

        {
            let manual_or_instr = (0x0E << 26) | (r(&5) << 21) | (r(&5) << 16) | imm16(25);
            let asm_or_instr = assembler::assemble_commands(vec![
                Instruction::OR {
                    rd: 5,
                    rs1: 5,
                    imm: Immediate::Direct(25),
                }
                .into(),
            ])?[0];
            assert_eq!(asm_or_instr, manual_or_instr as i32);

            info!(
                "OR r5, r5, 25: '{:032b}', opt: '{:032b}'",
                manual_or_instr,
                (0x0E << 26) as i32
            );
            Thread::test_parse_instruction(manual_or_instr as i32);
        }

        {
            let manual_halt_instr = (0x30 << 26) | (r(&0) << 21) | (r(&0) << 16) | imm16(0);
            let asm_halt_instr =
                assembler::assemble_commands(vec![Instruction::HALT {}.into()])?[0];
            info!("MANUAL:{manual_halt_instr:032b}, asm_halt_instr:{asm_halt_instr:032b}");
            assert_eq!(asm_halt_instr, manual_halt_instr as i32);

            Thread::test_parse_instruction(asm_halt_instr as i32);
        }
        Ok(())
    }
}
