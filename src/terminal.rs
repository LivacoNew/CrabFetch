use std::{env, fs::{self, File}, io::Read, os::unix::process};

use serde::Deserialize;

use crate::{formatter::CrabFetchColor, config_manager::Configuration, Module, ModuleError};

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
    pub format: Option<String>,
    pub chase_ssh_pts: bool
}
impl Module for TerminalInfo {
    fn new() -> TerminalInfo {
        TerminalInfo {
            terminal_name: "Unknown".to_string(),
        }
    }

    fn style(&self, config: &Configuration, max_title_size: u64) -> String {
        let title_color: &CrabFetchColor = config.terminal.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.terminal.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.terminal.title_italic.unwrap_or(config.title_italic);
        let seperator: &str = config.terminal.seperator.as_ref().unwrap_or(&config.seperator);

        let value: String = self.replace_color_placeholders(&self.replace_placeholders(config));

        Self::default_style(config, max_title_size, &config.terminal.title, title_color, title_bold, title_italic, seperator, &value)
    }
    fn unknown_output(config: &Configuration, max_title_size: u64) -> String { 
        let title_color: &CrabFetchColor = config.terminal.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.terminal.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.terminal.title_italic.unwrap_or(config.title_italic);
        let seperator: &str = config.terminal.seperator.as_ref().unwrap_or(&config.seperator);

        Self::default_style(config, max_title_size, &config.terminal.title, title_color, title_bold, title_italic, seperator, "Unknown")
    }

    fn replace_placeholders(&self, config: &Configuration) -> String {
        let format: String = config.terminal.format.clone().unwrap_or("{terminal_name}".to_string());
        format.replace("{terminal_name}", &self.terminal_name)
    }
}

pub fn get_terminal(chase_ssh_tty: bool) -> Result<TerminalInfo, ModuleError> {
    let mut terminal: TerminalInfo = TerminalInfo::new();

    // This is just the rust-ified solution from https://askubuntu.com/a/508047
    // Not sure how well it works in all terminals, but it works fine for my tests in Kitty and
    // Allacritty, so idm.
    //
    // basename "/"$(ps -o cmd -f -p $(cat /proc/$(echo $$)/stat | cut -d \  -f 4) | tail -1 | sed 's/ .*$//')
    // Essentially, it grabs the parent process, grabs the parent of that pid from /proc/x/stat and
    // gets the name of it from ps

    // println!("Starting terminal:");
    let mut terminal_pid: Option<u32> = None;

    let mut loops = 0; // always use protection against infinite loops kids
    let mut parent_pid: u32 = process::parent_id();
    let mut shell_level: u8 = match env::var("SHLVL") {
        Ok(r) => match r.parse::<u8>() {
            Ok(r) => r,
            Err(e) => return Err(ModuleError::new("Terminal", format!("Could not parse $SHLVL env variable: {}", e)))
        },
        Err(e) => return Err(ModuleError::new("Terminal", format!("Could not get $SHLVL env variable: {}", e)))
    };
    let mut stat_contents: String = String::new();
    while shell_level > 0 {
        if loops > 10 {
            return Err(ModuleError::new("Terminal", "Terminal PID loop ran for more than 10 iterations! Either I'm in a infinite loop, or you're >10 shells deep, in which case you're a moron.".to_string()));
        }
        loops += 1;

        let path: String = format!("/proc/{}/stat", parent_pid);
        // println!("Shell proccess ID should be {} leading to {}", parent_pid, path);

        let mut parent_stat: File = match File::open(&path) {
            Ok(r) => r,
            Err(e) => return Err(ModuleError::new("Terminal", format!("Can't open from {} - {}", path, e))),
        };
        match parent_stat.read_to_string(&mut stat_contents) {
            Ok(_) => {},
            Err(e) => return Err(ModuleError::new("Terminal", format!("Can't open from {} - {}", path, e))),
        }
        // println!("Got contents: {}", contents);

        let content_split: Vec<&str> = stat_contents.split(' ').collect::<Vec<&str>>();

        if shell_level == 1 {
            terminal_pid = Some(match content_split[3].parse() {
                Ok(r) => r,
                Err(e) => return Err(ModuleError::new("Terminal", format!("Can't parse terminal pid: {}", e))),
            });
        } else {
            // go up a level
            parent_pid = match content_split[3].parse() {
                Ok(r) => r,
                Err(e) => return Err(ModuleError::new("Terminal", format!("Can't parse parent pid: {}", e))),
            };
        }

        shell_level -= 1;
    }

    // And credit to https://superuser.com/a/632984 for the file based solution, as ps and
    // sysinfo were too slow
    // println!("{}", terminal_pid);
    if terminal_pid.is_none() {
        return Err(ModuleError::new("Terminal", format!("Was unsuccessfull in finding Terminal's PID, last checked; {}", parent_pid)));
    }

    let terminal_pid: u32 = terminal_pid.unwrap();
    let path: String = format!("/proc/{}/cmdline", terminal_pid);

    let mut terminal_cmdline: File = match File::open(&path) {
        Ok(r) => r,
        Err(e) => return Err(ModuleError::new("Terminal", format!("Can't open from {} - {}", path, e))),
    };
    let mut contents: String = String::new();
    match terminal_cmdline.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(e) => return Err(ModuleError::new("Terminal", format!("Can't open from {} - {}", path, e))),
    }

    contents = contents.split(' ').collect::<Vec<&str>>()[0].to_string();
    // Fix for this happening; https://cdn.discordapp.com/attachments/1011301373482115163/1221945908250280096/image.png?ex=66146ccf&is=6601f7cf&hm=2045e0d8150ff468c84ee0fe10ca9105dd4793df05c599715bd1bd7c74d4dc9d&
    contents = contents.split("--").next().unwrap().to_string();
    contents = contents.split('/').last().unwrap().to_string();
    // Fix for gnome terminal coming out as gnome-terminal-server
    if contents.trim().replace('\0', "") == "gnome-terminal-server" {
        contents = "GNOME Terminal".to_string();
    }
    if contents.trim().replace('\0', "") == "sshd:" {
        if !chase_ssh_tty {
            contents = "SSH Terminal".to_string();
        } else {
            // Find the tty number in the current /stat and just construct it from that
            // Taken from the comment on https://unix.stackexchange.com/a/77797 by "user723" ...
            // get a more creative username man
            contents = match fs::canonicalize("/proc/self/fd/0") {
                Ok(r) => r.display().to_string(),
                Err(e) => return Err(ModuleError::new("Terminal", format!("Failed to canonicalize /proc/self/fd/0 symlink: {}", e)))
            };
        }
    }
    terminal.terminal_name = contents;

    Ok(terminal)
}
