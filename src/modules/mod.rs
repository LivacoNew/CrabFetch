pub mod cpu;
pub mod gpu;
pub mod os;
pub mod hostname;
pub mod memory;
pub mod swap;
pub mod mounts;
pub mod host;
pub mod displays;
pub mod packages;
pub mod desktop;
pub mod terminal;
pub mod shell;
pub mod uptime;
pub mod editor;
pub mod locale;
pub mod battery;
#[cfg(feature = "player")]
pub mod player;
pub mod initsys;
pub mod processes;
pub mod datetime;
pub mod localip;
pub mod theme;
pub mod icon_theme;
