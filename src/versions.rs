// Purely handles version detection
use std::{env, fs, process::Command};

use sha2::{Sha256, Digest};

use crate::{modules::shell::KNOWN_SHELLS, package_managers::ManagerInfo, proccess_info::ProcessInfo};

pub fn find_version(exe_path: &str, name: Option<&str>, use_checksums: bool, package_managers: &ManagerInfo) -> Option<String> {
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

        _ => None
    };
    if app_specific.is_some() {
        return app_specific;
    }

    if exe_path.starts_with("/usr/bin") || exe_path.starts_with("/usr/lib") {
        // Consult the package manager
        let package_manager: Option<String> = use_package_manager(substitite_package_name(name), package_managers);
        if package_manager.is_some() {
            return package_manager;
        }
    }

    if use_checksums {
        // Match the checksum
        let checksum: Option<String> = match_checksum(exe_path);
        if checksum.is_some() {
            return checksum;
        }
    }
    
    // Failing the above, we run {command} --version and parse it
    parse_command(exe_path, name)
}

fn use_package_manager(name: &str, package_managers: &ManagerInfo) -> Option<String> {
    if let Some(package) = package_managers.packages.get(name) {
        return Some(package.version.to_string());
    }
    None
}
fn match_checksum(path: &str) -> Option<String> {
    // Read all the byte of that file
    let file_bytes: Vec<u8> = match fs::read(path) {
        Ok(r) => r,
        Err(_) => return None,
    };

    let mut hasher = Sha256::new();
    hasher.update(file_bytes);
    
    compare_hash(&hex::encode(hasher.finalize()))
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
    if parent_name == name && !KNOWN_SHELLS.contains(&parent_name.as_str()) {
        panic!("DANGER: Parent process re-invoked for version checking, without it being a known shell. This has the possibility to create a mini-fork bomb! Called {parent_name} vs {name} \nStopping before I break something...");
    }

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
        // Terminals
        "xterm" => Some(raw.split('(').collect::<Vec<&str>>()[1].split(')').next().unwrap().to_string()),
        "foot" => Some(raw.split(' ').collect::<Vec<&str>>()[2].trim().to_string()),
        // Shells
        "bash" => Some(raw.split(' ').collect::<Vec<&str>>()[3].split('(').next().unwrap().trim().to_string()),
        "fish" => Some(raw.split(' ').collect::<Vec<&str>>()[2].trim().to_string()),
        "elvish" => Some(raw.split('+').collect::<Vec<&str>>()[0].trim().to_string()),
        // Editors
        "vim" => Some(raw.split(' ').collect::<Vec<&str>>()[4].to_string()),
        "nvim" => Some(raw.split(' ').collect::<Vec<&str>>()[1].split('\n').next().unwrap()[1..].to_string()),
        // Init Systems
        "systemd" => Some(raw.split(' ').collect::<Vec<&str>>()[2].split('\n').next().unwrap().trim_matches(&['(', ')']).to_string()),

        _ => {
            let raw_split: Vec<&str> = raw.split(' ').collect();
            let attempted_ver: &str = match raw_split.get(1) {
                Some(r) => r,
                None => return None,
            };
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


// Known Hashes
// Please contribute these so I'm not mind-numbingly doing these
// 
// Oh, and to the nerds who want to critisize this detection method in issues; this is a last ditch
// resort to running {program} --version, as running commands like that is likely to cause really
// really fuckin bad performance, but if we already know the hashes, we don't even have to do that. 
// I'm sorry if it upsets you that this isn't going to catch every package's binary with every
// build paramater and difference.
fn compare_hash(hash: &str) -> Option<String> {
    match hash {
        // Kitty
        "bfc1a826895089928bd40eb09a340c6f3b6eb22d51589ca32c032761ff44843b" => Some("0.35.2".to_string()), // Arch

        // Alacritty
        "da793f342a25fd9cb017154bc960d7e6a93d1d9d87a5bfdaf46738d7356fce13" => Some("0.13.2".to_string()), // Arch

        // Foot
        "2806412806ca7289f0f9fe1d73cd28c050f53204e46d9d5610acb0bac9f347ff" => Some("1.17.2".to_string()), // Arch 

        // Terminator
        "8735bed0720a0f5ed8297fcf870a091199044a762dd360d439cfbb6b9871a7b1" => Some("2.1.4".to_string()), // Arch

        // ZSH
        "7ac8cc89b75b595955ec56d8e4b6047c2fc233a6a10c81a137c8417d17a9a970" => Some("5.9".to_string()), // Arch

        // Bash
        "c3a57cc13505b15535662f892fa73f2fd922d1de80be4fa6a4b78be8a59e69f8" => Some("5.2.26".to_string()), // Arch
        "0936c178ac1e145ede22b277f8cb6a6ce3d1390492628ef999b941a65e51fe8e" => Some("5.2.21".to_string()), // VoidLinux live boot

        _ => None
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
