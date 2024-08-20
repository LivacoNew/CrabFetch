// Some utility functions

use std::{ffi::{c_char, CStr}, fs::File, io::Read, path::{Path, PathBuf}};

/// Quickly reads the full contents of a specified file using [File::open] and [File::read_to_string]
/// Don't use this for medium to large sized files, for performance reasons please use a buffer instead.
/// `Err<String>` is returned on failure with the string being the error message.
pub fn file_read(path: &Path) -> Result<String, String> {
    let mut file: File = match File::open(path) {
        Ok(r) => r,
        Err(e) => return Err(e.to_string()),
    };
    let mut contents: String = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(e) => return Err(e.to_string()),
    }

    Ok(contents)
}

/// Checks each [Path] in `paths` for existance, and returns the first one that exists.
/// If none of the paths exists, returns [None]
/// For an owned version, see [find_first_pathbuf_exists]
pub fn find_first_path_exists(paths: Vec<&Path>) -> Option<&Path> {
    for p in paths {
        if !p.exists() {
            continue
        }

        return Some(p);
    }

    None
}
/// Checks each [PathBuf] in `paths` for existance, and returns the first one that exists.
/// If none of the paths exists, returns [None]
/// For a borrowed version, see [find_first_path_exists]
pub fn find_first_pathbuf_exists(paths: Vec<PathBuf>) -> Option<PathBuf> {
    for p in paths {
        if !p.exists() {
            continue
        }

        return Some(p);
    }

    None
}

/// Checks if `value` contains the bits set in `flag`, returning true if so
/// This is just a shorthand for `value & flag > 0`
pub fn is_flag_set_u32(value: u32, flag: u32) -> bool {
    value & flag > 0
}

/// Converts a C string's pointer into a rust [String].
/// `Err<String>` is returned on failure with the string being the error message.
pub fn cstr_from_ptr(ptr: *const c_char) -> Result<String, String> {
    if ptr.is_null() {
        return Err("Pointer is null!".to_string());
    }

    unsafe {
        let str: &str = match CStr::from_ptr(ptr).to_str() {
            Ok(r) => r,
            Err(e) => return Err(e.to_string()),
        };
        Ok(str.to_string())
    }
}

/// Returns true if we're running under Window's WSL
pub fn in_wsl() -> bool {
    // Credit: https://superuser.com/a/1749811
    // Using the first method
    Path::new("/proc/sys/fs/binfmt_misc/WSLInterop").exists()
}
