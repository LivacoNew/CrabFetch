use core::str;
use std::{collections::HashMap, env, fs::{self, read_dir, ReadDir}};

#[cfg(feature = "jsonschema")]
use schemars::JsonSchema;
use serde::Deserialize;
use wayland_client::{protocol::{wl_output::{self, Transform}, wl_registry}, ConnectError, Connection, Dispatch, QueueHandle, WEnum};
use x11rb::{connection::RequestConnection, protocol::{randr::{self, ConnectionExt, GetCrtcInfoReply, GetOutputInfoReply, GetScreenResourcesCurrentReply, ModeInfo, MonitorInfo, Rotation}, xproto::{self, Screen}}};

use crate::{config_manager::Configuration, formatter::CrabFetchColor, module::Module, util::{self, is_flag_set_u32}, ModuleError};

#[derive(Clone)]
pub struct DisplayInfo {
    name: String,
    make: String,
    model: String,
    width: u16,
    height: u16,
    scale: i32,
    refresh_rate: u16,
    rotation: u16,
}
impl DisplayInfo {
    fn calc_rotation(&mut self) {
        if self.rotation % 180 != 0 {
            // Swap width/height
            (self.width, self.height) = (self.height, self.width);
        }
    }
    fn scale_resolution(&mut self) {
        self.width /= u16::try_from(self.scale).expect("Cannot convert scale to u16");
        self.height /= u16::try_from(self.scale).expect("Cannot convert scale to u16");
    }
}

#[derive(Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(JsonSchema))]
pub struct DisplayConfiguration {
    pub title: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub separator: Option<String>,
    pub format: String,
    pub scale_size: bool,
}
impl Module for DisplayInfo {
    fn new() -> DisplayInfo {
        DisplayInfo {
            name: "Unknown".to_string(),
            make: "Unknown".to_string(),
            model: "Unknown".to_string(),
            width: 0,
            height: 0,
            scale: 0,
            refresh_rate: 0,
            rotation: 0
        }
    }

    fn style(&self, config: &Configuration) -> (String, String) {
        let title_color: &CrabFetchColor = config.displays.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.displays.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.displays.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.displays.separator.as_ref().unwrap_or(&config.separator);

        let title: String = self.replace_placeholders(&config.displays.title, config);
        let value: String = self.replace_color_placeholders(&self.replace_placeholders(&config.displays.format, config), config);

        Self::default_style(config, &title, title_color, title_bold, title_italic, separator, &value)
    }
    fn unknown_output(config: &Configuration) -> (String, String) {
        let title_color: &CrabFetchColor = config.displays.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.displays.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.displays.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.displays.separator.as_ref().unwrap_or(&config.separator);

        let title: String = config.displays.title
            .replace("{name}", "Unknown")
            .replace("{make}", "Unknown")
            .replace("{model}", "Unknown")
            .replace("{width}", "Unknown")
            .replace("{height}", "Unknown")
            .replace("{refresh_rate}", "Unknown");

        Self::default_style(config, &title, title_color, title_bold, title_italic, separator, "Unknown")
    }

    fn replace_placeholders(&self, text: &str, _: &Configuration) -> String {
        text.replace("{name}", &self.name)
            .replace("{make}", &self.make)
            .replace("{model}", &self.model)
            .replace("{width}", &self.width.to_string())
            .replace("{height}", &self.height.to_string())
            .replace("{refresh_rate}", &self.refresh_rate.to_string())
    }

