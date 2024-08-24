use core::str;

use serde::Deserialize;

use crate::{config_manager::Configuration, formatter::{self, CrabFetchColor}, module::Module, syscalls::SyscallCache, ModuleError};

pub struct SwapInfo {
    used_kb: u64,
    total_kb: u64,
    percent: f32
}
#[derive(Deserialize)]
pub struct SwapConfiguration {
    pub title: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub separator: Option<String>,
    pub progress_left_border: Option<String>,
    pub progress_right_border: Option<String>,
    pub progress_progress: Option<String>,
    pub progress_empty: Option<String>,
    pub progress_target_length: Option<u8>,
    pub decimal_places: Option<u32>,
    pub use_ibis: Option<bool>,
    pub format: String
}
impl Module for SwapInfo {
    fn new() -> SwapInfo {
        SwapInfo {
            used_kb: 0,
            total_kb: 0,
            percent: 0.0
        }
    }

    fn style(&self, config: &Configuration, max_title_size: u64) -> String {
        let title_color: &CrabFetchColor = config.swap.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.swap.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.swap.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.swap.separator.as_ref().unwrap_or(&config.separator);

        let title: String = self.replace_placeholders(&config.swap.title, config);
        let value: String = self.replace_color_placeholders(&self.replace_placeholders(&config.swap.format, config));

        Self::default_style(config, max_title_size, &title, title_color, title_bold, title_italic, separator, &value)
    }
    fn unknown_output(config: &Configuration, max_title_size: u64) -> String { 
        let title_color: &CrabFetchColor = config.swap.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.swap.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.swap.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.swap.separator.as_ref().unwrap_or(&config.separator);

        let title: String = config.uptime.title
            .replace("{used}", "Unknown")
            .replace("{total}", "Unknown")
            .replace("{bar}", "Unknown");

        Self::default_style(config, max_title_size, &title, title_color, title_bold, title_italic, separator, "Unknown")
    }

    fn replace_placeholders(&self, text: &str, config: &Configuration) -> String {
        let dec_places: u32 = config.swap.decimal_places.unwrap_or(config.decimal_places);
        let use_ibis: bool = config.swap.use_ibis.unwrap_or(config.use_ibis);

        let mut bar: String = String::new();
        if text.contains("{bar}") {
            let left_border: &str = config.swap.progress_left_border.as_ref().unwrap_or(&config.progress_left_border);
            let right_border: &str = config.swap.progress_right_border.as_ref().unwrap_or(&config.progress_right_border);
            let progress: &str = config.swap.progress_progress.as_ref().unwrap_or(&config.progress_progress);
            let empty: &str = config.swap.progress_empty.as_ref().unwrap_or(&config.progress_empty);
            let length: u8 = config.swap.progress_target_length.unwrap_or(config.progress_target_length);
            formatter::make_bar(&mut bar, left_border, right_border, progress, empty, self.percent, length);
        }

        formatter::process_percentage_placeholder(text, formatter::round(self.percent as f64, dec_places) as f32, config)
            .replace("{used}", &formatter::auto_format_bytes(self.used_kb, use_ibis, dec_places))
            .replace("{total}", &formatter::auto_format_bytes(self.total_kb, use_ibis, dec_places))
            .replace("{bar}", &bar)
    }

    fn gen_info_flags(_: &str) -> u32 {
        panic!("gen_info_flags called on swap module. This should never happen, please make a bug report!")
    }
}

pub fn get_swap(syscall_cache: &mut SyscallCache) -> Result<SwapInfo, ModuleError> {
    let mut swap: SwapInfo = SwapInfo::new();
    // no info flags here as it's all dependent on eachother

    let sysinfo: libc::sysinfo = syscall_cache.get_sysinfo_cached();

    swap.total_kb = (sysinfo.totalswap * sysinfo.mem_unit as u64) / 1000;
    swap.used_kb = swap.total_kb - ((sysinfo.freeswap * sysinfo.mem_unit as u64) / 1000);

    if swap.total_kb != 0 {
        swap.percent = (swap.used_kb as f32 / swap.total_kb as f32) * 100.0;
    }

    Ok(swap)
}
