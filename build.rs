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
    let mut message: String = String::new();
    let mut in_message: bool = false;
    for line in &output_lines {
        if let Some(commit_hash) = line.trim().strip_prefix("commit ") {
            println!("cargo:rustc-env=GIT_HASH={}", &commit_hash);
        }
        if let Some(commit_date) = line.trim().strip_prefix("Date: ") {
            println!("cargo:rustc-env=GIT_DATE={}", &commit_date.trim());
        }

        if line.is_empty() {
            // Next lines are all part of the message
            in_message = true;
        }
        if in_message {
            message.push_str(line.trim());
            message.push_str("<br>");
        }
    }

    println!("cargo:rustc-env=GIT_MESSAGE={}", message);
}