    fn gen_info_flags(format: &str) -> u32 {
        let mut info_flags: u32 = 0;

        if format.contains("{name}") {
            info_flags |= DISPLAYS_INFOFLAG_DRM_NAME;
        }
        if format.contains("{make}") {
            info_flags |= DISPLAYS_INFOFLAG_MAKE;
            info_flags |= DISPLAYS_INFOFLAG_DRM_NAME; // DRM name is required for EDID
        }
        if format.contains("{model}") {
            info_flags |= DISPLAYS_INFOFLAG_MODEL;
            info_flags |= DISPLAYS_INFOFLAG_DRM_NAME; // DRM name is required for EDID
        }
        if format.contains("{width}") {
            info_flags |= DISPLAYS_INFOFLAG_WIDTH;
        }
        if format.contains("{height}") {
            info_flags |= DISPLAYS_INFOFLAG_HEIGHT;
        }
        if format.contains("{refresh_rate}") {
            info_flags |= DISPLAYS_INFOFLAG_REFRESH_RATE;
        }

        info_flags
    }
}

const DISPLAYS_INFOFLAG_DRM_NAME: u32 = 1;
const DISPLAYS_INFOFLAG_MAKE: u32 = 2;
const DISPLAYS_INFOFLAG_MODEL: u32 = 4;
const DISPLAYS_INFOFLAG_WIDTH: u32 = 8;
const DISPLAYS_INFOFLAG_HEIGHT: u32 = 16;
const DISPLAYS_INFOFLAG_REFRESH_RATE: u32 = 32;

pub fn get_displays(config: &Configuration) -> Result<Vec<DisplayInfo>, ModuleError> {
    // title is tagged onto the end here to account for the title placeholders
    let info_flags: u32 = DisplayInfo::gen_info_flags(&format!("{}{}", config.displays.format, config.displays.title));

    // Good news, during my college final deadline hell over the past 2 months, I learned how to
    // use a display server connection!

    // Instead of relying on XDG_SESSION_TYPE line Desktop, I simply just check the sockets as it
    // can report any string and break if someone's dumb enough to do that
    if env::var("WAYLAND_DISPLAY").is_ok() {
        fetch_wayland(config, info_flags)
    } else if env::var("DISPLAY").is_ok() {
        fetch_xorg(info_flags)
    } else {
        Err(ModuleError::new("Display", "Could not identify desktop session type.".to_string()))
    }
}


