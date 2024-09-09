use core::str;
use std::{fs::{read_dir, File, ReadDir}, io::{BufRead, BufReader, Read}, path::{Component, Path}};

#[cfg(feature = "android")]
use {android_system_properties::AndroidSystemProperties, std::env};
#[cfg(target_arch = "x86_64")]
use raw_cpuid::CpuId;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{config_manager::Configuration, formatter::{self, CrabFetchColor}, module::Module, util::{self, is_flag_set_u32}, ModuleError};

pub struct CPUInfo {
    name: String,
    cores: u16,
    threads: u16,
    current_clock_mhz: f32,
    max_clock_mhz: f32,
    arch: String
}
#[derive(Deserialize, JsonSchema)]
pub struct CPUConfiguration {
    pub title: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub separator: Option<String>,
    pub format: String,
    pub decimal_places: Option<u32>,
    pub remove_trailing_processor: bool
}

impl Module for CPUInfo {
    fn new() -> CPUInfo {
        CPUInfo {
            name: "Unknown".to_string(),
            cores: 0,
            threads: 0,
            current_clock_mhz: 0.0,
            max_clock_mhz: 0.0,
            arch: "Unknown".to_string()
        }
    }

    fn style(&self, config: &Configuration) -> (String, String) {
        let title_color: &CrabFetchColor = config.cpu.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.cpu.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.cpu.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.cpu.separator.as_ref().unwrap_or(&config.separator);

        let title: String = self.replace_placeholders(&config.cpu.title, config);
        let value: String = self.replace_color_placeholders(&self.replace_placeholders(&config.cpu.format, config), config);

        Self::default_style(config, &title, title_color, title_bold, title_italic, separator, &value)
    }
    fn unknown_output(config: &Configuration) -> (String, String) {
        let title_color: &CrabFetchColor = config.cpu.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.cpu.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.cpu.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.cpu.separator.as_ref().unwrap_or(&config.separator);

        let title: String = config.cpu.title
            .replace("{name}", "Unknown")
            .replace("{core_count}", "Unknown")
            .replace("{thread_count}", "Unknown")
            .replace("{current_clock_mhz}", "Unknown")
            .replace("{current_clock_ghz}", "Unknown")
            .replace("{max_clock_mhz}", "Unknown")
            .replace("{max_clock_ghz}", "Unknown")
            .replace("{arch}", "Unknown");
        
        Self::default_style(config, &title, title_color, title_bold, title_italic, separator, "Unknown")
    }

    fn replace_placeholders(&self, text: &str, config: &Configuration) -> String {
        let dec_places: u32 = config.cpu.decimal_places.unwrap_or(config.decimal_places);

        text.replace("{name}", &self.name)
            .replace("{core_count}", &self.cores.to_string())
            .replace("{thread_count}", &self.threads.to_string())
            .replace("{current_clock_mhz}", &formatter::round(f64::from(self.current_clock_mhz), dec_places).to_string())
            .replace("{current_clock_ghz}", &formatter::round(f64::from(self.current_clock_mhz / 1000.0), dec_places).to_string())
            .replace("{max_clock_mhz}", &formatter::round(f64::from(self.max_clock_mhz), dec_places).to_string())
            .replace("{max_clock_ghz}", &formatter::round(f64::from(self.max_clock_mhz / 1000.0), dec_places).to_string())
            .replace("{arch}", &self.arch.to_string())
    }

    fn gen_info_flags(format: &str) -> u32 {
        // Figure out the info we need to fetch
        let mut info_flags: u32 = 0;

        if format.contains("{name}") {
            info_flags |= CPU_INFOFLAG_MODEL_NAME;
        }
        if format.contains("{core_count}") {
            info_flags |= CPU_INFOFLAG_CORES;
        }
        if format.contains("{thread_count}") {
            info_flags |= CPU_INFOFLAG_THREADS;
        }
        if format.contains("{current_clock_mhz}") || format.contains("{current_clock_ghz}") {
            info_flags |= CPU_INFOFLAG_CURRENT_CLOCK;
        }
        if format.contains("{max_clock_mhz}") || format.contains("{max_clock_ghz}") {
            info_flags |= CPU_INFOFLAG_MAX_CLOCK;
        }
        if format.contains("{arch}") || format.contains("{arch}") {
            info_flags |= CPU_INFOFLAG_ARCH;
        }

        info_flags
    }
}

