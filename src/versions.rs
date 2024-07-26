// Purely handles version detection

use std::{fs::{read_dir, ReadDir}, process::Command};

pub fn find_version(exe_path: &str, name: Option<&str>) -> Option<String> {
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

    // Match the checksum
    let checksum: Option<String> = match_checksum(exe_path);
    if checksum.is_some() {
        return checksum;
    }
    
    // Failing the above, we run {command} --version and parse it
    parse_command(exe_path, name)
}

fn use_package_manager(name: &str) -> Option<String> {
    find_pacman_package(name)
}
fn match_checksum(path: &str) -> Option<String> {
    None
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

fn substitite_package_name(name: &str) -> &str {
    // Substitutes a executable name for the package's name
    // E.g turns nvim to neovim 
    match name {
        "nvim" => "neovim",
        _ => name
    }
}
