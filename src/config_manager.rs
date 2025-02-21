use std::{env, fmt::{Debug, Display}, fs::{self, File}, io::Write, path::{Path, PathBuf}};

use config::{builder::DefaultState, Config, ConfigBuilder};
use serde::Deserialize;

use crate::{ascii::AsciiConfiguration, battery::BatteryConfiguration, cpu::CPUConfiguration, datetime::DateTimeConfiguration, desktop::DesktopConfiguration, displays::DisplayConfiguration, editor::EditorConfiguration, formatter::CrabFetchColor, gpu::GPUConfiguration, host::HostConfiguration, hostname::HostnameConfiguration, initsys::InitSystemConfiguration, locale::LocaleConfiguration, memory::MemoryConfiguration, modules::{icon_theme::IconThemeConfiguration, localip::LocalIPConfiguration, theme::ThemeConfiguration}, mounts::MountConfiguration, os::OSConfiguration, packages::PackagesConfiguration, processes::ProcessesConfiguration, shell::ShellConfiguration, swap::SwapConfiguration, terminal::TerminalConfiguration, uptime::UptimeConfiguration, util};
#[cfg(feature = "player")]
use crate::player::PlayerConfiguration;


#[allow(clippy::struct_excessive_bools)]
#[derive(Deserialize)]
pub struct Configuration {
    pub modules: Vec<String>,
    pub unknown_as_text: bool,
    pub allow_commands: bool,
    pub separator: String,
    pub title_color: CrabFetchColor,
    pub title_bold: bool,
    pub title_italic: bool,
    pub decimal_places: u32,
    pub inline_values: bool,
    pub underline_character: char,
    pub color_character: String,
    pub color_margin: u8,
    pub color_use_background: bool,
    pub use_os_color: bool,
    pub segment_top: String,
    pub segment_bottom: String,
    pub progress_left_border: String,
    pub progress_right_border: String,
    pub progress_progress: String,
    pub progress_empty: String,
    pub progress_target_length: u8,
    pub percentage_color_thresholds: Vec<String>,
    pub use_ibis: bool,
    pub suppress_errors: bool,

    pub ascii: AsciiConfiguration,

    pub hostname: HostnameConfiguration,
    pub cpu: CPUConfiguration,
    pub gpu: GPUConfiguration,
    pub memory: MemoryConfiguration,
    pub swap: SwapConfiguration,
    pub mounts: MountConfiguration,
    pub host: HostConfiguration,
    pub displays: DisplayConfiguration,
    pub os: OSConfiguration,
    pub packages: PackagesConfiguration,
    pub desktop: DesktopConfiguration,
    pub terminal: TerminalConfiguration,
    pub shell: ShellConfiguration,
    pub uptime: UptimeConfiguration,
    pub battery: BatteryConfiguration,
    pub locale: LocaleConfiguration,
    #[cfg(feature = "player")]
    pub player: PlayerConfiguration,
    pub editor: EditorConfiguration,
    pub initsys: InitSystemConfiguration,
    pub processes: ProcessesConfiguration,
    pub datetime: DateTimeConfiguration,
    pub localip: LocalIPConfiguration,
    pub theme: ThemeConfiguration,
    pub icontheme: IconThemeConfiguration
}

// Config Error 
pub struct ConfigurationError {
    config_file: String,
    message: String
}
impl ConfigurationError {
    pub fn new(file_path: Option<String>, message: String) -> ConfigurationError {
        ConfigurationError {
            config_file: file_path.unwrap_or("Unknown".to_string()),
            message
        }
    }
}
impl Display for ConfigurationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to parse from file '{}': {}", self.config_file, self.message)
    }
}
impl Debug for ConfigurationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} failed to parse: {}", self.config_file, self.message)
    }
}

