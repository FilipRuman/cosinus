#[cfg(test)]
pub mod test {
    use std::{
        fs::{File, OpenOptions},
        io::Write,
        sync::LazyLock,
    };

    use crate::{
        dir_handling::project_dir,
        emulator::disk::{self, DISK, GroupDescriptor, SuperBlock},
    };
    use anyhow::{Ok, Result};
    const GROUP_COUNT: u32 = 2048;
    fn super_block() -> SuperBlock {
        let group_descriptor_table_blocks =
            (size_of::<GroupDescriptor>() as u32 * GROUP_COUNT).div_ceil(disk::BLOCK_SIZE_BYTES);

        SuperBlock {
            total_blocks: 2048 * 8 * 3000 + 1 + group_descriptor_table_blocks,
            total_inodes: 122880,
            blocks_per_group: 8 * 3000,
            inodes_per_group: 60,
            group_count: GROUP_COUNT,
            inode_size: size_of::<disk::INode>() as u32,
            root_inode: 0,
            first_data_block: 1 + group_descriptor_table_blocks,
            flags: 0,
        }
    }
    fn get_file() -> File {
        let path = project_dir(disk::INTERNAL_FILE_PATH);
        OpenOptions::new()
            .write(true)
            .read(true)
            .create(true) // if the file may not exist
            .open(path)
            .expect("internal file path for disk is invalid!")
    }

    fn init() -> Result<()> {
        let file = &mut {
            let path = project_dir(disk::INTERNAL_FILE_PATH);
            OpenOptions::new()
                .write(true)
                .truncate(true)
                .read(true)
                .create(true) // if the file may not exist
                .open(path)
                .expect("internal file path for disk is invalid!")
        };
        disk::init_file_system(super_block(), file)?;
        assert_eq!(super_block(), SuperBlock::parse_from_internal_file(file)?);

        file.flush()?;
        Ok(())
    }
    fn super_block_parse() -> Result<()> {
        assert_eq!(
            super_block(),
            SuperBlock::parse_from_internal_file(&DISK.internal_file)?
        );
        Ok(())
    }
    const ENTRY_CONTENTS: LazyLock<Vec<u8>> = LazyLock::new(|| {
        vec![
            25, 210, 21, 52, 79, 25, 88, 076, 251, 02, 32, 21, 092, 252, 21, 76,
        ]
    });
    const ENTRY_NAME: &'static str = "test name";
    fn add_entry() -> Result<()> {
        DISK.add_entry_at_root(ENTRY_NAME, 0, 0, 0, 0, ENTRY_CONTENTS.to_vec())?;
        Ok(())
    }
    fn test_entry() -> Result<()> {
        Ok(())
    }
    pub fn test_all() -> Result<()> {
        init()?;
        super_block_parse()?;
        add_entry()?;
        test_entry()?;
        Ok(())
    }
}
