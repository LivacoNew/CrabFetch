use core::str;
use std::{fmt::Display, fs::File, io::Read, os::unix::process};

use crate::Module;

pub struct TerminalInfo {
    terminal_name: String
}
impl Module for TerminalInfo {
    fn new() -> TerminalInfo {
        TerminalInfo {
            terminal_name: "Unknown".to_string(),
        }
    }
    fn format(&self, format: &str, _: u32) -> String {
        format.replace("{terminal_name}", &self.terminal_name)
    }
}
impl Display for TerminalInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.terminal_name)
    }
}

pub fn get_terminal() -> TerminalInfo {
    let mut terminal: TerminalInfo = TerminalInfo::new();

    // This is just the rust-ified solution from https://askubuntu.com/a/508047
    // Not sure how well it works in all terminals, but it works fine for my tests in Kitty and
    // Allacritty, so idm.
    //
    // basename "/"$(ps -o cmd -f -p $(cat /proc/$(echo $$)/stat | cut -d \  -f 4) | tail -1 | sed 's/ .*$//')
    // Essentially, it grabs the parent process, grabs the parent of that pid from /proc/x/stat and
    // gets the name of it from ps

    let parent_pid: u32 = process::parent_id();
    let path: String = format!("/proc/{}/stat", parent_pid);

    let mut parent_stat: File = match File::open(path.to_string()) {
        Ok(r) => r,
        Err(e) => {
            print!("Can't open from {} - {}", path, e);
            return terminal
        },
    };
    let mut contents: String = String::new();
    match parent_stat.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(e) => {
            print!("Can't open from {} - {}", path, e);
            return terminal
        },
    }

    let terminal_pid: u32 = match contents.split(" ").collect::<Vec<&str>>()[3].parse() {
        Ok(r) => r,
        Err(e) => {
            print!("Can't parse terminal pid: {}", e);
            return terminal
        },
    };

    // And credit to https://superuser.com/a/632984 for the file based solution, as ps and
    // sysinfo were too slow
    // println!("{}", terminal_pid);
    let path: String = format!("/proc/{}/cmdline", terminal_pid);

    let mut terminal_cmdline: File = match File::open(path.to_string()) {
        Ok(r) => r,
        Err(e) => {
            print!("Can't open from {} - {}", path, e);
            return terminal
        },
    };
    let mut contents: String = String::new();
    match terminal_cmdline.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(e) => {
            print!("Can't open from {} - {}", path, e);
            return terminal
        },
    }

    contents = contents.split(" ").collect::<Vec<&str>>()[0].to_string();
    contents = contents.split("/").last().unwrap().to_string();
    terminal.terminal_name = contents;

    terminal
}