const CPU_INFOFLAG_MODEL_NAME: u32 = 1;
const CPU_INFOFLAG_CORES: u32 = 2;
const CPU_INFOFLAG_THREADS: u32 = 4;
const CPU_INFOFLAG_CURRENT_CLOCK: u32 = 8;
const CPU_INFOFLAG_MAX_CLOCK: u32 = 16;
const CPU_INFOFLAG_ARCH: u32 = 32;

pub fn get_cpu(config: &Configuration) -> Result<CPUInfo, ModuleError> {
    let mut cpu: CPUInfo = CPUInfo::new();
    let info_flags: u32 = CPUInfo::gen_info_flags(&config.cpu.format);

    // This ones split into 2 as theres a lot to parse
    match get_basic_info(&mut cpu, info_flags) {
        Ok(_) => {},
        Err(e) => return Err(e)
    };
    match get_max_clock(&mut cpu, info_flags) {
        Ok(_) => {},
        Err(e) => return Err(e)
    };

    if config.cpu.remove_trailing_processor {
        // Tried doing this with Regex but it added 400 micro secs so fuck that shit
        let loc: usize = match cpu.name.find("-Core Processor") {
            Some(r) => r,
            None => return Ok(cpu), // ignore it
        };

        // Find the last space
        let search: &str = &cpu.name[..loc];
        let mut space_index: usize = 0;
        for (i, x) in search.chars().enumerate() {
            if x == ' ' && space_index < i {
                space_index = i;
            }
        }

        let replace_me: &str = &cpu.name[space_index..];
        cpu.name = cpu.name.replace(replace_me, "");
    }

    Ok(cpu)
}

