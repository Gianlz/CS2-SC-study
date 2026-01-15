pub mod entity;
mod dumper;
mod find_offsets;
mod offsets;
mod schema;
mod skin_changer;

use crate::{
    config::SkinChangerConfig,
    constants::cs2,
    cs2::offsets::Offsets,
    os::process::Process,
};

#[derive(Debug)]
pub struct CS2 {
    is_valid: bool,
    pub(crate) process: Process,
    pub(crate) offsets: Offsets,
}

impl CS2 {
    pub fn new() -> Self {
        Self {
            is_valid: false,
            process: Process::new(-1),
            offsets: Offsets::default(),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.is_valid && self.process.is_valid()
    }

    pub fn setup(&mut self) {
        let Some(process) = Process::open(cs2::PROCESS_NAME) else {
            self.is_valid = false;
            return;
        };
        log::info!("process found, pid: {}", process.pid);
        self.process = process;

        self.offsets = match self.find_offsets() {
            Some(offsets) => offsets,
            None => {
                self.process = Process::new(-1);
                self.is_valid = false;
                return;
            }
        };
        log::info!("offsets found");

        self.is_valid = true;
    }

    pub fn run(&mut self, config: &SkinChangerConfig) {
        if !self.process.is_valid() {
            self.is_valid = false;
            log::debug!("process is no longer valid");
            return;
        }

        self.skin_changer(config);
    }
}
