use crate::emulator::disk::*;
impl DiskReader {
    pub fn skip_bytes(&mut self, mut n: usize) {
        self.total_skipped_bytes += n as u32;
        self.current_byte_index += n as u32;
        // First skip buffered bytes
        let buffered = self.current_bytes.len();

        if n < buffered {
            let new_len = buffered - n;
            self.current_bytes.truncate(new_len);
            return;
        }

        n -= buffered;
        self.current_bytes.clear();

        // Skip whole blocks without reading them
        let whole_blocks = n / BLOCK_SIZE_BYTES as usize;
        let remaining = n % BLOCK_SIZE_BYTES as usize;

        let blocks_to_remove = whole_blocks.min(self.blocks_left.len());

        let new_len = self.blocks_left.len() - blocks_to_remove;
        self.blocks_left.truncate(new_len);

        // If no remainder, we're done
        if remaining == 0 {
            return;
        }

        // Need to land inside the next block
        let next_block = match self.blocks_left.pop() {
            Some(b) => b,
            None => return,
        };

        let mut new_bytes = DISK
            .read_extent(Extent::single(next_block))
            .expect("reading block while skipping inode contents");
        self.current_block = next_block;
        self.current_byte_index = remaining as u32;

        // Keep only unread bytes
        let keep = BLOCK_SIZE_BYTES as usize - remaining;
        new_bytes.truncate(keep);

        self.current_bytes = new_bytes;
    }
}
/// Usage:
/// Works like a stream- it will copy disk contents based on the reader initialization function, and override certain parts of its contents.
/// Writes will override data at the current cursor position and move the cursor forward, seamlessly handling loading new blocks.
/// This allows for easy writes to the INode contents without thinking about fragmentation of the
/// data.
pub struct DiskWriteStream {
    pub blocks_left: Vec<u32>,
    pub cursor: u32,
    pub modified_blocks: Vec<(u32, Vec<u8>)>,
    pub current_block_index: u32,
    pub current_bytes: Vec<u8>,
    pub was_current_block_modified: bool,
}

impl DiskWriteStream {
    pub fn handle_new_block(&mut self) -> Result<()> {
        debug!("handle_new_block: blocks_left:{:?}", self.blocks_left);
        if self.was_current_block_modified {
            self.modified_blocks
                .push((self.current_block_index, self.current_bytes.clone()));
        }
        self.was_current_block_modified = false;
        let new_block_index = self
            .blocks_left
            .pop()
            .context("there wasn't enough blocks to use")?;
        self.current_block_index = new_block_index;
        self.cursor = 0;
        let bytes = DISK.read_extent(Extent::single(new_block_index))?;
        self.current_bytes = bytes;

        Ok(())
    }

    pub fn write_to_extent(extent: Extent) -> Result<Self> {
        let mut blocks_left = extent.blocks();
        blocks_left.reverse(); // needed because Vec::pop pops from the back
        let mut writer = Self {
            blocks_left,
            cursor: 0,
            modified_blocks: vec![],
            current_block_index: 0,
            current_bytes: vec![],
            was_current_block_modified: false,
        };
        writer.handle_new_block().with_context(|| {
            format!(
                "Supplied extent has zero size: extent.block_count:{}",
                extent.block_count
            )
        })?;

        // TEMP:
        let before = &writer.current_bytes[0..size_of::<INode>()].to_owned();
        debug!(
            "write_to_extent: current_block_index:'{}' before:'{:?}' disk:'{:?}'",
            writer.current_block_index,
            INode::from_bytes_le(before),
            INode::from_bytes_le(&DISK.read_extent(Extent::single(7))?[0..size_of::<INode>()]),
        );
        Ok(writer)
    }

