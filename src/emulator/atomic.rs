use std::{cell::UnsafeCell, collections::HashMap, sync::LazyLock};

use crate::emulator::thread::Thread;

pub static ATOMIC_LOCKS: LazyLock<AtomicLocks> = LazyLock::new(|| AtomicLocks::new());

#[derive(Clone, Copy)]
pub struct Lock {
    pub valid: bool,
    pub addr: i32,
}

pub struct AtomicLocks {
    locks: UnsafeCell<HashMap<i32, Lock>>,
}

// Required because UnsafeCell removes Sync by default
unsafe impl Sync for AtomicLocks {}

impl AtomicLocks {
    pub fn new() -> Self {
        Self {
            locks: UnsafeCell::new(HashMap::new()),
        }
    }

    #[inline]
    unsafe fn get_mut(&self) -> &mut HashMap<i32, Lock> {
        unsafe { &mut *self.locks.get() }
    }

    // --- LR ---
    pub fn lr(&self, tid: i32, addr: i32) {
        unsafe {
            let locks = self.get_mut();
            locks.insert(tid, Lock { valid: true, addr });
        }
    }

    // --- SC ---
    pub fn sc(&self, tid: i32, addr: i32) -> bool {
        unsafe {
            let locks = self.get_mut();

            let success = matches!(
                locks.get(&tid),
                Some(lock) if lock.valid && lock.addr == addr
            );

            if success {
                // Invalidate all reservations (typical LR/SC behavior)
                for (_, lock) in locks.iter_mut() {
                    if lock.addr == addr {
                        lock.valid = false;
                    }
                }
            }

            success
        }
    }

    pub fn invalidate_addr(&self, addr: i32) {
        unsafe {
            let locks = self.get_mut();
            for (_, l) in locks.iter_mut() {
                if l.addr == addr {
                    l.valid = false;
                }
            }
        }
    }
}
impl Thread {
    pub fn lr(&mut self, rd: u8, rs1: u8) {
        self.load(rd, rs1, 0);
        ATOMIC_LOCKS.lr(self.tid, self.reg(rs1));
    }
    pub fn sc(&mut self, rd: u8, rs1: u8, rs2: u8) {
        let success = ATOMIC_LOCKS.sc(self.tid, self.reg(rs1));
        self.gpr[rd as usize] = if success { 1 } else { 0 };
        if success {
            self.store(rs1, rs2, 0);
        }
    }
}