fn get_basic_info(cpu: &mut CPUInfo, info_flags: u32) -> Result<(), ModuleError> {
    // Starts by reading and parsing /proc/cpuinfo
    // This gives us the cpu name, cores, threads and current clock
    let file: File = match File::open("/proc/cpuinfo") {
        Ok(r) => r,
        Err(e) => return Err(ModuleError::new("CPU", format!("Can't read from /proc/cpuinfo - {e}"))),
    };

    let buffer: BufReader<File> = BufReader::new(file);
    let mut cpu_mhz_count: u8 = 0;
    let mut first_entry: bool = true;
    let mut cores: u16 = 0; // This acts as a backup for the "cpu cores" being missing
    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    let mut arm_vendor: String = String::new();
    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    let mut arm_part: String = String::new();

    for line in buffer.lines() {
        if line.is_err() {
            continue;
        }
        let line = line.unwrap();
        if line.is_empty() {
            first_entry = false;
            cores += 1;
        }

        if first_entry {
            if line.starts_with("model name") && is_flag_set_u32(info_flags, CPU_INFOFLAG_MODEL_NAME) {
                cpu.name = line.split(": ").collect::<Vec<&str>>()[1].to_string();
            }
            #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
            if line.starts_with("CPU part") && is_flag_set_u32(info_flags, CPU_INFOFLAG_MODEL_NAME) {
                arm_part = line.split(": ").collect::<Vec<&str>>()[1].to_string();
            }
            #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
            if line.starts_with("CPU implementer") && is_flag_set_u32(info_flags, CPU_INFOFLAG_MODEL_NAME) {
                arm_vendor = line.split(": ").collect::<Vec<&str>>()[1].to_string();
            }
            if line.starts_with("cpu cores") && is_flag_set_u32(info_flags, CPU_INFOFLAG_CORES) {
                cpu.cores = match line.split(": ").collect::<Vec<&str>>()[1].parse::<u16>() {
                    Ok(r) => r,
                    Err(e) => return Err(ModuleError::new("CPU", format!("WARNING: Could not parse cpu cores: {e}"))),
                }
            }
            if line.starts_with("siblings") && is_flag_set_u32(info_flags, CPU_INFOFLAG_THREADS) {
                cpu.threads = match line.split(": ").collect::<Vec<&str>>()[1].parse::<u16>() {
                    Ok(r) => r,
                    Err(e) => return Err(ModuleError::new("CPU", format!("WARNING: Could not parse cpu threads: {e}"))),
                }
            }
            if line.starts_with("flags") && is_flag_set_u32(info_flags, CPU_INFOFLAG_ARCH) {
                // https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/tree/arch/x86/include/asm/cpufeatures.h
                for flag in line.split(": ").collect::<Vec<&str>>()[1].split(' ') {
                    // prepare for trouble
                    cpu.arch = match flag {
                        "ia86" => {"IA86".to_string()}
                        "arch_capabilities" => {"IA32".to_string()}
                        "lm" => {"x86_64".to_string()}
                        _ => {continue} 
                    };
                    if !cpu.arch.is_empty() {
                        break;
                    }
                }
            }
        }
        // This ignore's it's feature flag to prevent issues allow max freq to back up to this on
        // failure
        if line.starts_with("cpu MHz") {
            cpu.current_clock_mhz += match line.split(": ").collect::<Vec<&str>>()[1].parse::<f32>() {
                Ok(r) => r,
                Err(e) => return Err(ModuleError::new("CPU", format!("WARNING: Could not parse current cpu frequency: {e}"))),
            };
            cpu_mhz_count += 1;
        }
    }
    if cpu.cores == 0 && is_flag_set_u32(info_flags, CPU_INFOFLAG_CORES) {
        cpu.cores = cores;
        // Backup to /sys/devices/system/cpu/present for threads too
        // Thanks to https://stackoverflow.com/a/30150409
        let mut file: File = match File::open("/sys/devices/system/cpu/present") {
            Ok(r) => r,
            Err(e) => return Err(ModuleError::new("CPU", format!("Can't read from /sys/devices/system/cpu/present - {e}"))),
        };
        let mut contents: String = String::new();
        match file.read_to_string(&mut contents) {
            Ok(_) => {},
            Err(e) => return Err(ModuleError::new("CPU", format!("Can't read from /sys/devices/system/cpu/present - {e}"))),
        }
        cpu.threads = match contents.trim().split('-').last().unwrap().parse::<u16>() {
            Ok(r) => r + 1,
            Err(e) => return Err(ModuleError::new("CPU", format!("Failed to parse thread count from /sys/devices/system/cpu/present - {e}"))),
        };
    }

    // Android 
    #[cfg(feature = "android")]
    if env::consts::OS == "android" {
        // This property was a fucking nightmare to find, only being able to find out about it here https://github.com/ArrowOS-Devices/android_device_xiaomi_daisy/blob/arrow-12.1/vendor.prop#L185
        // I have no idea how standard that property is, especially since I couldn't find it after
        // a couple hours of scowering Android's docs but oh well
        let props = AndroidSystemProperties::new();
        // https://github.com/termux/termux-api/issues/448#issuecomment-927345222
        if let Some(soc_manu) = props.get("ro.soc.manufacturer") {
            // chaining these let statements is only in the unstable branch so egypt it is
            if let Some(soc_model) = props.get("ro.soc.model") {
                cpu.name = format!("{} {}", soc_manu, soc_model);
            }
        }
    }

    #[cfg(target_arch = "x86_64")]
    if cpu.name == "Unknown" && is_flag_set_u32(info_flags, CPU_INFOFLAG_MODEL_NAME) {
        // This **seems** to be the only thing that can be missing, mostly on ARM
        // So, we'll use cpuid to grab it
        // Credit to Emma lol https://mastodon.social/@Livaco/113027370696819081
        backup_to_cpuid(cpu);
    }

    // Handle ARM by looking up the ID
    // Credit to https://github.com/util-linux/util-linux/blob/d1917754c633f8eed141ff1b1dbde6bcfe1f8098/sys-utils/lscpu-arm.c#L402
    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    if is_flag_set_u32(info_flags, CPU_INFOFLAG_MODEL_NAME) {
        let key: String = format!("{}-{}", arm_vendor, arm_part);
        for x in ARM_LOOKUP {
            if x.0 != key {
                continue
            }
            cpu.name = x.1.to_string();
        }

        // We can also safely assume we're running ARM
        cpu.arch = "AArch64".to_string();
    }

    cpu.current_clock_mhz /= f32::from(cpu_mhz_count);
    Ok(())
}
fn get_max_clock(cpu: &mut CPUInfo, info_flags: u32) -> Result<(), ModuleError> {
    if !is_flag_set_u32(info_flags, CPU_INFOFLAG_MAX_CLOCK) {
        return Ok(())
    }
    // All of this is relative to /sys/devices/system/cpu/X/cpufreq
    // There's 3 possible places to get the frequency in here;
    // - bios_limit - Only present if a limit is set in BIOS
    // - scaling_max_freq - The max freq set by the policy
    // - cpuinfo_max_freq - The max possible the CPU can run at uncapped
    //
    // This just takes the first of those three that are present
    //
    // Source: https://docs.kernel.org/admin-guide/pm/cpufreq.html

    let freq_path: Option<&Path> = util::find_first_path_exists(vec![
        Path::new("/sys/devices/system/cpu/cpu0/cpufreq/bios_limit"),
        Path::new("/sys/devices/system/cpu/cpu0/cpufreq/scaling_max_freq"),
        Path::new("/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq")
    ]);

    if freq_path.is_none() {
        // Back up to the repoted value in /proc/cpuinfo
        // At this point all I can assume is your in a VM, which doesn't have any of the above
        // paths and seems to keep a steady CPU frequency in here instead
        // Not the most elegant thing but I can't seem to find anything else to do
        cpu.max_clock_mhz = cpu.current_clock_mhz;
        return Ok(())
    }
    let freq_path: &Path = freq_path.unwrap();
    let mut freq_path_str: String = String::new();
    // sheesh
    for comp in freq_path.components().rev().take(2).collect::<Vec<Component<>>>().iter().rev() {
        freq_path_str.push('/');
        freq_path_str.push_str(comp.as_os_str().to_str().unwrap());
    }
    let freq_path: &str = &freq_path_str[1..];
    
    let dir: ReadDir = match read_dir("/sys/devices/system/cpu/") {
        Ok(r) => r,
        Err(e) => return Err(ModuleError::new("CPU", format!("Can't read from /sys/devices/system/cpu - {e}")))
    };
    for entry in dir {
        let freq_path = match entry {
            Ok(r) => {
                let file_name = r.file_name();
                let file_name = file_name.to_str().unwrap();
                if !file_name.starts_with("cpu") || file_name == "cpuidle" || file_name.starts_with("cpufreq") {
                    continue
                }
                r.path().join(freq_path)
            },
            Err(_) => continue, // ?
        };
        let freq_path = freq_path.as_path();

        match util::file_read(freq_path) {
            Ok(r) => {
                match r.trim().parse::<f32>() {
                    Ok(r) => cpu.max_clock_mhz = f32::max(r / 1000.0, cpu.max_clock_mhz),
                    Err(e) => return Err(ModuleError::new("CPU", format!("Unable to parse f32 from {} - {e}", freq_path.to_str().unwrap())))
                };
            },
            Err(e) => return Err(ModuleError::new("CPU", format!("Can't read from {} - {e}", freq_path.to_str().unwrap()))),
        };
    }

    Ok(())
}

