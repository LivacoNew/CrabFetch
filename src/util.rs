// Some utility functions

use std::{ffi::CStr, fs::File, io::Read, path::{Path, PathBuf}};

// Quickly read the full contents of a specified file
// Not reccomended for large files, use a buffer instead
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

// Select the first path in the vec that exists 
// Used by stuff like Battery and Host
pub fn find_first_path_exists(paths: Vec<&Path>) -> Option<&Path> {
    for p in paths {
        if !p.exists() {
            continue
        }

        return Some(p);
    }

    None
}
// Select the first path in the vec that exists 
// Used by stuff like Battery and Host
pub fn find_first_pathbuf_exists(paths: Vec<PathBuf>) -> Option<PathBuf> {
    for p in paths {
        if !p.exists() {
            continue
        }

        return Some(p);
    }

    None
}

// Is a bigflag set?
pub fn is_flag_set_u32(value: u32, flag: u32) -> bool {
    value & flag > 0
}

// Quickly convert a CStr to String 
// Wrapper to make this safe
pub fn cstr_from_ptr(ptr: *const i8) -> Result<String, String> {
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
