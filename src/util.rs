// Some utility functions

use std::{fs::File, io::Read, path::Path};

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
pub fn find_first_that_exists(paths: Vec<&Path>) -> Option<&Path> {
    for p in paths {
        if !p.exists() {
            continue
        }

        return Some(p);
    }

    None
}
