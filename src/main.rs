use std::collections::HashMap;

use colored::Colorize;
use sysinfo::{Cpu, CpuRefreshKind, RefreshKind, System};

// I would just use  the sysinfo object, problem is it counts cpus as threads opposed to physical
// CPUs like I want.
struct CPUInfo {
    pub cpu_name: String,
    pub thread_count: u16,
    pub clock_speed: u64
}
impl CPUInfo {
    fn from_sysinfo_cpu(cpu: Cpu, thread_count: u16) -> CPUInfo {
        CPUInfo {
            cpu_name: cpu.brand().to_string(),
            thread_count,
            clock_speed: cpu.frequency()
        }
    }
}

fn main() {
    let mut s = System::new_with_specifics(
        RefreshKind::new().with_cpu(CpuRefreshKind::everything())
    );
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    s.refresh_cpu();

    // CPU
    let mut cpus: HashMap<&str, HashMap<usize, &Cpu>> = HashMap::new(); // Brand -> <Core/CPU> - This all later
    for cpu in s.cpus() {
        cpus.entry(cpu.brand()).or_insert(HashMap::new());
        let map = cpus.get_mut(cpu.brand()).unwrap();
        map.insert(map.len() + 1, cpu);
    }
    // Now we format them into the final vec
    for (brand, hash) in cpus {

    }
}
