use std::process::Command;

fn main() {
    get_specific_git_info("hash", "GIT_HASH", "--format=format:%H", false);
    get_specific_git_info("date", "GIT_DATE", "--format=format:%aD", false);
    get_specific_git_info("message", "GIT_MESSAGE", "--format=format:%B", true);
}

/// Store specific information about the most recent `git log` entry in
/// environment variables at build time.
///
/// The `label` is for human consumption and should be representative of the
/// information being retrieved.
///
/// The `env_var` is the name of the environment variable to store the
/// information in, such as `GIT_HASH`.
///
/// The `log_format_arg` is the full `--format` argument passed to `git log` to
/// retrieve the relevant information, such as `--format=format:%H`, which
/// returns the full commit hash and nothing more.
///
/// If `multiline_treatment` is true, the value returned by `git log` will be
/// split by lines, and each line will have `<br>` prepended, except the first.
fn get_specific_git_info(
    label: &str,
    env_var: &str,
    log_format_arg: &str,
    multiline_treatment: bool,
) {
    match Command::new("git")
        .args(["log", "-1", "--no-show-signature", log_format_arg])
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                let info = String::from_utf8(output.stdout)
                    .expect("Unable to parse git output to a string.");
                let info = info.trim();

                if multiline_treatment {
                    let info_lines: Vec<&str> = info.split('\n').collect();
                    let mut info = String::new();
                    let mut saw_first: bool = false;
                    for line in &info_lines {
                        if saw_first {
                            info.push_str("<br>");
                            info.push_str(line.trim());
                        } else {
                            info.push_str(line.trim());
                            saw_first = true;
                        }
                    }
                    println!("cargo:rustc-env={env_var}={}", info.trim());
                } else {
                    println!("cargo:rustc-env={env_var}={}", info.trim());
                }
            } else {
                println!("Warning: Unable to find git {label}, version commmand may be mangled.");
                println!("cargo:rustc-env={env_var}=Unknown");
            }
        }
        Err(err) => {
            println!("Warning: Unable to find git {label}, version commmand may be mangled: {err}");
            println!("cargo:rustc-env={env_var}=Unknown");
        }
    }
}
