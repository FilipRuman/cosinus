use log::info;
use thread::Thread;

mod memory;
mod thread;

pub async fn run() {
    info!("Hello from emulator!");
    let thread_0 = Thread { id: 0 };
    tokio::spawn(thread_0.run_loop());
    let thread_1 = Thread { id: 1 };
    tokio::spawn(thread_1.run_loop());
}