#[cfg(target_arch = "x86_64")]
fn backup_to_cpuid(cpu: &mut CPUInfo) {
    let cpuid = CpuId::new();
    if let Some(model) = cpuid.get_processor_brand_string() {
        cpu.name = model.as_str().to_string();
    }
}



// ARM CPU part IDs
// Credit to
// https://github.com/util-linux/util-linux/blob/d1917754c633f8eed141ff1b1dbde6bcfe1f8098/sys-utils/lscpu-arm.c#L25
// https://github.com/util-linux/util-linux/blob/d1917754c633f8eed141ff1b1dbde6bcfe1f8098/sys-utils/lscpu-arm.c#L296
// for the full list
//
// Index by {vendor}-{part} and you should get your name :)
//
// I love how Rust can't tell the array length itself at compile time... when it could just count
// the elements lmfao
#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
const ARM_LOOKUP: &[(&str, &str)] = &[
    // ARM 
    ("0x41-0x810", "ARM810"),
    ("0x41-0x920", "ARM920"),
    ("0x41-0x922", "ARM922"),
    ("0x41-0x926", "ARM926"),
    ("0x41-0x940", "ARM940"),
    ("0x41-0x946", "ARM946"),
    ("0x41-0x966", "ARM966"),
    ("0x41-0xa20", "ARM1020"),
    ("0x41-0xa22", "ARM1022"),
    ("0x41-0xa26", "ARM1026"),
    ("0x41-0xb02", "ARM11 MPCore"),
    ("0x41-0xb36", "ARM1136"),
    ("0x41-0xb56", "ARM1156"),
    ("0x41-0xb76", "ARM1176"),
    ("0x41-0xc05", "ARM Cortex-A5"),
    ("0x41-0xc07", "ARM Cortex-A7"),
    ("0x41-0xc08", "ARM Cortex-A8"),
    ("0x41-0xc09", "ARM Cortex-A9"),
    // "Originally A12"
    ("0x41-0xc0d", "ARM Cortex-A17"),
    ("0x41-0xc0f", "ARM Cortex-A15"),
    ("0x41-0xc0e", "ARM Cortex-A17"),
    ("0x41-0xc14", "ARM Cortex-R4"),
    ("0x41-0xc15", "ARM Cortex-R5"),
    ("0x41-0xc17", "ARM Cortex-R7"),
    ("0x41-0xc18", "ARM Cortex-R8"),
    ("0x41-0xc20", "ARM Cortex-M0"),
    ("0x41-0xc21", "ARM Cortex-M1"),
    ("0x41-0xc23", "ARM Cortex-M3"),
    ("0x41-0xc24", "ARM Cortex-M4"),
    ("0x41-0xc27", "ARM Cortex-M7"),
    ("0x41-0xc60", "ARM Cortex-M0+"),
    ("0x41-0xd01", "ARM Cortex-A32"),
    ("0x41-0xd02", "ARM Cortex-A34"),
    ("0x41-0xd03", "ARM Cortex-A53"),
    ("0x41-0xd04", "ARM Cortex-A35"),
    ("0x41-0xd05", "ARM Cortex-A55"),
    ("0x41-0xd06", "ARM Cortex-A65"),
    ("0x41-0xd07", "ARM Cortex-A57"),
    ("0x41-0xd08", "ARM Cortex-A72"),
    ("0x41-0xd09", "ARM Cortex-A73"),
    ("0x41-0xd0a", "ARM Cortex-A75"),
    ("0x41-0xd0b", "ARM Cortex-A76"),
    ("0x41-0xd0c", "ARM Neoverse-N1"),
    ("0x41-0xd0d", "ARM Cortex-A77"),
    ("0x41-0xd0e", "ARM Cortex-A76AE"),
    ("0x41-0xd13", "ARM Cortex-R52"),
    ("0x41-0xd15", "ARM Cortex-R82"),
    ("0x41-0xd16", "ARM Cortex-R52+"),
    ("0x41-0xd20", "ARM Cortex-M23"),
    ("0x41-0xd21", "ARM Cortex-M33"),
    ("0x41-0xd22", "ARM Cortex-M55"),
    ("0x41-0xd23", "ARM Cortex-M85"),
    ("0x41-0xd40", "ARM Neoverse-V1"),
    ("0x41-0xd41", "ARM Cortex-A78"),
    ("0x41-0xd42", "ARM Cortex-A78AE"),
    ("0x41-0xd43", "ARM Cortex-A65AE"),
    ("0x41-0xd44", "ARM Cortex-X1"),
    ("0x41-0xd46", "ARM Cortex-A510"),
    ("0x41-0xd47", "ARM Cortex-A710"),
    ("0x41-0xd48", "ARM Cortex-X2"),
    ("0x41-0xd49", "ARM Neoverse-N2"),
    ("0x41-0xd4a", "ARM Neoverse-E1"),
    ("0x41-0xd4b", "ARM Cortex-A78C"),
    ("0x41-0xd4c", "ARM Cortex-X1C"),
    ("0x41-0xd4d", "ARM Cortex-A715"),
    ("0x41-0xd4e", "ARM Cortex-X3"),
    ("0x41-0xd4f", "ARM Neoverse-V2"),
    ("0x41-0xd80", "ARM Cortex-A520"),
    ("0x41-0xd81", "ARM Cortex-A720"),
    ("0x41-0xd82", "ARM Cortex-X4"),
    ("0x41-0xd84", "ARM Neoverse-V3"),
    ("0x41-0xd85", "ARM Cortex-X925"),
    ("0x41-0xd87", "ARM Cortex-A725"),
    ("0x41-0xd8e", "ARM Neoverse-N3"),

    // Broadcom
    ("0x42-0x0f", "Broadcom Brahma-B15"),
    ("0x42-0x100", "Broadcom Brahma-B53"),
    ("0x42-0x0516", "Broadcom ThunderX2"),

    // Cavium
    ("0x43-0x0a0", "Cavium ThunderX"),
    ("0x43-0x0a1", "Cavium ThunderX-88XX"),
    ("0x43-0x0a2", "Cavium ThunderX-81XX"),
    ("0x43-0x0a3", "Cavium ThunderX-83XX"),
    ("0x43-0x0af", "Cavium ThunderX2-99xx"),
    ("0x43-0x0b0", "Cavium OcteonTX2"),
    ("0x43-0x0b1", "Cavium OcteonTX2-98XX"),
    ("0x43-0x0b2", "Cavium OcteonTX2-96XX"),
    ("0x43-0x0b3", "Cavium OcteonTX2-95XX"),
    ("0x43-0x0b4", "Cavium OcteonTX2-95XXN"),
    ("0x43-0x0b5", "Cavium OcteonTX2-95XXMM"),
    ("0x43-0x0b6", "Cavium OcteonTX2-95XXO"),
    ("0x43-0x0b8", "Cavium ThunderX3-T110"),

    // DEC
    ("0x44-0xa10", "DEC SA110"),
    ("0x44-0xa11", "DEC SA1100"),

    // APM
    ("0x50-0x000", "APM X-Gene"),

    // Qualcomm
    ("0x51-0x001", "Qualcomm Oryon"),
    ("0x51-0x00f", "Qualcomm Scorpion"),
    ("0x51-0x02d", "Qualcomm Scorpion"),
    ("0x51-0x04d", "Qualcomm Krait"),
    ("0x51-0x06f", "Qualcomm Krait"),
    ("0x51-0x201", "Qualcomm Kryo"),
    ("0x51-0x205", "Qualcomm Kryo"),
    ("0x51-0x211", "Qualcomm Kryo"),
    ("0x51-0x800", "Qualcomm Falkor-V1/Kryo"),
    ("0x51-0x801", "Qualcomm Kryo-V2"),
    ("0x51-0x802", "Qualcomm Kryo-3XX-Gold"),
    ("0x51-0x803", "Qualcomm Kryo-3XX-Silver"),
    ("0x51-0x804", "Qualcomm Kryo-4XX-Gold"),
    ("0x51-0x805", "Qualcomm Kryo-4XX-Silver"),
    ("0x51-0xc00", "Qualcomm Falkor"),
    ("0x51-0xc01", "Qualcomm Saphira"),

    // Samsung
    ("0x53-0x001", "Samsung Exynos-M1"),
    ("0x53-0x002", "Samsung Exynos-M3"),
    ("0x53-0x003", "Samsung Exynos-M4"),
    ("0x53-0x004", "Samsung Exynos-M5"),

    // NVIDIA
    ("0x4e-0x000", "NVIDIA Denver"),
    ("0x4e-0x003", "NVIDIA Denver 2"),
    ("0x4e-0x004", "NVIDIA Carmel"),

    // Marvell
    ("0x56-0x131", "Marvell Feroceon-88FR131"),
    ("0x56-0x581", "Marvell PJ4/PJ4b"),
    ("0x56-0x584", "Marvell PJ4B-MP"),

    // Apple (yuck)
    ("0x61-0x000", "Apple Swift"),
    ("0x61-0x001", "Apple Cyclone"),
    ("0x61-0x002", "Apple Typhoon"),
    ("0x61-0x003", "Apple Typhoon/Capri"),
    ("0x61-0x004", "Apple Twister"),
    ("0x61-0x005", "Apple Twister/Elba/Malta"),
    ("0x61-0x006", "Apple Hurricane"),
    ("0x61-0x007", "Apple Hurricane/Myst"),
    ("0x61-0x008", "Apple Monsoon"),
    ("0x61-0x009", "Apple Mistral"),
    ("0x61-0x00b", "Apple Vortex"),
    ("0x61-0x00c", "Apple Tempest"),
    ("0x61-0x00f", "Apple Tempest-M9"),
    ("0x61-0x010", "Apple Vortex/Aruba"),
    ("0x61-0x011", "Apple Tempest/Aruba"),
    ("0x61-0x012", "Apple Lightning"),
    ("0x61-0x013", "Apple Thunder"),
    ("0x61-0x020", "Apple Icestorm-A14"),
    ("0x61-0x021", "Apple Firestorm-A14"),
    ("0x61-0x022", "Apple Icestorm-M1"),
    ("0x61-0x023", "Apple Firestorm-M1"),
    ("0x61-0x024", "Apple Icestorm-M1-Pro"),
    ("0x61-0x025", "Apple Firestorm-M1-Pro"),
    ("0x61-0x026", "Apple Thunder-M10"),
    ("0x61-0x028", "Apple Icestorm-M1-Max"),
    ("0x61-0x029", "Apple Firestorm-M1-Max"),
    ("0x61-0x030", "Apple Blizzard-A15"),
    ("0x61-0x031", "Apple Avalanche-A15"),
    ("0x61-0x032", "Apple Blizzard-M2"),
    ("0x61-0x033", "Apple Avalanche-M2"),
    ("0x61-0x034", "Apple Blizzard-M2-Pro"),
    ("0x61-0x035", "Apple Avalanche-M2-Pro"),
    ("0x61-0x036", "Apple Sawtooth-A16"),
    ("0x61-0x037", "Apple Everest-A16"),
    ("0x61-0x038", "Apple Blizzard-M2-Max"),
    ("0x61-0x039", "Apple Avalanche-M2-Max"),

    // Faraday
    ("0x66-0x526", "Faraday FA526"),
    ("0x66-0x626", "Faraday FA626"),

    // Intel
    ("0x69-0x200", "Intel i80200"),
    ("0x69-0x210", "Intel PXA250A"),
    ("0x69-0x212", "Intel PXA210A"),
    ("0x69-0x242", "Intel i80321-400"),
    ("0x69-0x243", "Intel i80321-600"),
    ("0x69-0x290", "Intel PXA250B/PXA26x"),
    ("0x69-0x292", "Intel PXA210B"),
    ("0x69-0x2c2", "Intel i80321-400-B0"),
    ("0x69-0x2c3", "Intel i80321-600-B0"),
    ("0x69-0x2d0", "Intel PXA250C/PXA255/PXA26x"),
    ("0x69-0x2d2", "Intel PXA210C"),
    ("0x69-0x411", "Intel PXA27x"),
    ("0x69-0x41c", "Intel IPX425-533"),
    ("0x69-0x41d", "Intel Intel IPX425-400"),
    ("0x69-0x41f", "Intel IPX425-266"),
    ("0x69-0x682", "Intel PXA32x"),
    ("0x69-0x683", "Intel PXA930/PXA935"),
    ("0x69-0x688", "Intel PXA30x"),
    ("0x69-0x689", "Intel PXA31x"),
    ("0x69-0xb11", "Intel SA1110"),
    ("0x69-0xc12", "Intel IPX1200"),

    // Fujitsu
    ("0x46-0x001", "FUJITSU A64FX"),

    // HISI
    ("0x48-0xd01", "HiSilicon TaiShan-v110"),
    ("0x48-0xd02", "HiSilicon TaiShan-v120"),
    // TODO: Should this be credited to hisilicon or arm??
    ("0x48-0xd40", "HiSilicon Cortex-A76"),
    ("0x48-0xd41", "HiSilicon Cortex-A77"),

    // Ampere
    ("0xc0-0xac3", "Ampere-1"),
    ("0xc0-0xac4", "Ampere-1a"),

    // Phytium
    ("0x70-0x303", "Phytilum FTC310"),
    ("0x70-0x660", "Phytilum FTC660"),
    ("0x70-0x661", "Phytilum FTC661"),
    ("0x70-0x662", "Phytilum FTC662"),
    ("0x70-0x663", "Phytilum FTC663"),
    ("0x70-0x664", "Phytilum FTC664"),
    ("0x70-0x862", "Phytilum FTC862"),

    // Microsoft
    ("0x6d-0xd49", "Microsoft Azure-Cobalt-100")
];
