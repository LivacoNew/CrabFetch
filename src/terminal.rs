use std::{fs::File, io::Read, os::unix::process};

use serde::Deserialize;

use crate::{config_manager::CrabFetchColor, log_error, shell, Module, CONFIG};

pub struct TerminalInfo {
    terminal_name: String
}
#[derive(Deserialize)]
pub struct TerminalConfiguration {
    pub title: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub seperator: Option<String>,
    pub format: Option<String>
}
impl Module for TerminalInfo {
    fn new() -> TerminalInfo {
        TerminalInfo {
            terminal_name: "Unknown".to_string(),
        }
    }

    fn style(&self) -> String {
        let mut title_color: &CrabFetchColor = &CONFIG.title_color;
        if (&CONFIG.terminal.title_color).is_some() {
            title_color = &CONFIG.terminal.title_color.as_ref().unwrap();
        }

        let mut title_bold: bool = CONFIG.title_bold;
        if (CONFIG.terminal.title_bold).is_some() {
            title_bold = CONFIG.terminal.title_bold.unwrap();
        }
        let mut title_italic: bool = CONFIG.title_italic;
        if (CONFIG.terminal.title_italic).is_some() {
            title_italic = CONFIG.terminal.title_italic.unwrap();
        }

        let mut seperator: &str = CONFIG.seperator.as_str();
        if CONFIG.terminal.seperator.is_some() {
            seperator = CONFIG.terminal.seperator.as_ref().unwrap();
        }

        self.default_style(&CONFIG.terminal.title, title_color, title_bold, title_italic, &seperator)
    }

    fn replace_placeholders(&self) -> String {
        let mut format: String = "{terminal_name}".to_string();
        if CONFIG.host.format.is_some() {
            format = CONFIG.host.format.clone().unwrap();
        }

        format.replace("{terminal_name}", &self.terminal_name)
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

    // println!("Starting terminal:");
    let default_shell: String = shell::get_default_shell().shell_name;
    let mut terminal_pid: Option<u32> = None;

    let mut found_terminal_pid: bool = false;
    let mut loops = 0; // always use protection against infinite loops kids
    let mut parent_pid: u32 = process::parent_id();
    while !found_terminal_pid {
        // Once we reach the default shell's PID, go up once more and we're at the terminal PID.
        // This is scrappy but it's the best solution I have for the time being
        // This also breaks if you enter multiple shells and go default shell -> some other shell
        // -> back to your default shell
        // but I don't really care once your at that point
        if loops > 10 {
            panic!("Terminal PID loop ran for more than 10 iterations! Either I'm in a infinite loop, or you're >10 shells deep, in which case you're a moron.");
        }
        loops += 1;

        let path: String = format!("/proc/{}/stat", parent_pid);
        // println!("Shell proccess ID should be {} leading to {}", parent_pid, path);

        let mut parent_stat: File = match File::open(path.to_string()) {
            Ok(r) => r,
            Err(e) => {
                log_error("Terminal", format!("Can't open from {} - {}", path, e));
                return terminal
            },
        };
        let mut contents: String = String::new();
        match parent_stat.read_to_string(&mut contents) {
            Ok(_) => {},
            Err(e) => {
                log_error("Terminal", format!("Can't open from {} - {}", path, e));
                return terminal
            },
        }
        // println!("Got contents: {}", contents);

        let content_split: Vec<&str> = contents.split(" ").collect::<Vec<&str>>();
        let mut pid_name: String = content_split[1].to_string();
        pid_name = pid_name[1..pid_name.len() - 1].to_string();

        if pid_name == default_shell {
            found_terminal_pid = true;
            terminal_pid = Some(match content_split[3].parse() {
                Ok(r) => r,
                Err(e) => {
                    log_error("Terminal", format!("Can't parse terminal pid: {}", e));
                    return terminal
                },
            });
        } else {
            // go up a level
            parent_pid = match content_split[3].parse() {
                Ok(r) => r,
                Err(e) => {
                    log_error("Terminal", format!("Can't parse parent pid: {}", e));
                    return terminal
                },
            };
        }
    }

    // And credit to https://superuser.com/a/632984 for the file based solution, as ps and
    // sysinfo were too slow
    // println!("{}", terminal_pid);
    if terminal_pid.is_none() {
        log_error("Terminal", format!("Was unsuccessfull in finding Terminal's PID, last checked; {}", parent_pid));
        return terminal
    }

    let terminal_pid: u32 = terminal_pid.unwrap();
    let path: String = format!("/proc/{}/cmdline", terminal_pid);

    let mut terminal_cmdline: File = match File::open(path.to_string()) {
        Ok(r) => r,
        Err(e) => {
            log_error("Terminal", format!("Can't open from {} - {}", path, e));
            return terminal
        },
    };
    let mut contents: String = String::new();
    match terminal_cmdline.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(e) => {
            log_error("Terminal", format!("Can't open from {} - {}", path, e));
            return terminal
        },
    }
    // println!("Got full cmdline as {}", contents);
    contents = contents.split(" ").collect::<Vec<&str>>()[0].to_string();
    // Fix for this happening; https://cdn.discordapp.com/attachments/1011301373482115163/1221945908250280096/image.png?ex=66146ccf&is=6601f7cf&hm=2045e0d8150ff468c84ee0fe10ca9105dd4793df05c599715bd1bd7c74d4dc9d&
    contents = contents.split("--").next().unwrap().to_string();
    // println!("Filter 1: {}", contents);
    contents = contents.split("/").last().unwrap().to_string();
    // println!("Filter 2: {}", contents);
    terminal.terminal_name = contents;

    terminal
}
