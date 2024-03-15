use crate::{config_manager::Configuration, memory::MemoryInfo, cpu::CPUInfo};

mod cpu;
mod memory;
mod config_manager;
mod ascii;

trait Module {
    fn new() -> Self;
    fn format(&self, format: &str) -> String;
}

fn main() {
    let config: Configuration = config_manager::parse();
    let cpu: CPUInfo = cpu::get_cpu();
    let memory: MemoryInfo = memory::get_memory();

    let ascii = ascii::get_ascii("default");

    let mut line_number: u8 = 0;
    let target_length = 48;
    for line in ascii.split("\n") {
        print!("{}", line);
        let remainder = target_length - line.len();
        for _ in 0..remainder {
            print!(" ");
        }

        // println!("{} {}", config.modules.len(), line_number);
        if config.modules.len() > line_number as usize {
            if config.modules[line_number as usize] == "cpu" {
                let mut str = String::new();
                str.push_str(&config_manager::color_string(&config.cpu_title, &config.title_color).to_string());
                str.push_str(&config.seperator);
                str.push_str(&cpu.format(&config.cpu_format));
                print!("{}", str);
            }
            if config.modules[line_number as usize] == "memory" {
                let mut str = String::new();
                str.push_str(&config_manager::color_string(&config.memory_title, &config.title_color).to_string());
                str.push_str(&config.seperator);
                str.push_str(&memory.format(&config.memory_format));
                print!("{}", str);
            }
        }

        line_number = line_number + 1;
        println!();
    }
}