    pub fn write_to_inode_contents(inode: &INode) -> Result<Self> {
        let mut blocks_left = inode.blocks();
        blocks_left.reverse(); // needed because Vec::pop pops from the back
        let mut writer = Self {
            blocks_left,
            cursor: 0,
            modified_blocks: vec![],
            current_block_index: 0,
            current_bytes: vec![],
            was_current_block_modified: false,
        };
        writer.handle_new_block().with_context(|| {
            format!(
                "Supplied inode has zero size: inode.blocks().len():'{}'",
                inode.blocks().len()
            )
        })?;
        Ok(writer)
    }
    /// Only write to blocks that were modified.
    pub fn apply_changes(&self) -> Result<()> {
        for (block_index, bytes) in &self.modified_blocks {
            DISK.write_extent(Extent::single(*block_index), bytes)?;
        }
        if self.was_current_block_modified {
            DISK.write_extent(
                Extent::single(self.current_block_index),
                &self.current_bytes,
            )?;
        }
        // TEMP:

        assert_eq!(
            DISK.read_extent(Extent::single(self.current_block_index))?,
            self.current_bytes,
        );
        Ok(())
    }
    /// Move cursor by n bytes forward.
    pub fn skip(&mut self, n: u32) -> Result<()> {
        let blocks_to_skip = self.cursor + n / BLOCK_SIZE_BYTES;
        if blocks_to_skip != 0 {
            // Skip blocks in the middle that don't matter anyway
            if blocks_to_skip > 1 {
                let blocks_to_pop_immediately = blocks_to_skip - 1;
                for _ in 0..blocks_to_pop_immediately {
                    self.blocks_left.pop();
                }
            }
            self.handle_new_block()?;
        }
        let new_cursor = self.cursor + n - blocks_to_skip * BLOCK_SIZE_BYTES;
        self.cursor = new_cursor;
        Ok(())
    }
    // This could be done a faster by writing directly to the memory of the vec, but this would be very
    // unsafe.
    pub fn write(&mut self, bytes: &Vec<u8>) -> Result<()> {
        // TEMP:
        let before = &self.current_bytes[0..size_of::<INode>()].to_owned();
        debug!("write: end root_value:'{:?}'", INode::from_bytes_le(before),);
        for byte in bytes {
            self.current_bytes[self.cursor as usize] = *byte;

            self.was_current_block_modified = true;
            self.cursor += 1;
            if self.cursor >= BLOCK_SIZE_BYTES {
                // warn!("write-self.cursor == BLOCK_SIZE_BYTES");
                self.handle_new_block()?;
            }
        }
        let after = &self.current_bytes[0..size_of::<INode>()].to_owned();
        Ok(())
    }
}

#[derive(Debug)]
pub struct DiskReader {
    blocks_left: Vec<u32>,
    current_bytes: Vec<u8>,
    pub current_block: u32,
    pub total_skipped_bytes: u32,
    pub current_byte_index: u32,
}
impl DiskReader {
    pub fn read_extent(extent: Extent) -> Self {
        debug!(
            "read_extent::new()- inode:'{extent:?}', blocks.len:'{}'",
            extent.blocks().len()
        );
        Self {
            blocks_left: extent.blocks(),
            current_bytes: vec![],
            current_byte_index: 0,
            current_block: 0,
            total_skipped_bytes: 0,
        }
    }
    pub fn read_inode_contents(inode: &INode) -> Self {
        debug!(
            "read_inode_contents::new()- inode:'{inode:?}', blocks.len:'{}'",
            inode.blocks().len()
        );
        Self {
            blocks_left: inode.blocks(),
            current_bytes: vec![],
            current_byte_index: 0,
            current_block: 0,
            total_skipped_bytes: 0,
        }
    }
}

impl DiskReader {
    pub fn next_u32(&mut self) -> Option<u32> {
        Some(u32::from_le_bytes([
            self.next()?,
            self.next()?,
            self.next()?,
            self.next()?,
        ]))
    }
}
impl Iterator for DiskReader {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(byte) = self.current_bytes.pop() {
                self.current_byte_index += 1;
                self.total_skipped_bytes += 1;
                return Some(byte);
            }

            let block = self.blocks_left.pop()?;
            self.current_bytes = DISK
                .read_extent(Extent::single(block))
                .expect("reading block while iterating inode contents");
            self.current_bytes.reverse();

            debug!("INodeContentsIterator - pop from blocks left: block_index:'{block}'");
            self.current_block = block;
            self.current_byte_index = 0;
        }
    }
}

pub fn set_bit_in_block(bit_index: u32, block: &mut Vec<u8>) {
    let byte_idx = bit_index / 8;
    let bit = bit_index % 8;
    let mask = 1 << bit;
    block[byte_idx as usize] |= mask;
}