fn fetch_xorg(info_flags: u32) -> Result<Vec<DisplayInfo>, ModuleError> {
    // This has really opened my eyes as to why more pieces of software haven't swapped over to
    // Wayland yet, it's so much more convoluted at times compared to X11
    let (conn, screen_num) = match x11rb::connect(None) {
        Ok(r) => r,
        Err(e) => return Err(ModuleError::new("Display", format!("Can't connect to X11 server: {e}"))),
    };

    let screen: &Screen = &x11rb::connection::Connection::setup(&conn).roots[screen_num];

    if conn.extension_information(randr::X11_EXTENSION_NAME).is_err() {
        return Err(ModuleError::new("Display", "X11 compositor doesn't have required 'randr' extension.".to_string()));
    }

    let monitors: Vec<MonitorInfo> = match randr::get_monitors(&conn, screen.root, true) {
        Ok(r) => match r.reply() {
            Ok(r) => r.monitors,
            Err(e) => return Err(ModuleError::new("Display", format!("Failed to get monitors from randr: {e}"))),
        },
        Err(e) => return Err(ModuleError::new("Display", format!("Failed to get monitors from randr: {e}"))),
    };
    let mut displays: Vec<DisplayInfo> = Vec::new();
    for monitor in monitors {
        // Get the DRM name
        let mut drm_name: String = "Unknown".to_string();
        if is_flag_set_u32(info_flags, DISPLAYS_INFOFLAG_DRM_NAME) {
            drm_name = match xproto::get_atom_name(&conn, monitor.name) {
                Ok(r) => match r.reply() {
                    Ok(r) => String::from_utf8(r.name).unwrap(),
                    Err(e) => return Err(ModuleError::new("Display", format!("Failed to get atomic name for monitor {}: {e}", monitor.name))),
                },
                Err(e) => return Err(ModuleError::new("Display", format!("Failed to get atomic name for monitor {}: {e}", monitor.name))),
            };
        }
        // Find the make/model from the EDID
        let (mut make, mut model): (String, String) = ("Unknown".to_string(), "Unknown".to_string());
        if is_flag_set_u32(info_flags, DISPLAYS_INFOFLAG_MAKE) || is_flag_set_u32(info_flags, DISPLAYS_INFOFLAG_MODEL) {
            (make, model) = match get_edid_makemodel(&drm_name) {
                Ok(r) => r,
                Err(e) => return Err(ModuleError::new("Display", format!("Failed to get make/model for monitor {}: {e}", monitor.name))),
            };
        }

        // Find the screen rotation
        let resources: GetScreenResourcesCurrentReply = match conn.randr_get_screen_resources_current(screen.root) {
            Ok(r) => match r.reply() {
                Ok(r) => r,
                Err(e) => return Err(ModuleError::new("Display", format!("Failed to get screen resources: {e}"))),
            },
            Err(e) => return Err(ModuleError::new("Display", format!("Failed to get screen resources: {e}"))),
        };

        let output: u32 = match monitor.outputs.first() {
            Some(r) => *r,
            None => return Err(ModuleError::new("Display", format!("Monitor {} has no outputs", monitor.name))),
        };

        let output_info: GetOutputInfoReply = match conn.randr_get_output_info(output, resources.config_timestamp) {
            Ok(r) => match r.reply() {
                Ok(r) => r,
                Err(e) => return Err(ModuleError::new("Display", format!("Failed to get output info: {e}"))),
            }
            Err(e) => return Err(ModuleError::new("Display", format!("Failed to get output info: {e}"))),
        };

        let crtc: GetCrtcInfoReply = match conn.randr_get_crtc_info(output_info.crtc, resources.config_timestamp) {
            Ok(r) => match r.reply() {
                Ok(r) => r,
                Err(e) => return Err(ModuleError::new("Display", format!("Failed to get crtc info: {e}"))),
            }
            Err(e) => return Err(ModuleError::new("Display", format!("Failed to get crtc info: {e}"))),
        };

        // And finally
        let mode: &ModeInfo = match resources.modes.iter().find(|x| x.id == crtc.mode) {
            Some(r) => r,
            None => return Err(ModuleError::new("Display", format!("Failed to find mode {}", crtc.mode))),
        };

        let mut display = DisplayInfo {
            name: drm_name,
            make,
            model,
            width: mode.width,
            height: mode.height,
            scale: 1,
            refresh_rate: u16::try_from(mode.dot_clock / (u32::from(mode.htotal) * u32::from(mode.vtotal))).unwrap_or(0),
            rotation: match crtc.rotation & 0b111 {
                Rotation::ROTATE90 => 90,
                Rotation::ROTATE180 => 180,
                Rotation::ROTATE270 => 270,
                _ => 0,
            }
        };
        display.calc_rotation();
        displays.push(display);
    }

    displays.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(displays)
}

