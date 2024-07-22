// Purely handles version detection

fn get_version(exe_path: &str) {
    // Steps;
    // If it's located in /usr/bin, go to the package manager caches and search for it
    // If not (or not found), check the known checksums 
    // If not found either, ONLY THEN go to {command} --version parsing 

    if exe_path.starts_with("/usr/bin") {
        // Consult the package manager

    }
}

fn use_package_manager(name: &str) {
    todo!()
}
fn match_checksum(path: &str) {
    todo!()
}
fn parse_command(path: &str) {
    todo!()
}
