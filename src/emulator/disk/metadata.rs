use crate::disk::*;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy, Debug, PartialEq)]
pub struct SuperBlock {
    pub total_inodes: u32,
    /// NEEDS TO BE DIVIDABLE BY 8
    pub data_blocks_per_group: u32,
    /// NEEDS TO BE DIVIDABLE BY 8
    pub inodes_per_group: u32,
    pub group_count: u32,
    pub inode_size: u32,
    pub flags: u32,
    pub boot_code_block_base_index: u32,
    pub boot_code_block_count: u32,
}
impl SuperBlock {
    pub fn parse_from_internal_file(file: &File) -> Result<Self> {
        let magic = {
            let mut buffer = [0u8; 4];
            file.read_exact_at(&mut buffer, 0)?;
            u32::from_le_bytes(buffer)
        };
        if magic != 0x00325246 {
            bail!(
                "Magic number is invalid for fr2 file system!:'{magic:#x}' expected:'0x00325246'"
            );
        }

        let mut buffer = [0u8; size_of::<SuperBlock>()];
        file.read_exact_at(&mut buffer, 4)?; // 4-> skip magic number

        Ok(*bytemuck::from_bytes(&buffer))
    }
}
#[derive(Debug)]
pub struct GroupBlockInfo {
    pub base_block_index: u32,
    /// Global index of the block that contains the first data block
    pub first_data_block_index: u32,
    /// Global index of the block that contains the bitmap
    pub data_block_bitmap_index: u32,
    /// Global index of the block that contains the bitmap
    pub inode_bitmap_index: u32,
}
pub struct FsInfo {
    /// Blocks
    pub group_descriptor_indoe_table_size: u32,
    pub inode_table_blocks: u32,
    pub group_size: u32,
    pub total_fs_size: u32,
}
impl FsInfo {
    pub fn get_group_info_for_block(&self, block: u32) -> (GroupBlockInfo, u32) {
        let initial_offset = 1 /*superblock*/ +self.group_descriptor_indoe_table_size;
        let group_index = (block - initial_offset) / self.group_size;
        (self.get_group_info(group_index), group_index)
    }
    pub fn get_group_info(&self, group_index: u32) -> GroupBlockInfo {
        let initial_offset = 1 /*superblock*/ +self.group_descriptor_indoe_table_size;
        let base_block_index = initial_offset + self.group_size * group_index;

        let first_data_block_index = base_block_index + self.group_descriptor_indoe_table_size + 2;
        let data_block_bitmap_index = base_block_index + 0;
        let inode_bitmap_index = base_block_index + 1;
        debug!(
            "get_block_info_for_group: group_index:'{group_index}',  initial_offset:'{initial_offset}', self.group_descriptor_indoe_table_size:'{}', output:'{:?}'",
            self.group_descriptor_indoe_table_size,
            GroupBlockInfo {
                base_block_index,
                first_data_block_index,
                data_block_bitmap_index,
                inode_bitmap_index,
            }
        );
        GroupBlockInfo {
            base_block_index,
            first_data_block_index,
            data_block_bitmap_index,
            inode_bitmap_index,
        }
    }
    pub fn get_start_block_for_group(&self, group_idx: u32) -> u32 {
        1 + self.group_descriptor_indoe_table_size + self.group_size * group_idx
    }

    pub fn new(superblock: &SuperBlock) -> Self {
        let group_descriptor_indoe_table_size = (size_of::<GroupDescriptor>() as u32
            * superblock.group_count)
            .div_ceil(BLOCK_SIZE_BYTES);

        let inode_table_blocks =
            (size_of::<INode>() as u32 * superblock.inodes_per_group).div_ceil(BLOCK_SIZE_BYTES);
        let group_size = superblock.data_blocks_per_group
        + 1/* Block Bitmap */ + 1 /*inode Bitmap*/ + inode_table_blocks;
        let total_fs_size = 1 /* super block */ + group_descriptor_indoe_table_size +superblock.group_count * group_size;
        Self {
            group_descriptor_indoe_table_size,
            inode_table_blocks,
            group_size,
            total_fs_size,
        }
    }
}

pub struct GroupDescriptorIterator {
    i: u32,
    current_block: Vec<u8>,
    current_block_index: u32,
}

impl GroupDescriptorIterator {
    pub fn new() -> Self {
        Self {
            i: 0,
            current_block: vec![],
            current_block_index: 0,
        }
    }
}

const GROUP_DESCRIPTOR_BYTES: u32 = size_of::<GroupDescriptor>() as u32;
impl Iterator for GroupDescriptorIterator {
    type Item = (u32, GroupDescriptor);

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= DISK.superblock.group_count {
            return None;
        }
        let base_block_index = 1 + self.i * GROUP_DESCRIPTOR_BYTES / BLOCK_SIZE_BYTES;
        if self.current_block_index != base_block_index {
            self.current_block_index = base_block_index;

            self.current_block = DISK
                .read_extent(Extent {
                    base_block_index,
                    block_count: 1,
                })
                .ok()?;
        }
        let base_address_in_block = ((self.i * GROUP_DESCRIPTOR_BYTES) % BLOCK_SIZE_BYTES) as usize;
        let bytes = &self.current_block
            [base_address_in_block..base_address_in_block + GROUP_DESCRIPTOR_BYTES as usize];
        self.i += 1;
        Some((self.i - 1, *GroupDescriptor::from_bytes_le(bytes)))
    }
}
#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy, Debug)]
pub struct GroupDescriptor {
    pub free_blocks_count: u32,
    pub free_inodes_count: u32,
}
impl GroupDescriptor {
    pub fn to_bytes_le(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
    pub fn from_bytes_le(bytes: &[u8]) -> &Self {
        bytemuck::from_bytes(bytes)
    }
}

#[derive(Debug)]
pub struct DirEntry {
    pub inode: u32,
    pub name_len: u32,
    pub name: Vec<u8>,
}
impl DirEntry {
    pub fn to_bytes_le(&self) -> Vec<u8> {
        let mut output = vec![];
        output.extend_from_slice(&self.inode.to_le_bytes());
        output.extend_from_slice(&self.name_len.to_le_bytes());
        output.extend(&self.name);
        output
    }
}
#[derive(Debug)]
pub struct BlockOperation {
    pub block_index: u32,
    pub base_block_local_address: u32,
    pub bytes_count: u32,
}
