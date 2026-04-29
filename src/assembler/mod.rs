use std::collections::HashMap;

use crate::assembler::{
    instruction::{Command, Instruction},
    instruction_parsing::parse_program,
};

pub mod instruction;
pub mod instruction_parsing;
pub mod parsing_test;
mod test;
use anyhow::{Context, Result};

pub fn assemble_from_string(input: &str) -> Result<Vec<i32>> {
    let pased = parse_program(input)?;
    assemble_commands(pased)
}

pub fn assemble_commands(to_assemble: Vec<Command>) -> Result<Vec<i32>> {
    let mut output = Vec::with_capacity(to_assemble.len());
    let mut labels = HashMap::new();

    let mut pc = 0;
    for value in &to_assemble {
        match value {
            Command::Instr(_) => {
                pc += 1;
            }

            Command::Macro(_macro) => {
                let instructions: Result<Vec<Instruction>> = _macro.into();
                pc += instructions?.len();
            }
            Command::Label(name) => {
                labels.insert(name, pc - 1);
            }
            Command::RawData(data) => pc += data.len(),
        }
    }

    pc = 0;
    for (instr_nr, value) in to_assemble.iter().enumerate() {
        match value {
            Command::Instr(instruction) => {
                output.push(instruction.encode(&labels, pc).with_context(|| {
                    format!("instruction:'{instruction:?}', nr:'{instr_nr}' labels: '{labels:?}'")
                })? as i32);
                pc += 1;
            }
            Command::Label(_) => {}
            Command::Macro(_macro) => {
                let instructions: Result<Vec<Instruction>> = _macro.into();

                for instruction in instructions? {
                    output.push(instruction.encode(&labels, pc).with_context(|| {
                        format!(
                            "instruction:'{instruction:?}', nr:'{instr_nr}' labels: '{labels:?}'"
                        )
                    })? as i32);
                    pc += 1;
                }
            }

            Command::RawData(data) => {
                data.clone().iter().for_each(|val| output.push(*val));
                pc += data.len()
            }
        }
    }

    Ok(output)
}
// #[test]
// pub fn test() -> Result<()> {
//     let test1 = assemble_commands(vec![
//         Instruction::ADD {
//             rd: 0,
//             rs1: 1,
//             imm: Immediate::Direct(2),
//         }
//         .into(),
//         Instruction::ADD {
//             rd: 0,
//             rs1: 1,
//             imm: Immediate::Direct(1),
//         }
//         .into(),
//     ])?;
//     assert_eq!(test1, vec![0x2C010002, 0x2C010001]);
//
//     let test1 = assemble_commands(vec![
//         Instruction::ADD {
//             rd: 0,
//             rs1: 1,
//             imm: Immediate::Label("one".into()),
//         }
//         .into(),
//         Command::Label("one".into()),
//         Instruction::ADD {
//             rd: 0,
//             rs1: 1,
//             imm: Immediate::Direct(1),
//         }
//         .into(),
//     ])?;
//     assert_eq!(test1, vec![0x2C010001, 0x2C010001]);
//
//     Ok(())
// }
