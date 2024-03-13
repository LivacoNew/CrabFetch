use cpu::GenericCPU;

mod cpu;

fn main() {
    // CPU
    let cpu: GenericCPU = cpu::get_cpu();
}
