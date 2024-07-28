// Purely handles version detection

use std::{fs::{self, read_dir, File, ReadDir}, io::{BufRead, BufReader}, process::Command};

use sha2::{Sha256, Digest};

pub fn find_version(exe_path: &str, name: Option<&str>, use_checksums: bool) -> Option<String> {
    // Steps;
    // If it's located in /usr/bin, go to the package manager caches and search for it
    // If not (or not found), check the known checksums 
    // If not found either, ONLY THEN go to {command} --version parsing 

    let name: &str = name.unwrap_or(exe_path.split('/').last().unwrap());

    if exe_path.starts_with("/usr/bin") {
        // Consult the package manager
        let package_manager: Option<String> = use_package_manager(substitite_package_name(name));
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

fn use_package_manager(name: &str) -> Option<String> {
    find_pacman_package(name)
        .or(find_dpkg_package(name))
        .or(find_xbps_package(name))
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

    // Fixes for different terminals outputs
    match name {
        // Terminals
        "xterm" => Some(raw.split('(').collect::<Vec<&str>>()[1].split(')').next().unwrap().to_string()),
        "foot" => Some(raw.split(' ').collect::<Vec<&str>>()[2].trim().to_string()),
        // Shells
        "bash" => Some(raw.split(' ').collect::<Vec<&str>>()[3].split('(').next().unwrap().trim().to_string()),
        "fish" => Some(raw.split(' ').collect::<Vec<&str>>()[2].trim().to_string()),
        "elvish" => Some(raw.split('+').collect::<Vec<&str>>()[0].trim().to_string()),
        // Editors
        "nvim" => Some(raw.split(' ').collect::<Vec<&str>>()[1].split('\n').next().unwrap()[1..].to_string()),

        _ => Some(raw.split(' ').collect::<Vec<&str>>()[1].to_string()),
    }
}





// Package Managers 
fn find_pacman_package(name: &str) -> Option<String> {
    let dir: ReadDir = match read_dir("/var/lib/pacman/local") {
        Ok(r) => r,
        Err(_) => return None,
    };

    for x in dir {
        let d = x.unwrap();
        if !d.metadata().unwrap().is_dir() {
            continue;
        }

        let file_name = d.file_name();
        let package_split: Vec<&str> = file_name.to_str().unwrap().split('-').collect();
        let package_name: String = package_split[0..package_split.len() - 2].join("-");
        
        if package_name != name {
            continue;
        }
        let package_version: String = package_split[package_split.len() - 2].to_string();

        return Some(package_version);
    }

    None
}
fn find_dpkg_package(name: &str) -> Option<String> {
    let file: File = match File::open("/var/lib/dpkg/status") {
        Ok(r) => r,
        Err(_) => return None,
    };

    let buffer: BufReader<File> = BufReader::new(file);
    let mut in_package: bool = false;
    for line in buffer.lines() {
        if line.is_err() {
            continue;
        }
        let line = line.unwrap();
        if line.is_empty() {
            continue;
        }

        if let Some(package_name) = line.strip_prefix("Package: ") {
            if in_package {
                break;
            }

            in_package = package_name == name;
        }
        if in_package {
            if let Some(version) = line.strip_prefix("Version: ") {
                // https://developer.bigfix.com/relevance/reference/debian-package-version.html
                let split: Vec<&str> = version.split(':').collect();
                let version = match split.get(1) {
                    Some(r) => r.to_string(),
                    None => split[0].to_string()
                };
                let final_version: String = version.split('-').next().unwrap().to_string();

                return Some(final_version);
            }
        }
    }

    None
}
fn find_xbps_package(name: &str) -> Option<String> {
    let file: File = match File::open("/var/db/xbps/pkgdb-0.38.plist") {
        Ok(r) => r,
        Err(_) => return None,
    };

    let buffer: BufReader<File> = BufReader::new(file);
    let target_key: String = format!("<key>{name}</key>");
    let mut in_package: bool = false;
    let mut dict_level: u8 = 0;
    let mut parse_str: bool = false;
    for line in buffer.lines() {
        if line.is_err() {
            continue;
        }
        let line = line.unwrap();
        if line.is_empty() {
            continue;
        }
        let line = line.trim();

        if line == "<dict>" && in_package { 
            dict_level += 1;
        }
        if line == "</dict>" && in_package { 
            dict_level -= 1;
            if dict_level == 0 {
                continue;
            }
        }
        if line == target_key {
            in_package = true;
            dict_level += 1;
            continue;
        }
        
        if in_package {
            if line == "<key>pkgver</key>" {
                // Next line is the version
                parse_str = true;
                continue;
            }
            if parse_str {
                let string: String = line[8..line.len() - 9].to_string();
                // {package-name}-{ver}_{rev} 
                let no_rev: &str = string.split('_').next().unwrap();
                let version: &str = no_rev.split('-').last().unwrap();

                return Some(version.to_string());
            }
        }
    }

    None
}

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
