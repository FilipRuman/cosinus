#[cfg(test)]
pub mod test {
    use std::sync::LazyLock;

    use crate::emulator::disk::{
        self, DISK, SuperBlock,
        helpers::DiskReader,
        mode::{EntryType, Mode, Permissions},
    };
    use anyhow::{Ok, Result};
    use log::debug;
    fn super_block() -> SuperBlock {
        SuperBlock {
            total_inodes: 122880,
            data_blocks_per_group: 8 * 3000,
            inodes_per_group: 60,

            group_count: 2048,
            inode_size: size_of::<disk::INode>() as u32,
            boot_code_block_base_index: 0,
            boot_code_block_count: 0,
            flags: 0,
        }
    }

    fn init() -> Result<()> {
        disk::init_file_system(super_block())?;
        assert_eq!(
            super_block(),
            SuperBlock::parse_from_internal_file(&DISK.internal_file)?
        );

        Ok(())
    }
    fn super_block_parse() -> Result<()> {
        assert_eq!(
            super_block(),
            SuperBlock::parse_from_internal_file(&DISK.internal_file)?
        );
        Ok(())
    }
    fn test_add_root_file() -> Result<()> {
        const ENTRY_CONTENTS: LazyLock<Vec<u8>> = LazyLock::new(|| {
            vec![
                25, 210, 21, 52, 79, 25, 88, 076, 251, 02, 32, 21, 092, 252, 21, 76,
            ]
        });
        const ENTRY_NAME: &'static str = "test name";
        DISK.add_entry_at_root(ENTRY_NAME, 0, 0, 0, 0, &ENTRY_CONTENTS.to_vec())?;

        let entries = DISK.list_entries_under_root()?;
        debug!("entries: {entries:?}");

        assert_eq!(entries.len(), 1);
        let name_bytes: Vec<u8> = ENTRY_NAME.bytes().collect();
        assert_eq!(entries[0].name, name_bytes);
        let inode = DISK.read_inode(entries[0].inode)?;
        assert_eq!(
            DiskReader::read_inode_contents(&inode)
                .take(ENTRY_CONTENTS.len())
                .collect::<Vec<u8>>(),
            ENTRY_CONTENTS.to_vec()
        );

        Ok(())
    }
    fn test_many_files() -> Result<()> {
        let contents = vec![21u8; 1 << 8];

        let mode = Mode {
            entry_type: EntryType::File,
            user: Permissions::READ | Permissions::WRITE | Permissions::EXECUTE, // Random permission
            other: Permissions::READ | Permissions::WRITE | Permissions::EXECUTE,
            group: Permissions::READ | Permissions::WRITE | Permissions::EXECUTE,
        };

        for i in 0..1 << 12 {
            let name = i.to_string();
            DISK.add_entry_at_root(&name, mode.into(), 0, 0, 0, &contents)?;
        }
        Ok(())
    }
    fn test_big_file() -> Result<()> {
        let contents = vec![21u8; 1 << 12];

        let mode = Mode {
            entry_type: EntryType::File,
            user: Permissions::READ | Permissions::WRITE | Permissions::EXECUTE, // Random permission
            other: Permissions::READ | Permissions::WRITE | Permissions::EXECUTE,
            group: Permissions::READ | Permissions::WRITE | Permissions::EXECUTE,
        };

        let name = "BIG file";
        DISK.add_entry_at_root(&name, mode.into(), 0, 0, 0, &contents)?;
        Ok(())
    }
    fn test_inode_mode() -> Result<()> {
        let mode = Mode {
            entry_type: EntryType::Directory,
            user: Permissions::READ | Permissions::WRITE | Permissions::EXECUTE, // Random permission
            other: Permissions::READ | Permissions::WRITE | Permissions::EXECUTE,
            group: Permissions::READ | Permissions::WRITE | Permissions::EXECUTE,
        };

        assert_eq!(mode.entry_type, EntryType::Directory);
        let num: u16 = mode.into();
        assert_eq!(num, 16895);
        let mode_from_num: Mode = num.try_into()?;

        assert_eq!(mode, mode_from_num);
        Ok(())
    }
    #[test]
    pub fn test_all() -> Result<()> {
        test_inode_mode()?;
        init()?;
        super_block_parse()?;
        test_add_root_file()?;
        test_big_file()?;
        test_many_files()?;
        Ok(())
    }
}
