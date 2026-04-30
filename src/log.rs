use std::sync::atomic::AtomicBool;

use log::LevelFilter;

const LEVEL_FILTER: LevelFilter = LevelFilter::Info;
static LOG_INITALIZED: AtomicBool = AtomicBool::new(false);
pub fn init_log() {
    if !LOG_INITALIZED.load(std::sync::atomic::Ordering::Relaxed) {
        LOG_INITALIZED.store(true, std::sync::atomic::Ordering::Relaxed);
        let mut builder = colog::default_builder();
        builder.filter_level(LEVEL_FILTER);
        builder.init();
    }
}
