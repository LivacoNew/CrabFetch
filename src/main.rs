use crate::{config_manager::Configuration, memory::MemoryInfo, cpu::CPUInfo};

mod cpu;
mod memory;
mod config_manager;

fn main() {
    let config: Configuration = config_manager::parse();
    println!("{}", config.enable_cpu);

    let cpu: CPUInfo = cpu::get_cpu();
    let memory: MemoryInfo = memory::get_memory();
}
