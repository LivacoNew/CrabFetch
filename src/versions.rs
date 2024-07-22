// Purely handles version detection

pub fn find_version(exe_path: &str) -> Option<String> {
    // Steps;
    // If it's located in /usr/bin, go to the package manager caches and search for it
    // If not (or not found), check the known checksums 
    // If not found either, ONLY THEN go to {command} --version parsing 

    if exe_path.starts_with("/usr/bin") {
        // Consult the package manager
        let name: &str = exe_path.split('/').last().unwrap();
        let package_manager: Option<String> = use_package_manager(name);
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
    parse_command(exe_path)
}

fn use_package_manager(name: &str) -> Option<String> {
    todo!()
}
fn match_checksum(path: &str) -> Option<String> {
    todo!()
}
fn parse_command(path: &str) -> Option<String> {
    todo!()
}
