use std::process::Command;

fn main() {
    let command = match Command::new("git")
        .args(["log", "-1"])
        .output() {
            Ok(r) => r,
            Err(e) => {
                println!("Warning: Unable to find git information, version commmand may be mangled: {e}");
                println!("cargo:rustc-env=GIT_HASH=Unknown");
                println!("cargo:rustc-env=GIT_DATE=Unknown");
                println!("cargo:rustc-env=GIT_MESSAGE=Unknown");
                return
            },
        };
    if !command.status.success() {
        println!("Warning: Unable to find git information, version commmand may be mangled.");
        println!("cargo:rustc-env=GIT_HASH=Unknown");
        println!("cargo:rustc-env=GIT_DATE=Unknown");
        println!("cargo:rustc-env=GIT_MESSAGE=Unknown");
        return
    }
    
    let output: String = String::from_utf8(command.stdout).expect("Unable to parse git output to a string.");
    let output_lines: Vec<&str> = output.split('\n').collect();

    for line in &output_lines {
        if let Some(commit_hash) = line.trim().strip_prefix("commit ") {
            println!("cargo:rustc-env=GIT_HASH={}", &commit_hash);
        }
        if let Some(commit_date) = line.trim().strip_prefix("Date: ") {
            println!("cargo:rustc-env=GIT_DATE={}", &commit_date.trim());
        }
    }

    let commit_message: &str = output_lines[output_lines.len() - 2].trim();
    println!("cargo:rustc-env=GIT_MESSAGE={}", commit_message);
}
