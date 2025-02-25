use std::{fs::{self, ReadDir}, mem, net::{IpAddr, Ipv4Addr, Ipv6Addr}};

#[cfg(feature = "jsonschema")]
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{config_manager::Configuration, formatter::CrabFetchColor, module::Module, util, ModuleError};

pub struct LocalIPInfo {
    interface: String,
    ip_addr: String,
}
#[derive(Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(JsonSchema))]
pub struct LocalIPConfiguration {
    pub title: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub separator: Option<String>,
    pub format: String
}
impl Module for LocalIPInfo {
    fn new() -> LocalIPInfo {
        LocalIPInfo {
            interface: "Unknown".to_string(),
            ip_addr: "Unknown".to_string()
        }
    }

    fn style(&self, config: &Configuration) -> (String, String) {
        let title_color: &CrabFetchColor = config.localip.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.localip.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.localip.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.localip.separator.as_ref().unwrap_or(&config.separator);

        let title: String = self.replace_placeholders(&config.localip.title, config);
        let value: String = self.replace_color_placeholders(&self.replace_placeholders(&config.localip.format, config), config);

        Self::default_style(config, &title, title_color, title_bold, title_italic, separator, &value)
    }
    fn unknown_output(config: &Configuration) -> (String, String) {
        let title_color: &CrabFetchColor = config.localip.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.localip.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.localip.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.localip.separator.as_ref().unwrap_or(&config.separator);

        let title: String = config.locale.title
            .replace("{interface}", "Unknown")
            .replace("{addr}", "Unknown");

        Self::default_style(config, &title, title_color, title_bold, title_italic, separator, "Unknown")
    }

    fn replace_placeholders(&self, text: &str, _: &Configuration) -> String {
        text.replace("{interface}", &self.interface)
            .replace("{addr}", &self.ip_addr)
    }

    fn gen_info_flags(_: &str) -> u32 {
        panic!("gen_info_flags called on local IP module. This should never happen, please make a bug report!")
    }
}

#[allow(clippy::cast_ptr_alignment)] // i'm really shitty with dealing with pointers, but afaik its fine 
pub fn get_local_ips() -> Result<Vec<LocalIPInfo>, ModuleError> {
    // no info flags here as it's all from the same source
    let mut addrs: Vec<LocalIPInfo> = Vec::new();

    // First, scan /sys/devices/virtual/net and find any known virtual devices
    // Credit to https://stackoverflow.com/a/52561720 for the initial explanation on how the kernel
    // handles this stuff
    let mut virt_interfaces: Vec<String> = Vec::new();
    let dir: ReadDir = match fs::read_dir("/sys/devices/virtual/net") {
        Ok(r) => r,
        Err(e) => return Err(ModuleError::new("LocalIP", format!("Can't read from /sys/devices/virtual/net: {e}"))),
    };
    for dev in dir {
        let d = match dev {
            Ok(r) => r,
            Err(e) => return Err(ModuleError::new("LocalIP", format!("Failed to open directory: {e}"))),
        };
        virt_interfaces.push(d.file_name().into_string().unwrap());
    }

    // Credit to this guy's example https://users.rust-lang.org/t/best-way-to-get-your-own-ips/14308/2
    // (Adapted a bit) 
    unsafe {
        // Get the first one
        let mut ifaddr: *mut libc::ifaddrs = mem::zeroed();
        if libc::getifaddrs(&mut ifaddr) != 0 {
            return Err(ModuleError::new("LocalIP", "getifaddrs syscall failed!".to_string()));
        }

        let mut ifaddrs: libc::ifaddrs = *ifaddr;
        let mut inf_loop_protection: u8 = 0;
        // Go through every interface
        loop {
            inf_loop_protection += 1;
            let interface_name: String = match util::cstr_from_ptr(ifaddrs.ifa_name) {
                Ok(r) => r,
                Err(e) => return Err(ModuleError::new("LocalIP", format!("Failed to convert interface name into Rust string: {e}")))
            };

            // Ignore any virtual devices
            if virt_interfaces.contains(&interface_name) {
                if ifaddrs.ifa_next.is_null() || inf_loop_protection > 25 {
                    break;
                }
                ifaddrs = *ifaddrs.ifa_next;
                continue;
            }

            // the spooky part, at least for me
            if i32::from((*ifaddrs.ifa_addr).sa_family) == libc::AF_INET {
                // ipv4
                let addr: *mut libc::sockaddr_in = (ifaddrs.ifa_addr).cast::<libc::sockaddr_in>();
                let ipaddr: Ipv4Addr = Ipv4Addr::from((*addr).sin_addr.s_addr.to_be());

                let data: LocalIPInfo = LocalIPInfo {
                    interface: interface_name,
                    ip_addr: IpAddr::V4(ipaddr).to_string()
                };
                addrs.push(data);
            } else if i32::from((*ifaddrs.ifa_addr).sa_family) == libc::AF_INET6 {
                // ipv6
                let addr: *mut libc::sockaddr_in6 = (ifaddrs.ifa_addr).cast::<libc::sockaddr_in6>();
                // https://man7.org/linux/man-pages/man7/ipv6.7.html
                // "Linux supports it only for link-local addresses"
                // I'm guessing that means if it's not 0, it's not a global IP, and thus not a
                // "real" ip and should be discarded? Not too sure tbh
                if (*addr).sin6_scope_id == 0 {
                    let ipaddr: Ipv6Addr = Ipv6Addr::from((*addr).sin6_addr.s6_addr);

                    let mut interface_name: String = interface_name.clone();
                    interface_name.push_str(" (v6)");

                    let data: LocalIPInfo = LocalIPInfo {
                        interface: interface_name,
                        ip_addr: IpAddr::V6(ipaddr).to_string()
                    };
                    addrs.push(data);
                }
            }

            if ifaddrs.ifa_next.is_null() || inf_loop_protection > 25 {
                break;
            }
            ifaddrs = *ifaddrs.ifa_next;
        }
    }

    Ok(addrs)
}
