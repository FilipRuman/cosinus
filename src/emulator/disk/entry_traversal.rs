use crate::emulator::disk::{
    DirEntry, Disk, EntryType, INode, Mode, ROOT_INODE_INDEX, helpers::DiskReader,
};
use anyhow::{Result, bail};
use log::debug;

pub struct DirEntriesIterator {
    pub bytes_iterator: DiskReader,
}

impl DirEntriesIterator {
    pub fn new(inode: &INode) -> Result<Self> {
        let mode: Mode = inode.mode.try_into()?;
        debug!("DirEntriesIterator::new: inode:'{inode:?}', mode:'{mode:?}'");
        // IsFile
        if mode.entry_type != EntryType::Directory {
            bail!("Inode isn't an Directory!");
        }

        Ok(Self {
            bytes_iterator: DiskReader::read_inode_contents(inode),
        })
    }
    pub fn bytes_to_skip_until_free_space(&mut self) -> u32 {
        self.for_each(drop); // Move to the end
        self.bytes_iterator.total_skipped_bytes - 4 // -4: next() will always check for the inode before returning None 
    }
}
impl Iterator for DirEntriesIterator {
    type Item = DirEntry;

    fn next(&mut self) -> Option<Self::Item> {
        debug!(
            "DirEntriesIterator::next: bytes_iterator.current_byte_index:'{}'",
            self.bytes_iterator.current_byte_index
        );
        let inode = self.bytes_iterator.next_u32()?;
        debug!("DirEntriesIterator::next: inode:'{inode}'");
        if inode == 0 {
            return None;
        }
        let name_len = self.bytes_iterator.next_u32()?;
        let name: Vec<u8> = (&mut self.bytes_iterator).take(name_len as usize).collect();
        Some(DirEntry {
            name,
            inode,
            name_len,
        })
    }
}

impl Disk {
    pub fn list_entries_under_root(&self) -> Result<Vec<DirEntry>> {
        let root_inode = self.read_inode(ROOT_INODE_INDEX)?;

        let output = DirEntriesIterator::new(&root_inode)?.collect();
        Ok(output)
    }
}
