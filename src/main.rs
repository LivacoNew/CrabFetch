use cpu::CPUInfo;

mod cpu;

fn main() {
    // CPU
    let cpu: CPUInfo = cpu::get_cpu();
}
