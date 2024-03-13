use cpu::CPUInfo;
use memory::MemoryInfo;

mod cpu;
mod memory;

fn main() {
    let cpu: CPUInfo = cpu::get_cpu();
    let memory: MemoryInfo = memory::get_memory();
}
