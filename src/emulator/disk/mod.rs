pub mod allocation;
pub mod entry_traversal;
pub mod helpers;
pub mod inode;
pub mod io_device;
pub mod metadata;
pub mod mode;
pub mod test;
use anyhow::{Context, Result, bail};
use bytemuck::{Pod, Zeroable};
use log::debug;
use metadata::*;
use std::{
    fs::{File, OpenOptions},
    io::{Seek, Write},
    os::unix::fs::FileExt,
    sync::LazyLock,
};
const ROOT_INODE_INDEX: u32 = 0;

use crate::{
    dir_handling::project_dir,
    emulator::disk::{
        helpers::{DiskReader, DiskWriteStream},
        inode::INode,
        mode::{EntryType, Mode, Permissions},
    },
};

#[cfg(test)]
pub(crate) const INTERNAL_FILE_PATH: &str = "./disk/test_disk.fr2";

#[cfg(not(test))]
pub(crate) const INTERNAL_FILE_PATH: &str = "./disk/disk.fr2";

pub static DISK: LazyLock<Disk> = LazyLock::new(|| Disk::new());

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy, Debug, PartialEq)]
pub struct Extent {
    pub base_block_index: u32,
    pub block_count: u32,
}
impl Default for Extent {
    fn default() -> Self {
        Self {
            base_block_index: Default::default(),
            block_count: Default::default(),
        }
    }
}
impl Extent {
    pub fn zero() -> Self {
        Extent {
            base_block_index: 0,
            block_count: 0,
        }
    }
    fn end_block(&self) -> u32 {
        self.base_block_index + self.block_count
    }
    pub fn is_connected_to(&self, other: &Self) -> bool {
        self.end_block() + 1 == other.base_block_index
            || other.end_block() + 1 == self.base_block_index
    }
    pub fn blocks(&self) -> Vec<u32> {
        let mut output = vec![];
        for i in 0..self.block_count {
            output.push(self.base_block_index + i);
        }
        output
    }
    pub fn single(base_block_index: u32) -> Self {
        Self {
            base_block_index,
            block_count: 1,
        }
    }
    pub fn bytes(&self) -> u32 {
        self.block_count * BLOCK_SIZE_BYTES
    }

    pub fn base_byte_address(&self) -> u64 {
        self.base_block_index as u64 * BLOCK_SIZE_BYTES as u64
    }
}
pub struct Disk {
    pub internal_file: File,
    pub superblock: SuperBlock,
    pub fs_info: FsInfo,
}
pub(crate) const BLOCK_SIZE_BYTES: u32 = 0x1000;
impl Disk {
    pub fn write_extent(&self, extent: Extent, value: &Vec<u8>) -> Result<()> {
        if value.len() as u32 != extent.bytes() {
            bail!(
                "The amount of bytes:'{}' in the input_buffer wasn't equal to the amount of bytes required by the block count- BLOCK_SIZE_BYTES*block_count:'{}' ",
                value.len(),
                extent.bytes()
            );
        }

        self.internal_file
            .write_all_at(&value[..], extent.base_byte_address() as u64)
            .context("write data to internal file")
    }
    pub fn modify_extent<F>(&self, extent: Extent, modify_fn: F) -> Result<()>
    where
        F: Fn(&mut Vec<u8>),
    {
        let mut value = self.read_extent(extent)?;
        modify_fn(&mut value);
        self.write_extent(extent, &value)?;
        Ok(())
    }
    pub fn read_extent(&self, extent: Extent) -> Result<Vec<u8>> {
        let mut buffer = vec![0u8; extent.bytes() as usize];
        let offset = extent.base_byte_address();
        self.internal_file
            .read_at(&mut buffer[..], offset as u64)
            .context("Read from a block did not succeed this probably means that the file system was corrupted.")?;
        Ok(buffer)
    }
    pub fn new() -> Self {
        let path = project_dir(INTERNAL_FILE_PATH);

        let internal_file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(path)
            .expect("internal file path for disk is invalid!");
        let superblock = SuperBlock::parse_from_internal_file(&internal_file)
            .expect("parsing metadata for a file system did not succeed!");
        Self {
            internal_file,
            fs_info: FsInfo::new(&superblock),
            superblock,
        }
    }
    pub fn read_inode(&self, inode_index: u32) -> Result<INode> {
        let group_idx = inode_index / self.superblock.inodes_per_group;
        let idx_inside_group = inode_index - group_idx * self.superblock.inodes_per_group;
        let block_of_the_inode_table =
            size_of::<INode>() as u32 * idx_inside_group / BLOCK_SIZE_BYTES;
        if block_of_the_inode_table
            != size_of::<INode>() as u32 * (idx_inside_group + 1) / BLOCK_SIZE_BYTES
        {
            bail!(
                "Current implementation doesn't support data of inode being split across many blocks- NEEDS TO BE REDESIGNED LIKE THE write_inode"
            );
        }
        let block_to_read_idx = self.fs_info.get_start_block_for_group(group_idx) + 2 /*Move past bitmaps*/ + block_of_the_inode_table;

        let block = self.read_extent(Extent {
            base_block_index: block_to_read_idx,
            block_count: 1,
        })?;
        let start_byte = idx_inside_group * size_of::<INode>() as u32;
        let bytes_of_inode = &block[start_byte as usize..start_byte as usize + size_of::<INode>()];

        debug!(
            "read_inode- inode_index:'{inode_index}', block:'{block_to_read_idx}', start_byte:'{start_byte}', bytes_count:'{}', value:{:?}",
            size_of::<INode>(),
            INode::from_bytes_le(bytes_of_inode)
        );
        Ok(*INode::from_bytes_le(bytes_of_inode))
    }
    pub fn write_inode(&self, inode_index: u32, value: &INode) -> Result<()> {
        let group_idx = inode_index / self.superblock.inodes_per_group;
        let idx_inside_group = inode_index - group_idx * self.superblock.inodes_per_group;

        let write_extent = Extent {
            base_block_index: self.fs_info.get_start_block_for_group(group_idx) + 2, /*Move past bitmaps*/
            block_count: self.fs_info.inode_table_blocks,
        };
        let bytes = value.to_bytes_le();
        let mut writer = DiskWriteStream::write_to_extent(write_extent)?;
        debug!(
            "write_inode: idx_inside_group:'{idx_inside_group}', bytes_to_skip:'{}', write_extent:'{write_extent:?}', inode:'{value:?}'",
            idx_inside_group * size_of::<INode>() as u32
        );
        writer.skip(idx_inside_group * size_of::<INode>() as u32)?;
        writer.write(&bytes.to_vec())?;
        writer.apply_changes()?;
        Ok(())
    }

