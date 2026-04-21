use log::info;

#[tokio::main]
async fn main() {
    colog::init();
    emulator::run().await;
    loop {}
}
