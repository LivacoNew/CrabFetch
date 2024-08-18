// Handles and caches all syscalls used between module
// This is to prevent duplicate work being done as well as leaving most of our unsafe options in a
// single place

use std::mem;

use crate::util;

pub struct SyscallCache {
    // https://man7.org/linux/man-pages/man2/sysinfo.2.html
    sysinfo: Option<libc::sysinfo>,
    uname: Option<libc::utsname>,
}
impl SyscallCache {
    pub fn new() -> Self {
        Self {
            sysinfo: None,
            uname: None
        }
    }

    // Cache our syscalls
    fn cache_sysinfo(&mut self) {
        let mut sysinfo_buffer: libc::sysinfo;
        unsafe {
            sysinfo_buffer = mem::zeroed();
            libc::sysinfo(&mut sysinfo_buffer);
        }
        self.sysinfo = Some(sysinfo_buffer);
    }
    fn cache_uname(&mut self) {
        let mut uname_buffer: libc::utsname;
        unsafe {
            uname_buffer = mem::zeroed();
            libc::uname(&mut uname_buffer);
        }
        self.uname = Some(uname_buffer);
    }

    // Get the syscalls, and process/cache them if they're not gotten already
    pub fn get_sysinfo_cached(&mut self) -> libc::sysinfo {
        if self.sysinfo.is_none() {
            self.cache_sysinfo();
        }
        self.sysinfo.unwrap()
    }
    pub fn get_uname_cached(&mut self) -> Utsname {
        if self.uname.is_none() {
            self.cache_uname();
        }

        Utsname::from_libc(self.uname.unwrap())
    }
}

// Better syscall structures than the built in libc ones
// These handle all the parsing from C to Rust stuff for us
// Anything that doesn't need complex parsing (e.g sysinfo) just returns the original structure
pub struct Utsname {
    pub sysname: String,
    pub nodename: String,
    pub release: String,
    pub version: String,
    pub machine: String
}
impl Utsname {
    pub fn from_libc(utsname: libc::utsname) -> Self {
        Self {
            sysname: util::cstr_from_ptr(utsname.sysname.as_ptr()).expect("Unable to convert CStr to Rust String"),
            nodename: util::cstr_from_ptr(utsname.nodename.as_ptr()).expect("Unable to convert CStr to Rust String"),
            release: util::cstr_from_ptr(utsname.release.as_ptr()).expect("Unable to convert CStr to Rust String"),
            version: util::cstr_from_ptr(utsname.version.as_ptr()).expect("Unable to convert CStr to Rust String"),
            machine: util::cstr_from_ptr(utsname.machine.as_ptr()).expect("Unable to convert CStr to Rust String"),
        }
    }
}
