mod parsing;

use std::{collections::HashMap, fs::File, io::Read, path::PathBuf};

use anyhow::{Context, Ok, Result, bail};

use crate::assembler::{
    self,
    instruction::{Command, Immediate, Instruction},
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct LinkerSettings {
    pub entry_point_label: String,
    pub files_to_link: Vec<String>,
}

struct FileLinkData {
    pub path: String,
    pub base_address: u32,
    pub size_bytes: u32,
    pub relative_offset_labels: HashMap<String, u32>,
    pub commands: Vec<Command>,
}
fn get_link_data_for_file(file_path: &PathBuf, base_address: u32) -> Result<FileLinkData> {
    let mut file = File::open(&file_path)?;
    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents)?;
    let commands = assembler::instruction_parsing::parse_program(&file_contents)?;
    let linking_data = assembler::get_data_for_linking(&commands)?;

    Ok(FileLinkData {
        path: file_path.to_str().unwrap().to_string(),
        base_address,
        commands: commands,
        size_bytes: linking_data.size_bytes,
        relative_offset_labels: linking_data.declared_labels,
    })
}
fn generate_metadata(
    settings: LinkerSettings,
    offsets_for_labels: &HashMap<String, u32>,
) -> Result<Vec<i32>> {
    let mut output = vec![];
    let entry_point = offsets_for_labels
        .get(&settings.entry_point_label)
        .with_context(|| {
            format!(
                "entry point label doesn't exist-'{}'",
                settings.entry_point_label
            )
        })?;
    output.push(*entry_point as i32);
    Ok(output)
}
/// Uses link script(./link.toml) inside of the specified dir to generate elf.
pub fn generate_elf_for_dir(dir_path: PathBuf, boot_code: bool) -> Result<Vec<i32>> {
    let settings = {
        let mut link_script_path = dir_path.clone();
        link_script_path.push("link.toml");
        if !link_script_path.is_file() {
            bail!("there was no link script(link.toml) in the dir that was used to create elf")
        }
        LinkerSettings::from_file(&link_script_path)
            .with_context(|| format!("parsing link script at path: {link_script_path:?}"))?
    };

    generate_elf(dir_path, settings, boot_code)
}

/// This is not the most efficient implementation in the world but it works
/// boot_code: adds a jump to the entry point before the metadata. this means that you can just use
/// this code directly.
pub fn generate_elf(
    base_file_path: PathBuf,
    settings: LinkerSettings,
    boot_code: bool,
) -> Result<Vec<i32>> {
    // Needs to be updated upon elf spec update
    const METADATA_BYTES: u32 = 4;
    const ADDITIONAL_BOOT_CODE_BYTES: u32 = 4;
    let mut parsed_files = Vec::with_capacity(settings.files_to_link.len());
    {
        let mut current_address = METADATA_BYTES + ADDITIONAL_BOOT_CODE_BYTES;
        for path in &settings.files_to_link {
            let mut path_buf = base_file_path.clone();
            path_buf.push(path);

            let file_data =
                get_link_data_for_file(&path_buf, current_address).with_context(|| {
                    format!(
                        "while creating linking data for a file at path: '{:?}'",
                        path_buf
                    )
                })?;
            current_address += file_data.size_bytes;
            parsed_files.push(file_data);
        }
    }

    let mut offsets_for_labels = HashMap::new();
    for file in &parsed_files {
        for label in &file.relative_offset_labels {
            let global_addr = file.base_address + label.1;
            offsets_for_labels.insert(label.0.to_string(), global_addr);
        }
    }

    let mut output: Vec<i32> = vec![];
    // Adds a jump to the entry point before the metadata. This means that you can just use
    // this code directly.
    if boot_code {
        let commands = vec![
            Instruction::JMP {
                imm: Immediate::Label(settings.entry_point_label.to_string()),
            }
            .into(),
        ];
        let mut assembled_code =
            assembler::assemble_with_linker_data(&offsets_for_labels, 0, commands)
                .context("assembling jump instruction for boot code")?;
        output.append(&mut assembled_code);
    }

    {
        let mut metadata = generate_metadata(settings, &offsets_for_labels)?;
        output.append(&mut metadata);
    }

    for file in parsed_files {
        let mut assembled_code = assembler::assemble_with_linker_data(
            &offsets_for_labels,
            file.base_address,
            file.commands,
        )
        .with_context(|| format!("while assembling file at path: {}", file.path))?;

        output.append(&mut assembled_code);
    }

    Ok(output)
}