    pub fn read_group_descriptor(&self, group_index: u32) -> Result<GroupDescriptor> {
        let mut reader = DiskReader::read_extent(Extent {
            base_block_index: 1,
            block_count: self.fs_info.group_descriptor_indoe_table_size,
        });
        let _ = (&mut reader).skip(group_index as usize * size_of::<GroupDescriptor>());
        let bytes: Vec<u8> = reader.take(size_of::<GroupDescriptor>()).collect();
        Ok(*GroupDescriptor::from_bytes_le(&bytes))
    }
    pub fn write_group_descriptor(&self, group_index: u32, value: GroupDescriptor) -> Result<()> {
        let block_idx = group_index * size_of::<GroupDescriptor>() as u32 / BLOCK_SIZE_BYTES + 1;
        let address_in_block =
            (group_index * size_of::<GroupDescriptor>() as u32) % BLOCK_SIZE_BYTES;
        let descriptor_bytes = value.to_bytes_le();

        self.modify_extent(
            Extent {
                base_block_index: block_idx,
                block_count: 1,
            },
            |bytes| {
                for i in 0..descriptor_bytes.len() {
                    bytes[address_in_block as usize + i] = descriptor_bytes[i]
                }
            },
        )
    }

    pub fn append_to_inode_contents(&self, inode: &INode, to_append: &Vec<u8>) -> Result<()> {
        debug!(
            "append_to_inode_contents:{inode:?}, allocated_bytes:{} to_append:{to_append:?}",
            inode.allocated_bytes()
        );
        let mut writer = DiskWriteStream::write_to_inode_contents(inode)?;
        writer.skip(inode.content_size)?;
        writer.write(to_append)?;
        writer.apply_changes()?;
        Ok(())
    }
    pub fn append_with_alloc_to_inode_contents(
        &self,
        inode: &mut INode,
        inode_index: u32,
        to_append: &Vec<u8>,
    ) -> Result<()> {
        debug!(
            "0. ->append_with_alloc_to_inode_contents- root_inode:{:?}",
            self.read_inode(0)?
        );
        while (inode.allocated_bytes() - inode.content_size as u64) <= to_append.len() as u64 {
            let alloc = self.allocate_blocks(1)?;
            let mut appended_to_extents = false;
            for extent in inode.extents.iter_mut() {
                if extent.base_block_index == 0 {
                    *extent = alloc;
                    debug!("ALLOCATE: inode{inode:?} allocation:{alloc:?}");
                    appended_to_extents = true;
                    break;
                } else if extent.is_connected_to(&alloc) {
                    let new_extent = Extent {
                        base_block_index: extent.base_block_index,
                        block_count: extent.block_count + alloc.block_count,
                    };
                    *extent = new_extent;
                }
            }
            if !appended_to_extents {
                self.defrag_inode_allocation(inode, inode_index)?;
            }
        }
        self.append_to_inode_contents(inode, &to_append)?;
        debug!(
            "1. ->append_with_alloc_to_inode_contents- root_inode:{:?}",
            self.read_inode(0)?
        );
        inode.content_size += to_append.len() as u32;
        self.write_inode(inode_index, inode)?;

        debug!(
            "2. ->append_with_alloc_to_inode_contents- root_inode:{:?}",
            self.read_inode(0)?
        );

        Ok(())
    }