fn get_edid_makemodel(drm_name: &str) -> Result<(String, String), String> {
    // Relative to /sys/class/drm
    // Scans the dir until it finds the first directory ending in that drm name
    // This is because we don't know the GPU device index, and from my (limited) knowledge, no 2
    // DRM names should be repeated. If they can, I'll need to revisit this func.
    let dir: ReadDir = match read_dir("/sys/class/drm") {
        Ok(r) => r,
        Err(e) => return Err(format!("Unable to open /sys/class/drm: {e}")),
    };

    let mut make: String = "Unknown".to_string();
    let mut model: String = "Unknown".to_string();
    for x in dir {
        if x.is_err() {
            continue;
        }
        let x = x.unwrap();
        let dir_name = x.file_name();
        let dir_name = dir_name.to_str().unwrap();
        if !dir_name.ends_with(drm_name) {
            // Strip the ending -{id} - Can happen with X11 seemingly
            if !dir_name[..dir_name.len() - 2].ends_with(drm_name) {
                continue;
            }
        }

        // Found it
        // Get the EDID now
        let edid_bytes: Vec<u8> = match fs::read(format!("/sys/class/drm/{dir_name}/edid")) {
            Ok(r) => r,
            Err(e) => return Err(format!("Unable to open /sys/class/drm/{dir_name}/edid: {e}")),
        };
        if edid_bytes.is_empty() {
            continue; // Can happen with VM's, ignore it
        }

        // Thanks to these wonderful sources;
        // - https://glenwing.github.io/docs/VESA-EEDID-A2.pdf
        // - https://github.com/tuomas56/edid-rs/tree/master?tab=readme-ov-file
        //
        // From what I can tell, manufacturer ID is at byte 8+2
        // Display model name itself is somewhere buried within a display descriptor, which I have
        // to go through and find

        let manuid: u16 = (u16::from(edid_bytes[8]) << 8) | u16::from(edid_bytes[9]);
        // + 64 to convert em to uppercase ascii
        let char1: char = (((manuid & 0b011111_00000000) >> 10) as u8 + 64) as char;
        let char2: char = (((manuid & 0b00000011_11100000) >> 5) as u8 + 64) as char;
        let char3: char = ((manuid & 0b00000000_00011111) as u8 + 64) as char;
        make = format!("{char1}{char2}{char3}");

        // Now to scower the display descriptors
        // Byte 48 is where this starts
        let mut starting_byte: usize = 54;
        for _ in 0..3 {
            let is_display: u16 = (u16::from(edid_bytes[starting_byte]) << 8) | u16::from(edid_bytes[starting_byte + 1]);
            if is_display != 0 {
                starting_byte += 18;
                continue;
            }

            // Check the tag
            let tag: u8 = edid_bytes[starting_byte + 3];
            if tag != 252 {
                starting_byte += 18;
                continue;
            }

            model = String::new();
            // Read from byte 5+13 to find the full name
            for byte in edid_bytes.iter().take(starting_byte + 13).skip(starting_byte + 5) {
                model.push(*byte as char);
            }
            model = model.trim().to_string();

            break;
        }

        if model == "Unknown" {
            // Now we go for the ID Product Code as a final grasp
            // This appends the manufacturer on the front as this seems to be the common strategy
            // for these, tested by my laptop as well as well as this issue's laptop screen
            // https://github.com/LivacoNew/CrabFetch/issues/21
            model = format!("{}{:X}", make, u16::from(edid_bytes[10]) | u16::from(edid_bytes[11]) << 8);
        }

        break;
    }

    Ok((make, model))
}


