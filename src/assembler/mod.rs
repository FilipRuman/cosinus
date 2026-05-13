use std::collections::HashMap;

use crate::assembler::{
    instruction::{Command, Instruction},
    instruction_parsing::parse_program,
};

pub mod instruction;
pub mod instruction_parsing;
pub mod parsing_test;
mod test;
use anyhow::{Context, Ok, Result, bail};
use log::debug;

pub fn assemble_from_string(input: &str) -> Result<Vec<i32>> {
    let parsed = parse_program(input)?;
    assemble_without_linker_data(parsed)
}
#[derive(Debug)]
pub struct DataForLinking {
    pub size_bytes: u32,
    pub declared_labels: HashMap<String, u32>,
}
pub fn get_data_for_linking(to_assemble: &Vec<Command>) -> Result<DataForLinking> {
    let mut declared_labels = HashMap::new();

    let mut pc = 0;
    for value in to_assemble {
        match value {
            Command::Instr(_) => {
                pc += 1;
            }

            Command::Macro(_macro) => {
                let instructions: Result<Vec<Instruction>> = _macro.into();
                pc += instructions?.len() as u32;
            }
            Command::Label(name) => {
                declared_labels.insert(name.clone(), (pc - 1) * 4);
            }
            Command::RawData(data) => pc += data.len() as u32,
        }
    }

    debug!("Assemble labels:{declared_labels:#?}");
    Ok(DataForLinking {
        declared_labels,
        size_bytes: pc * 4,
    })
}
pub fn assemble_with_linker_data(
    global_addresses_for_labels: &HashMap<String, u32>,
    base_address: u32,
    to_assemble: Vec<Command>,
) -> Result<Vec<i32>> {
    //TODO: MAKE A SINGLE FUNCTION FOR THIS
    let named_immediate_eval = |name: &str, immediate_size: u8, pc: u32| -> Result<i32> {
        if let Some(complex_immidiate) = name.strip_suffix(')') {
            let mut to_evaluate = String::new();
            for value in complex_immidiate.split(' ') {
                if let Some(label_adress) = global_addresses_for_labels.get(value) {
                    let offset = *label_adress as i64 - base_address as i64 - (pc * 4) as i64; // i64 to avoid
                    to_evaluate += &offset.to_string();
                    to_evaluate += " ";
                } else {
                    to_evaluate += value;
                    to_evaluate += " ";
                }
            }
            let offset = evalexpr::eval_int(&to_evaluate).with_context(|| {
                format!("evaluation of an complex immediate did not succeedi, value:'{name}'")
            })?;

            if offset.abs() > 1 << immediate_size - 1
            /* -1: signed integer*/
            {
                bail!(
                    "Value of the immediate- {offset} calculated during linking of labels was higher than it is possible to store in immediate of size 2^{immediate_size}"
                );
            }

            Ok(offset as i32)
        } else if let Some(label_adress) = global_addresses_for_labels.get(name) {
            let offset = *label_adress as i64 - base_address as i64 - (pc * 4) as i64; // i64 to avoid
            // any overflows
            debug!(
                "assemble_with_linker_data.label_handle: name:'{name}' base_address:'{base_address}' global:'{pc}' offset:'{offset}'"
            );
            if offset.abs() > 1 << immediate_size - 1
            /* -1: signed integer*/
            {
                bail!(
                    "Value of the immediate- {offset} calculated during linking of labels was higher than it is possible to store in immediate of size 2^{immediate_size}"
                );
            }

            Ok(offset as i32)
        } else {
            bail!("Name of immediate:'{name}' is invalid");
        }
    };
    assemble(to_assemble, named_immediate_eval)
}
fn assemble<F>(to_assemble: Vec<Command>, named_immediate_eval: F) -> Result<Vec<i32>>
where
    F: Fn(&str, u8, u32) -> Result<i32>,
{
    let mut output = Vec::with_capacity(to_assemble.len());
    let mut pc = 0;
    for (instr_nr, value) in to_assemble.iter().enumerate() {
        match value {
            Command::Instr(instruction) => {
                output.push(
                    instruction
                        .encode(&named_immediate_eval, pc)
                        .with_context(|| {
                            format!("instruction:'{instruction:?}', nr:'{instr_nr}'")
                        })? as i32,
                );
                pc += 1;
            }
            Command::Label(_) => {}
            Command::Macro(_macro) => {
                let instructions: Result<Vec<Instruction>> = _macro.into();

                for instruction in instructions? {
                    output.push(instruction.encode(&named_immediate_eval, pc).with_context(
                        || format!("instruction:'{instruction:?}', nr:'{instr_nr}'"),
                    )? as i32);
                    pc += 1;
                }
            }

            Command::RawData(data) => {
                data.clone().iter().for_each(|val| output.push(*val));
                pc += data.len() as u32
            }
        }
    }
    Ok(output)
}
pub fn assemble_without_linker_data(to_assemble: Vec<Command>) -> Result<Vec<i32>> {
    let linking_data = get_data_for_linking(&to_assemble)?;
    let labels = linking_data.declared_labels;

    //TODO: MAKE A SINGLE FUNCTION FOR THIS
    let named_immediate_eval = |name: &str, immediate_size: u8, pc: u32| -> Result<i32> {
        if let Some(complex_immidiate) = name.strip_suffix(')') {
            let mut to_evaluate = String::new();
            for value in complex_immidiate.split(' ') {
                if let Some(label_adress) = labels.get(value) {
                    let offset = *label_adress as i64 - (pc * 4) as i64; // i64 to avoid
                    to_evaluate += &offset.to_string();
                    to_evaluate += " ";
                } else {
                    to_evaluate += value;
                    to_evaluate += " ";
                }
            }
            let offset = evalexpr::eval_int(&to_evaluate).with_context(|| {
                format!("evaluation of an complex immediate did not succeedi, value:'{name}'")
            })?;

            if offset.abs() > 1 << immediate_size - 1
            /* -1: signed integer*/
            {
                bail!(
                    "Value of the immediate- {offset} calculated during linking of labels was higher than it is possible to store in immediate of size 2^{immediate_size}"
                );
            }

            Ok(offset as i32)
        } else if let Some(label_adress) = labels.get(name) {
            let offset = *label_adress as i64 - (pc * 4) as i64; // i64 to avoid
            // any overflows
            debug!(
                "assemble_without_linker_data.label_handle: name:'{name}'  global:'{pc}' offset:'{offset}'"
            );
            if offset.abs() > 1 << immediate_size - 1
            /* -1: signed integer*/
            {
                bail!(
                    "Value of the immediate- {offset} calculated during linking of labels was higher than it is possible to store in immediate of size 2^{immediate_size}"
                );
            }

            Ok(offset as i32)
        } else {
            bail!("Name of immediate:'{name}' is invalid");
        }
    };

    assemble(to_assemble, named_immediate_eval)
}