#[allow(clippy::ref_option)]
pub fn parse(location_override: &Option<String>, module_override: &Option<String>) -> Result<Configuration, ConfigurationError> {
    let mut builder: ConfigBuilder<DefaultState> = Config::builder();
    let mut config_path_str: Option<String> = None;
    if location_override.is_some() {
        let location_override: String = location_override.clone().unwrap();
        if location_override != "none" {
            config_path_str = Some(shellexpand::tilde(&location_override).to_string());
            let config_path_str: String = config_path_str.as_ref().unwrap().to_string();

            // Verify it exists
            let path: &Path = Path::new(&config_path_str);
            if !path.exists() {
                return Err(ConfigurationError::new(Some(config_path_str), "Unable to find config file.".to_string()));
            }
        }
    } else {
        // Find the config path
        config_path_str = find_file_in_config_dir("config.toml").map(|x| x.display().to_string());
    }

    if config_path_str.is_some() {
        builder = builder.add_source(config::File::with_name(config_path_str.as_ref().unwrap()).required(false));
    }

    // Set the defaults here
    // General
    builder = builder.set_default("modules", vec![
        "hostname".to_string(),
        "underline:16".to_string(),

        "cpu".to_string(),
        "gpu".to_string(),
        "memory".to_string(),
        "swap".to_string(),
        "mounts".to_string(),
        "host".to_string(),
        "displays".to_string(),

        "os".to_string(),
        "packages".to_string(),
        "desktop".to_string(),
        "terminal".to_string(),
        "shell".to_string(),
        "editor".to_string(),
        "uptime".to_string(),
        "locale".to_string(),
        "player".to_string(),
        "initsys".to_string(),
        "processes".to_string(),
        "battery".to_string(),
        "localip".to_string(),

        "space".to_string(),
        "colors".to_string(),
        "bright_colors".to_string(),
    ]).unwrap();

    // Android only module
    #[cfg(feature = "android")]
    if env::consts::OS == "android" {
        builder = builder.set_default("modules", vec![
            "hostname".to_string(),
            "underline:16".to_string(),

            "cpu".to_string(),
            "memory".to_string(),
            "swap".to_string(),
            "mounts".to_string(),
            "host".to_string(),

            "os".to_string(),
            "packages".to_string(),
            "terminal".to_string(),
            "shell".to_string(),
            "editor".to_string(),
            "uptime".to_string(),
            "locale".to_string(),

            "space".to_string(),
            "colors".to_string(),
            "bright_colors".to_string(),
        ]).unwrap();
    }
    builder = builder.set_default("unknown_as_text", false).unwrap();
    builder = builder.set_default("allow_commands", false).unwrap();

    builder = builder.set_default("separator", " > ").unwrap();
    builder = builder.set_default("title_color", "bright_magenta").unwrap();
    builder = builder.set_default("title_bold", true).unwrap();
    builder = builder.set_default("title_italic", false).unwrap();

    builder = builder.set_default("decimal_places", 2).unwrap();
    builder = builder.set_default("inline_values", false).unwrap();
    builder = builder.set_default("underline_character", "―").unwrap();
    builder = builder.set_default("color_character", "   ").unwrap();
    builder = builder.set_default("color_margin", 0).unwrap();
    builder = builder.set_default("color_use_background", true).unwrap();

    builder = builder.set_default("use_os_color", true).unwrap();

    builder = builder.set_default("segment_top", "{color-white}[======------{color-brightmagenta} {name} {color-white}------======]").unwrap();
    builder = builder.set_default("segment_bottom", "{color-white}[======------{color-brightmagenta} {name_sized_gap} {color-white}------======]").unwrap();

    builder = builder.set_default("progress_left_border", "[").unwrap();
    builder = builder.set_default("progress_right_border", "]").unwrap();
    builder = builder.set_default("progress_progress", "=").unwrap();
    builder = builder.set_default("progress_empty", " ").unwrap();
    builder = builder.set_default("progress_target_length", 20).unwrap();

    builder = builder.set_default("use_ibis", false).unwrap();
    builder = builder.set_default("suppress_errors", true).unwrap();

    builder = builder.set_default("percentage_color_thresholds", vec!["75:brightgreen", "85:brightyellow", "90:brightred"]).unwrap();

    // ASCII
    builder = builder.set_default("ascii.display", true).unwrap();
    builder = builder.set_default("ascii.side", "left").unwrap();
    builder = builder.set_default("ascii.margin", 4).unwrap();
    builder = builder.set_default("ascii.mode", "os").unwrap();
    builder = builder.set_default("ascii.solid_color", "bright_magenta").unwrap();
    builder = builder.set_default("ascii.band_colors", vec!["bright_magenta", "bright_cyan", "bright_white", "bright_cyan", "bright_magenta"]).unwrap();

    // Modules
    builder = builder.set_default("hostname.title", "").unwrap();
    builder = builder.set_default("hostname.format", "{color-title}{username}{color-white}@{color-title}{hostname}").unwrap();

    builder = builder.set_default("cpu.title", "CPU").unwrap();
    builder = builder.set_default("cpu.format", "{name} ({core_count}c {thread_count}t) @ {max_clock_ghz} GHz").unwrap();
    builder = builder.set_default("cpu.remove_trailing_processor", true).unwrap();

    builder = builder.set_default("gpu.amd_accuracy", true).unwrap();
    builder = builder.set_default("gpu.ignore_disabled_gpus", true).unwrap();
    builder = builder.set_default("gpu.detect_through_driver", false).unwrap();
    builder = builder.set_default("gpu.title", "GPU").unwrap();
    builder = builder.set_default("gpu.format", "{vendor} {model} ({vram})").unwrap();

    builder = builder.set_default("memory.title", "Memory").unwrap();
    builder = builder.set_default("memory.format", "{used} / {max} ({percent})").unwrap();

    builder = builder.set_default("swap.title", "Swap").unwrap();
    builder = builder.set_default("swap.format", "{used} / {total} ({percent})").unwrap();

    builder = builder.set_default("mounts.title", "Disk ({mount})").unwrap();
    builder = builder.set_default("mounts.format", "{space_used} used of {space_total} ({percent}) [{filesystem}]").unwrap();
    builder = builder.set_default("mounts.ignore", vec![""]).unwrap();

    builder = builder.set_default("host.title", "Host").unwrap();
    builder = builder.set_default("host.format", "{host} ({chassis})").unwrap();
    builder = builder.set_default("host.newline_chassis", false).unwrap();
    builder = builder.set_default("host.chassis_title", "Chassis").unwrap();
    builder = builder.set_default("host.chassis_format", "{chassis}").unwrap();

    builder = builder.set_default("displays.title", "Display ({make} {model})").unwrap();
    builder = builder.set_default("displays.format", "{width}x{height} @ {refresh_rate}Hz ({name})").unwrap();
    builder = builder.set_default("displays.scale_size", false).unwrap();

    builder = builder.set_default("os.title", "Operating System").unwrap();
    builder = builder.set_default("os.format", "{distro} ({kernel})").unwrap();
    builder = builder.set_default("os.newline_kernel", false).unwrap();
    builder = builder.set_default("os.kernel_title", "Kernel").unwrap();
    builder = builder.set_default("os.kernel_format", "Linux {kernel}").unwrap();


    builder = builder.set_default("packages.title", "Packages").unwrap();
    builder = builder.set_default("packages.format", "{count} ({manager})").unwrap();
    builder = builder.set_default("packages.ignore", Vec::<String>::new()).unwrap();

    builder = builder.set_default("desktop.title", "Desktop").unwrap();
    builder = builder.set_default("desktop.format", "{desktop} ({display_type})").unwrap();

    builder = builder.set_default("terminal.title", "Terminal").unwrap();
    builder = builder.set_default("terminal.format", "{name} {version}").unwrap();

    builder = builder.set_default("shell.title", "Shell").unwrap();
    builder = builder.set_default("shell.format", "{name} {version}").unwrap();
    builder = builder.set_default("shell.show_default_shell", "false").unwrap();

    builder = builder.set_default("uptime.title", "Uptime").unwrap();

    builder = builder.set_default("battery.title", "Battery {index}").unwrap();
    builder = builder.set_default("battery.format", "{percentage}%").unwrap();

    builder = builder.set_default("editor.title", "Editor").unwrap();
    builder = builder.set_default("editor.format", "{name} {version}").unwrap();
    builder = builder.set_default("editor.fancy", true).unwrap();

    builder = builder.set_default("locale.title", "Locale").unwrap();
    builder = builder.set_default("locale.format", "{language} ({encoding})").unwrap();

    builder = builder.set_default("player.title", "Player ({player})").unwrap();
    builder = builder.set_default("player.format", "{track} by {track_artists} ({album}) [{status}]").unwrap();
    builder = builder.set_default("player.ignore", Vec::<String>::new()).unwrap();

    builder = builder.set_default("initsys.title", "Init System").unwrap();
    builder = builder.set_default("initsys.format", "{name} {version}").unwrap();

    builder = builder.set_default("processes.title", "Total Processes").unwrap();

    builder = builder.set_default("datetime.title", "Date/Time").unwrap();
    builder = builder.set_default("datetime.format", "%H:%M:%S on %e %B %G").unwrap();

    builder = builder.set_default("localip.title", "Local IP ({interface})").unwrap();
    builder = builder.set_default("localip.format", "{addr}").unwrap();

    builder = builder.set_default("theme.title", "Theme").unwrap();
    builder = builder.set_default("theme.format", "Gtk3: {gtk3}  Gtk4: {gtk4}").unwrap();

    builder = builder.set_default("icontheme.title", "Icons").unwrap();
    builder = builder.set_default("icontheme.format", "Gtk3: {gtk3}  Gtk4: {gtk4}").unwrap();

    // Check for any module overrides
    if module_override.is_some() {
        let module_override: String = module_override.clone().unwrap();
        builder = builder.set_override("modules", module_override.split(',').collect::<Vec<&str>>()).unwrap();
    }

    // Now stop.
    let config: Config = match builder.build() {
        Ok(r) => r,
        Err(e) => return Err(ConfigurationError::new(config_path_str, e.to_string())),
    };

    let deserialized: Configuration = match config.try_deserialize::<Configuration>() {
        Ok(r) => r,
        Err(e) => return Err(ConfigurationError::new(config_path_str, e.to_string())),
    };

    Ok(deserialized)
}

