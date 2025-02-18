// Purely handles version detection
use std::{env, process::Command};

use crate::{modules::shell::KNOWN_SHELLS, common_sources::package_managers::ManagerInfo, proccess_info::ProcessInfo};

pub fn find_version(exe_path: &str, name: Option<&str>, package_managers: &ManagerInfo) -> Option<String> {
    // Steps;
    // If it's located in /usr/bin, go to the package manager caches and search for it
    // If not (or not found), check the known checksums 
    // If not found either, ONLY THEN go to {command} --version parsing 
    let name: &str = name.unwrap_or(exe_path.split('/').last().unwrap());

    // We'll try app specific stuff first 
    let app_specific: Option<String> = match name {
        "konsole" => konsole_version(),
        "xterm" => xterm_version(),

        "bash" => bash_version(),
        "fish" => fish_version(),
        "zsh" => zsh_version(),
        "nu" => nushell_version(),

        _ => None
    };
    if app_specific.is_some() {
        return app_specific;
    }

    if exe_path.starts_with("/usr/bin") || exe_path.starts_with("/usr/lib") || exe_path.starts_with("/data/data/com.termux/files/usr/bin") {
        // Consult the package manager
        let package_manager: Option<String> = use_package_manager(substitite_package_name(name), package_managers);
        if package_manager.is_some() {
            return package_manager;
        }
    }

    // Failing the above, we run {command} --version and parse it
    parse_command(exe_path, name)
}

fn use_package_manager(name: &str, package_managers: &ManagerInfo) -> Option<String> {
    if let Some(package) = package_managers.packages.get(name) {
        return Some(package.version.to_string());
    }

    if name == "weston-terminal" {
        if let Some(package) = package_managers.packages.get("weston") {
            return Some(package.version.to_string());
        }
    }

    None
}
fn parse_command(path: &str, name: &str) -> Option<String> {
    // uhoh, expect shitty performance

    // confirm this isn't a infinite loop situation where the parent process started us and we're
    // away to call the parent process again... starting us again and causing a black hole to form
    // and devour the universe... or worse, fill up your ram 
    let mut parent_process: ProcessInfo = ProcessInfo::new_from_parent();
    let parent_name: String = match parent_process.get_process_name() {
        Ok(r) => r,
        Err(_) => return None // Would rather play it safe
    };
    assert!(parent_name != name || KNOWN_SHELLS.contains(&parent_name.as_str()), "DANGER: Parent process re-invoked for version checking, without it being a known shell. This has the possibility to create a mini-fork bomb! Called {parent_name} vs {name} \nStopping before I break something...");

    let mut command: Command = Command::new(path);
    if name == "xterm" || name == "elvish" {
        command.arg("-version");
    } else {
        command.arg("--version");
    }
    let output: Vec<u8> = match command.output() {
            Ok(r) => r.stdout,
            Err(_) => return None,
        };

    let raw: String = match String::from_utf8(output) {
        Ok(r) => r.trim().to_string(),
        Err(_) => return None,
    };

    // Fixes for different outputs
    // Warning: Messy 1-liners
    match name {
        "bash" => Some(raw.split(' ').collect::<Vec<&str>>()[3].split('(').next().unwrap().trim().to_string()),
        "elvish" => Some(raw.split('+').collect::<Vec<&str>>()[0].trim().to_string()),
        "foot" | "fish" => Some(raw.split(' ').collect::<Vec<&str>>()[2].trim().to_string()),
        "nvim" => Some(raw.split(' ').collect::<Vec<&str>>()[1].split('\n').next().unwrap()[1..].to_string()),
        "systemd" => Some(raw.split(' ').collect::<Vec<&str>>()[2].split('\n').next().unwrap().trim_matches(['(', ')']).to_string()),
        "vim" => Some(raw.split(' ').collect::<Vec<&str>>()[4].to_string()),
        "xterm" => Some(raw.split('(').collect::<Vec<&str>>()[1].split(')').next().unwrap().to_string()),

        _ => {
            let raw_split: Vec<&str> = raw.split(' ').collect();
            let attempted_ver: &str = raw_split.get(1)?;
            Some(attempted_ver.to_string())
        },
    }
}

// Package Managers 
fn substitite_package_name(name: &str) -> &str {
    // Substitutes a executable name for the package's name
    // E.g turns nvim to neovim 
    match name {
        "nvim" => "neovim",
        _ => name
    }
}


// Program specific detections
// Shoutout to this answer for alerting me that these get exported in most apps;
// https://stackoverflow.com/a/38240328

// Terminals
fn konsole_version() -> Option<String> {
    // https://phabricator.kde.org/D12621 
    match env::var("KONSOLE_VERSION") {
        Ok(r) => Some(format!("{}.{}.{}", &r[0..2], &r[2..4], &r[4..6])),
        Err(_) => None,
    }
}
fn xterm_version() -> Option<String> {
    match env::var("XTERM_VERSION") {
        Ok(r) => Some(r.split('(').collect::<Vec<&str>>()[1].split(')').next().unwrap().to_string()),
        Err(_) => None,
    }
}

// Shells
fn bash_version() -> Option<String> {
    match env::var("BASH_VERSION") {
        Ok(r) => Some(r.split('(').collect::<Vec<&str>>()[0].to_string()),
        Err(_) => None,
    }
}
fn zsh_version() -> Option<String> {
    match env::var("ZSH_VERSION") {
        Ok(r) => Some(r),
        Err(_) => None,
    }
}
fn fish_version() -> Option<String> {
    match env::var("FISH_VERSION") {
        Ok(r) => Some(r),
        Err(_) => None,
    }
}
fn nushell_version() -> Option<String> {
    match env::var("NU_VERSION") {
        Ok(r) => Some(r),
        Err(_) => None,
    }
}
