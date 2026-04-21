use crate::memory::Memory;
use log::info;
use std::{sync::LazyLock, time::Duration};
use tokio::time::sleep;
static MEMORY: LazyLock<Memory> = LazyLock::new(|| Memory::new());

pub struct Thread {
    pub id: u8,
}

impl Thread {
    pub async fn run_loop(self) {
        println!("RUN");
        loop {
            let addr = (self.id * 4) as usize;
            let val = unsafe { MEMORY.read(addr) };
            info!("Thread with id:'{}' -> '{val}'", self.id);
            unsafe { MEMORY.write(addr, self.id.into()) };
            sleep(Duration::from_secs(1)).await;
        }
    }
}