fn find_file_in_config_dir(path: &str) -> Option<PathBuf> {
    // Tries $XDG_CONFIG_HOME/CrabFetch before backing up to $HOME/.config/CrabFetch
    let mut paths: Vec<PathBuf> = Vec::new();
    let mut temp_var_to_shut_up_the_borrow_checker: String;
    if let Ok(config_home) = env::var("XDG_CONFIG_HOME") {
        temp_var_to_shut_up_the_borrow_checker = config_home;
        temp_var_to_shut_up_the_borrow_checker.push_str("/CrabFetch/");
        temp_var_to_shut_up_the_borrow_checker.push_str(path);
        paths.push(PathBuf::from(temp_var_to_shut_up_the_borrow_checker));
    }
    let mut temp_var_to_shut_up_the_borrow_checker: String;
    if let Ok(user_home) = env::var("HOME") {
        temp_var_to_shut_up_the_borrow_checker = user_home;
        temp_var_to_shut_up_the_borrow_checker.push_str("/.config/CrabFetch/");
        temp_var_to_shut_up_the_borrow_checker.push_str(path);
        paths.push(PathBuf::from(temp_var_to_shut_up_the_borrow_checker));
    }

    util::find_first_pathbuf_exists(paths)
}

pub fn check_for_ascii_override() -> Option<String> {
    let path: PathBuf = find_file_in_config_dir("ascii")?;
    if !path.exists() {
        return None;
    }

    match util::file_read(&path) {
        Ok(r) => Some(r),
        Err(_) => None,
    }
}

