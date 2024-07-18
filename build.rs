use std::process::Command;

fn main() {
    // https://stackoverflow.com/a/44407625
    let command = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .expect("Unable to fetch git hash.");

    let hash = String::from_utf8(command.stdout).unwrap();
    println!("cargo:rustc-env=GIT_HASH={}", hash);
}
