use crate::{config_manager::Configuration, memory::MemoryInfo, cpu::CPUInfo};

mod cpu;
mod memory;
mod config_manager;

fn main() {
    let config: Configuration = config_manager::parse();
    let cpu: CPUInfo = cpu::get_cpu();
    let memory: MemoryInfo = memory::get_memory();

    if config.enable_cpu {
        println!("{}", cpu);
    }
    if config.enable_memory {
        println!("{}", memory);
    }
}
