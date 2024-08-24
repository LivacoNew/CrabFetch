// Handles and caches all syscalls used between module
// This is to prevent duplicate work being done as well as leaving most of our unsafe options in a
// single place

use std::mem;

use libc::{geteuid, getpwuid};

use crate::util;

pub struct SyscallCache {
    // https://man7.org/linux/man-pages/man2/sysinfo.2.html
    sysinfo: Option<libc::sysinfo>,
    uname: Option<libc::utsname>,
    euid: Option<u32>,
    passwd: Option<libc::passwd>,
}
impl SyscallCache {
    pub fn new() -> Self {
        Self {
            sysinfo: None,
            uname: None,
            euid: None,
            passwd: None,
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
    fn cache_euid(&mut self) {
        unsafe {
            self.euid = Some(geteuid());
        }
    }
    fn cache_passwd(&mut self) {
        let passwd_buffer: libc::passwd;
        unsafe { 
            let user_id: u32 = self.get_euid_cached();
            let buffer_ptr: *mut libc::passwd = getpwuid(user_id);
            if buffer_ptr.is_null() {
                // Null pointer, this is a crash as we have no error handling for the time being
                // TODO: Handle this properly
                panic!("passwd buffer pointer is null (No error handling for this is implemented yet)");
            }
            passwd_buffer = *buffer_ptr;
        }
        self.passwd = Some(passwd_buffer);
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
    pub fn get_euid_cached(&mut self) -> u32 {
        if self.euid.is_none() {
            self.cache_euid();
        }

        self.euid.unwrap()
    }
    pub fn get_passwd_cached(&mut self) -> Passwd {
        if self.passwd.is_none() {
            self.cache_passwd();
        }

        Passwd::from_libc(self.passwd.unwrap())
    }
}

// Better syscall structures than the built in libc ones
// These handle all the parsing from C to Rust stuff for us
// Anything that doesn't need complex parsing (e.g sysinfo) just returns the original structure
// Dead code is allowed in structs, as they may be able to be used for future things
#[allow(dead_code)]
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


#[allow(dead_code)]
pub struct Passwd {
    pub name: String,
    pub uid: u32,
    pub gid: u32,
    pub dir: String,
    pub shell: String
}
impl Passwd {
    pub fn from_libc(passwd: libc::passwd) -> Self {
        Self {
            name: util::cstr_from_ptr(passwd.pw_name).expect("Unable to convert CStr to Rust String"),
            uid: passwd.pw_uid,
            gid: passwd.pw_gid,
            dir: util::cstr_from_ptr(passwd.pw_dir).expect("Unable to convert CStr to Rust String"),
            shell: util::cstr_from_ptr(passwd.pw_shell).expect("Unable to convert CStr to Rust String"),
        }
    }
}
