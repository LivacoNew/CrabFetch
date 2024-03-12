use std::{fmt::Display, fs::File, io::Read};

struct GenericCPU {
    cpu_name: String,
    cores: u16,
    threads: u16,
    max_clock: f32,
    tempreature: f32,
}
impl GenericCPU {
    fn new() -> GenericCPU {
        GenericCPU {
            cpu_name: "".to_string(),
            cores: 0,
            threads: 0,
            max_clock: 0.0,
            tempreature: 0.0
        }
    }
}
impl Display for GenericCPU {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({}c {}t) @ {} [{}]", self.cpu_name, self.cores, self.threads, self.max_clock, self.tempreature)
    }
}

pub fn get_cpu() {
    // Starts by reading and parsing /proc/cpuinfo
    // This gives us the cpu name, cores and threads
    let mut file: File = match File::open("/proc/cpuinfo") {
        Ok(r) => r,
        Err(e) => {
            // Best guess I've got is that we're not on Linux
            // In which case, L
            panic!("Can't read from /proc/cpuinfo - {}", e);
        },
    };
    let mut contents: String = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(e) => {
            panic!("Can't read from /proc/cpuinfo - {}", e);
        },
    }

    // Now we parse
    // Just doing one entry as the rest are kinda redundant
    let mut cpu = GenericCPU::new();

    let entry: &str = contents.split("\n\n").collect::<Vec<&str>>()[0];
    let lines: Vec<&str> = entry.split("\n").collect();
    for line in lines {
        if line.starts_with("model name") {
            cpu.cpu_name = line.split(": ").collect::<Vec<&str>>()[1].to_string();
        }
        if line.starts_with("cpu cores") {
            cpu.cores = match line.split(": ").collect::<Vec<&str>>()[1].parse::<u16>() {
                Ok(r) => r,
                Err(e) => {
                    println!("WARNING: Could not parse cpu cores: {}", e);
                    0
                },
            }
        }
        if line.starts_with("siblings") {
            cpu.threads= match line.split(": ").collect::<Vec<&str>>()[1].parse::<u16>() {
                Ok(r) => r,
                Err(e) => {
                    println!("WARNING: Could not parse cpu threads: {}", e);
                    0
                },
            }
        }
    }


    // Now we parse /sys/devices/system/cpu/cpu0/cpufreq
    // There's 3 possible places to get the frequency in here;
    // - bios_limit - Only present if a limit is set in BIOS
    // - scaling_max_freq - The max freq set by the policy
    // - cpuinfo_max_freq - The max possible the CPU can run at uncapped
    //
    // This just takes the first of those three that are present
    //
    // Source: https://docs.kernel.org/admin-guide/pm/cpufreq.html



    println!("{}", cpu);
}