//
// The Wayland Zone
//
struct WaylandState {
    complete: bool, // Tells us whether to break out of the event loop or not
    num_outputs: u8, // How many outputs are waiting to be processed
    outputs: HashMap<wl_output::WlOutput, DisplayInfo> // The output data as it stands
}
impl Dispatch<wl_registry::WlRegistry, ()> for WaylandState {
    fn event(state: &mut Self, reg: &wl_registry::WlRegistry, event: wl_registry::Event, _: &(), _: &Connection, qh: &QueueHandle<WaylandState>,) {
        if let wl_registry::Event::Global {name, interface, version} = event {
            if interface == "wl_output" {
                // This is what we're looking for, bind to it
                reg.bind::<wl_output::WlOutput, _, _>(name, version, qh, ());
                state.num_outputs += 1;
            }
        }
    }
}
impl Dispatch<wl_output::WlOutput, ()> for WaylandState {
    fn event(state: &mut Self, output: &wl_output::WlOutput, event: wl_output::Event, _: &(), _: &Connection, _qh: &QueueHandle<WaylandState>,) {
        if !state.outputs.contains_key(output) {
            state.outputs.insert(output.clone(), DisplayInfo::new());
        }

        let display: &mut DisplayInfo = state.outputs.get_mut(output).unwrap();
        if let wl_output::Event::Name {name} = &event {
            display.name = name.to_string();
        }
        if let wl_output::Event::Scale {factor} = &event {
            display.scale = *factor;
        }
        if let wl_output::Event::Geometry {transform, ..} = &event {
            display.rotation = match *transform {
                WEnum::Value(v) => match v {
                    Transform::_90 | Transform::Flipped90 => 90,
                    Transform::_180 | Transform::Flipped180 => 180,
                    Transform::_270 | Transform::Flipped270=> 270,
                    _ => 0,
                },
                WEnum::Unknown(_) => 0
            };
        }
        #[allow(clippy::cast_precision_loss)]
        if let wl_output::Event::Mode { width, height, refresh, .. } = &event {
            display.width = width.to_string().parse::<u16>().unwrap_or(0);
            display.height = height.to_string().parse::<u16>().unwrap_or(0);
            display.refresh_rate = (*refresh as f32 / 1000.0).round().to_string().parse::<u16>().unwrap_or(0);
        }

        if !display.name.is_empty() && display.width != 0 && display.height != 0 && display.refresh_rate != 0 {
            // We're done, release it
            output.release();
            if state.num_outputs == 0 {
                state.complete = true;
                return;
            }
            state.num_outputs -= 1;
        }
    }
}
// NOTE: Wayland will ignore info flags, as all the events have to be passed through *regardless*
// It will only use them for make/model with EDID, nothing else
fn fetch_wayland(config: &Configuration, info_flags: u32) -> Result<Vec<DisplayInfo>, ModuleError> {
    let conn: Connection = match Connection::connect_to_env() {
        Ok(r) => r,
        Err(e) => {
            let msg: &str = match e {
                ConnectError::NoWaylandLib => "Unable to load the Wayland library.",
                ConnectError::NoCompositor => "Unable to find a Wayland compositor.",
                ConnectError::InvalidFd => "Found a Wayland compositor, but the socket contained garbage."
            };
            return Err(ModuleError::new("Display", format!("Failed to connect to Wayland compositor: {msg}")));
        },
    };
    let display = conn.display();

    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();

    let _registry = display.get_registry(&qh, ());
    let mut data: WaylandState = WaylandState {
        complete: false,
        num_outputs: 0,
        outputs: HashMap::new()
    };

    let mut loops: u16 = 0;
    while !data.complete {
        match event_queue.roundtrip(&mut data) {
            Ok(r) => r,
            Err(e) => return Err(ModuleError::new("Display", format!("Compositor roundtrip returned error: {e}")))
        };
        loops += 1;
        if loops > 1000 {
            return Err(ModuleError::new("Display", "Wayland compositor took too long to respond; over 1000 event loops have passed.".to_string()));
        }
    }

    let mut displays: Vec<DisplayInfo> = data.outputs.into_iter()
        .map(|x| x.1.clone())
        .collect();

    for x in &mut displays {
        x.calc_rotation();
        if config.displays.scale_size {
            x.scale_resolution();
        }

        if is_flag_set_u32(info_flags, DISPLAYS_INFOFLAG_MAKE) || is_flag_set_u32(info_flags, DISPLAYS_INFOFLAG_MODEL) {
            if util::in_wsl() {
                // WSL has no EDID and the compositor would just return weston's weird virtual display thing
                x.make = "N/A".to_string();
                x.model = "N/A".to_string();
            } else {
                (x.make, x.model) = match get_edid_makemodel(&x.name) {
                    Ok(r) => r,
                    Err(e) => return Err(ModuleError::new("Display", format!("Cannot parse EDID: {e}")))
                };
            }
        }
        if is_flag_set_u32(info_flags, DISPLAYS_INFOFLAG_DRM_NAME) && util::in_wsl() {
            // WSL also doesn't have any DRM name
            x.name = "N/A".to_string();
        }
    }

    displays.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(displays)
}