    pub fn add_entry_at_root(
        &self,
        file_name: &str,
        mode: u16,
        uid: u32,
        gid: u32,
        flags: u32,
        contents: &Vec<u8>,
    ) -> Result<()> {
        let allocated_inode_idx: u32 = {
            let new_inode_idx = self.allocate_inode_idx()?;
            let mut root_inode = self.read_inode(ROOT_INODE_INDEX)?;
            let file_entry = DirEntry {
                inode: new_inode_idx,
                name_len: file_name.len() as u32,
                name: file_name.bytes().collect(),
            };
            debug!(
                "add_entry_at_root- link new entry to root:, file_entry:'{file_entry:?}' new_inode_idx:{new_inode_idx}"
            );

            self.append_with_alloc_to_inode_contents(
                &mut root_inode,
                ROOT_INODE_INDEX,
                &file_entry.to_bytes_le(),
            )?;
            new_inode_idx
        };

        let blocks_needed_for_contents = (contents.len() as u32).div_ceil(BLOCK_SIZE_BYTES);
        let allocation = self.allocate_blocks(blocks_needed_for_contents)?;
        let mut extents = [Extent::default(); 12];
        extents[0] = allocation;
        let mut inode = INode {
            mode,
            links: 1,
            uid,
            gid,
            content_size: 0,
            extents,
            flags,
            _pad: 0,
        };
        debug!(
            "add_entry_at_root-write new inode: allocated_inode_idx:'{allocated_inode_idx}', contents:'{contents:?}' inode:{inode:?}"
        );
        self.append_with_alloc_to_inode_contents(&mut inode, allocated_inode_idx, &contents)?;

        inode.content_size = contents.len() as u32;
        self.write_inode(allocated_inode_idx, &inode)?;
        Ok(())
    }
}
pub fn init_file_system(metadata: SuperBlock) -> Result<()> {
    let internal_file = &mut {
        let path = project_dir(INTERNAL_FILE_PATH);
        OpenOptions::new()
            .write(true)
            .truncate(true)
            .read(true)
            .create(true) // if the file may not exist
            .open(path)
            .expect("internal file path for disk is invalid!")
    };

    let fs_info = FsInfo::new(&metadata);
    // Allocate enough space
    {
        const BATCHES: usize = 100;
        let buff = &vec![0u8; fs_info.total_fs_size as usize / BATCHES];
        for _ in 0..BATCHES {
            internal_file.write_all(buff)?;
        }
        internal_file.write_all(&vec![
            0u8;
            fs_info.total_fs_size as usize
                - buff.len() * BATCHES as usize
        ])?;
    }

    internal_file.rewind()?;
    // Super Block
    internal_file.write_all(&(0x00325246_u32.to_le_bytes()))?; // magic
    internal_file.write_all(&metadata.total_inodes.to_le_bytes())?;
    internal_file.write_all(&metadata.data_blocks_per_group.to_le_bytes())?;
    internal_file.write_all(&metadata.inodes_per_group.to_le_bytes())?;
    internal_file.write_all(&metadata.group_count.to_le_bytes())?;
    internal_file.write_all(&metadata.inode_size.to_le_bytes())?;
    internal_file.write_all(&metadata.flags.to_le_bytes())?;
    internal_file.write_all(&metadata.boot_code_block_base_index.to_le_bytes())?;
    internal_file.write_all(&metadata.boot_code_block_count.to_le_bytes())?;

    internal_file.seek(std::io::SeekFrom::Start(1 * BLOCK_SIZE_BYTES as u64))?;
    // Group Descriptor Table
    for _ in 0..metadata.group_count {
        let value: GroupDescriptor = GroupDescriptor {
            free_blocks_count: metadata.data_blocks_per_group,
            free_inodes_count: metadata.inodes_per_group,
        };
        let bytes = &value.to_bytes_le();
        internal_file.write_all(bytes)?;
    }
    // Groups can be skipped because they will be all zeros

    // Just initialize the root inode
    let root_inode = INode {
        mode: Mode {
            entry_type: EntryType::Directory,
            user: Permissions::READ | Permissions::WRITE | Permissions::EXECUTE, // Random permission
            other: Permissions::READ | Permissions::WRITE | Permissions::EXECUTE,
            group: Permissions::READ | Permissions::WRITE | Permissions::EXECUTE,
        }
        .into(),
        content_size: 0,
        uid: 0,
        _pad: 0,
        extents: [Extent {
            base_block_index: 0,
            block_count: 0,
        }; 12],
        flags: 0,
        gid: 0,
        links: 0,
    };
    // It should allocate the root node - it is first
    DISK.allocate_inode_idx()?;
    DISK.write_inode(ROOT_INODE_INDEX, &root_inode)?;

    Ok(())
}