pub fn generate_config_file(location_override: Option<String>) {
    let path: String;
    if location_override.is_some() {
        path = shellexpand::tilde(&location_override.unwrap()).to_string();
        // Config won't be happy unless it ends with .toml
        assert!(Path::new(&path).extension().is_some_and(|x| x.eq_ignore_ascii_case("toml")), "Config path must end with '.toml'");
    } else {
        // Find the config path
        // Tries $XDG_CONFIG_HOME/CrabFetch before backing up to $HOME/.config/CrabFetch
        path = if let Ok(mut r) = env::var("XDG_CONFIG_HOME") {
            r.push_str("/CrabFetch/config.toml");
            r
        } else {
            // Let's try the home directory
            let mut home_dir: String = match env::var("HOME") {
                Ok(r) => r,
                Err(e) => panic!("Unable to find suitable config folder; {e}")
            };
            home_dir.push_str("/.config/CrabFetch/config.toml");
            home_dir
        }
    }
    let config_path: &Path = Path::new(&path);

    assert!(!config_path.exists(), "Path already exists: {}", config_path.display());
    match fs::create_dir_all(config_path.parent().unwrap()) {
        Ok(_) => {},
        Err(e) => panic!("Unable to create directory: {e}"),
    };

    let mut file: File = match File::create(config_path) {
        Ok(r) => r,
        Err(e) => panic!("Unable to create file; {e}"),
    };
    match file.write_all(DEFAULT_CONFIG_CONTENTS.as_bytes()) {
        Ok(_) => {},
        Err(e) => panic!("Unable to write to file; {e}"),
    };
    println!("Created default config file at {path}");
}

