use std::fmt::Display;

pub struct MemoryInfo {
    used: f32,
    max: f32,
    speed: u16,
}
impl MemoryInfo {
    fn new() -> MemoryInfo {
        MemoryInfo {
            used: 0.0,
            max: 0.0,
            speed: 0
        }
    }
}
impl Display for MemoryInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} / {} @ {}MHz", self.used, self.max, self.speed)
    }
}

pub fn get_memory() -> MemoryInfo {
    let memory = MemoryInfo::new();
    memory
}
