// Fetch info from a process from /proc 

use std::{fs::{self, File}, io::Read, path::{Path, PathBuf}, os::unix::process::parent_id};

// https://man7.org/linux/man-pages/man5/proc_pid_stat.5.html
#[derive(Clone)]
#[allow(dead_code)]
pub struct ProcessStatus {
    pub pid: u32,
    pub comm: String,
    pub ppid: u32
}
impl ProcessStatus {
    pub fn from_stat_file(contents: String) -> Self {
        let split_space: Vec<&str> = contents.split(' ').collect();
        
        let lower_comm: usize = contents.find('(').unwrap();
        let upper_comm: usize = contents.find(')').unwrap() + 1;

        let ppid: u32 = contents[upper_comm..].split(' ').collect::<Vec<&str>>().get(2).unwrap().parse().unwrap();
        
        Self {
            pid: split_space[0].parse().unwrap(),
            comm: contents[lower_comm..upper_comm].to_string(),
            ppid
        }
    }
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct ProcessInfo {
    pub pid: u32,
    process_name: Option<String>,
    exe: Option<String>,
    cmdline: Option<Vec<String>>,
    stat: Option<ProcessStatus>,

    path: PathBuf
}
impl ProcessInfo {
    pub fn new(pid: u32) -> Self {
        ProcessInfo {
            pid,
            process_name: None,
            exe: None,
            cmdline: None,
            stat: None,

            path: Path::new(&format!("/proc/{}", pid)).to_owned()
        }
    }
    pub fn new_from_parent() -> Self {
        Self::new(parent_id())
    }

    pub fn is_valid(&self) -> bool {
        self.path.exists()
    }

    pub fn get_exe(&mut self, work_around_python: bool) -> Result<String, String> {
        match &self.exe {
            Some(r) => Ok(r.to_string()),
            None => {
                match fs::canonicalize(self.path.join("exe")) {
                    Ok(r) => {
                        self.exe = Some(r.display().to_string());
                        if work_around_python && self.exe.as_ref().unwrap().contains("python") {
                            // I hate python 
                            // Parse cmdline and use the second argument instead
                            let cmdline: Vec<String> = self.get_cmdline()?;
                            self.exe = match cmdline.get(1) {
                                Some(r) => Some(r.to_string()),
                                None => return Err("Unable to work around python. :(".to_string()),
                            }
                        }

                        Ok(self.exe.as_ref().unwrap().to_string())
                    },
                    Err(e) => Err(format!("Unable to canonicalize /exe, is this process still valid? ({})", e))
                }
            }
        }
    }
    pub fn get_process_name(&mut self) -> Result<String, String> {
        match &self.process_name {
            Some(r) => Ok(r.to_string()),
            None => {
                match self.get_exe(true) {
                    Ok(r) => {
                        self.process_name = Some(r.split('/').last().unwrap().to_string());
                        Ok(self.process_name.as_ref().unwrap().to_string())
                    },
                    Err(e) => Err(format!("Unable to get exe path: {}", e))
                }
            }
        }
    }

    pub fn get_cmdline(&mut self) -> Result<Vec<String>, String> {
        match &self.cmdline {
            Some(r) => Ok(r.to_vec()),
            None => {
                let mut file: File = match File::open(self.path.join("cmdline")) {
                    Ok(r) => r,
                    Err(e) => return Err(format!("Unable to open /cmdline, is this process still valid? ({})", e))
                };
                let mut contents: String = String::new();
                match file.read_to_string(&mut contents) {
                    Ok(_) => {},
                    Err(e) => return Err(format!("Unable to open /cmdline, is this process still valid? ({})", e))
                }
                
                self.cmdline = Some(contents.split('\0').map(|x| x.to_string()).collect());
                Ok(self.cmdline.as_ref().unwrap().to_vec())
            }
        }
    }

    pub fn get_stat(&mut self) -> Result<ProcessStatus, String> {
        match &self.stat {
            Some(r) => Ok(r.clone()),
            None => {
                let mut file: File = match File::open(self.path.join("stat")) {
                    Ok(r) => r,
                    Err(e) => return Err(format!("Unable to open /stat, is this process still valid? ({})", e))
                };
                let mut contents: String = String::new();
                match file.read_to_string(&mut contents) {
                    Ok(_) => {},
                    Err(e) => return Err(format!("Unable to open /stat, is this process still valid? ({})", e))
                }
                
                self.stat = Some(ProcessStatus::from_stat_file(contents));
                Ok(self.stat.as_ref().unwrap().clone())
            }
        }
    }

    pub fn get_parent_pid(&mut self) -> Result<u32, String> {
        let stat = self.get_stat()?;
        Ok(stat.ppid)
    }

    pub fn get_parent_process(&mut self) -> Result<ProcessInfo, String> {
        let pid: u32 = match self.get_parent_pid() {
            Ok(r) => r,
            Err(e) => return Err(e),
        };

        let process: ProcessInfo = ProcessInfo::new(pid);
        if !process.is_valid() {
            return Err("Parent process was not valid.".to_string());
        }

        Ok(process)
    }
}