mod tests {
    // Test configs get created correctly, in the correct place and that the TOML is valid
    #[test]
    fn create_config() {
        use std::{fs, path::Path, io::Error};

        let location: String = "/tmp/crabfetch_test_config.toml".to_string();
        crate::config_manager::generate_config_file(Some(location.clone()));
        assert!(Path::new(&location).exists());

        // Attempt to parse it
        let parse = crate::config_manager::parse(&Some(location.clone()), &None);
        assert!(crate::config_manager::parse(&Some(location.clone()), &None).is_ok(), "{:?}", parse.err());
        
        // Finally, we remove the tmp config file 
        let removed: Result<(), Error> = fs::remove_file(location);
        assert!(removed.is_ok()); // Asserting this cus if the file fails to remove it's likely cus it never existed
    }
    
    // Tests that the default-config.toml file is the same as the DEFAULT_CONFIG_CONTENTS string in
    // here 
    // In case anyone's wondering why they're separated; it's so that package maintainers or people
    // who want a copy of the default config without re-genning it can have it without digging in
    // CrabFetch's source code
    // This test's just to make sure I keep it up to date and don't forget to update one or the
    // other
    #[test]
    fn config_is_consistent() {
        use std::{path::{PathBuf, Path}, fs::File, io::Read};
        let mut cargo_loc = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        cargo_loc.push("default-config.toml");
        assert!(Path::new(&cargo_loc).exists());

        let mut file: File = File::open(cargo_loc).unwrap();
        let mut file_contents: String = String::new();
        let _ = file.read_to_string(&mut file_contents);
        // File saving will sometimes add new lines in different places than the rust ver, so I
        // don't bother checking it as it just causes problems
        file_contents = file_contents.replace('\n', "");
        let comparing: &str = &crate::config_manager::DEFAULT_CONFIG_CONTENTS.replace('\n', "");

        assert_eq!(file_contents, comparing);
    }
}

// The default config, stored so that it can be written
const DEFAULT_CONFIG_CONTENTS: &str = include_str!("../default-config.toml");
