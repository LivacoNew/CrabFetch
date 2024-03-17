use std::{fmt::Display, fs::File, io::Read, time::Duration};

use crate::Module;

pub struct UptimeInfo {
    uptime_seconds: Duration,
}
impl Module for UptimeInfo {
    fn new() -> UptimeInfo {
        UptimeInfo {
            uptime_seconds: Duration::new(0, 0),
        }
    }
    fn format(&self, format: &str, _: u32) -> String {
        // https://www.reddit.com/r/rust/comments/gju305/comment/fqo9zbb/?utm_source=share&utm_medium=web3x&utm_name=web3xcss&utm_term=1&utm_content=share_button
        let seconds = self.uptime_seconds.as_secs() % 60;
        let minutes = (self.uptime_seconds.as_secs() / 60) % 60;
        let hours = (self.uptime_seconds.as_secs() / 60) / 60;

        format.replace("{seconds}", &seconds.to_string())
        .replace("{minutes}", &minutes.to_string())
        .replace("{hours}", &hours.to_string())
    }
}
impl Display for UptimeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} seconds", self.uptime_seconds.as_secs())
    }
}

pub fn get_uptime() -> UptimeInfo {
    let mut uptime = UptimeInfo::new();
    get_basic_info(&mut uptime);

    uptime
}

fn get_basic_info(uptime: &mut UptimeInfo) {
    // Grabs from /proc/uptime
    let mut file: File = match File::open("/proc/uptime") {
        Ok(r) => r,
        Err(e) => {
            panic!("Can't read from /proc/uptime - {}", e);
        },
    };
    let mut contents: String = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(e) => {
            panic!("Can't read from /proc/uptime - {}", e);
        },
    }
    uptime.uptime_seconds = match contents.split(" ").collect::<Vec<&str>>()[0].parse::<f64>() {
        Ok(r) => Duration::new(r.floor() as u64, 0),
        Err(e) => {
            println!("WARNING: Could not parse /proc/uptime: {}", e);
            Duration::new(0, 0)
        },
    };
}