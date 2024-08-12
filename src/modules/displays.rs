use core::str;
use std::{collections::HashMap, env, fs::{self, read_dir, ReadDir}};

use serde::Deserialize;
use wayland_client::{protocol::{wl_output::{self, Transform}, wl_registry}, ConnectError, Connection, Dispatch, QueueHandle, WEnum};
use x11rb::{connection::RequestConnection, protocol::{randr::{self, MonitorInfo}, xproto::{self, ConnectionExt, CreateWindowAux, Screen, Window, WindowClass}}, COPY_DEPTH_FROM_PARENT};

use crate::{formatter::CrabFetchColor, config_manager::Configuration, module::Module, ModuleError};

#[derive(Clone)]
pub struct DisplayInfo {
    name: String,
    make: String,
    model: String,
    width: u16,
    height: u16,
    scale: i32,
    refresh_rate: Option<u16>,
    // Tempararily holds the transform for wayland while the rest of the data comes in
    // Should never be accessed otherwise
    wl_transform: Option<WEnum<Transform>>
}
impl DisplayInfo {
    fn wl_calc_transform(&mut self) {
        // ONLY FOR WAYLAND!!!!!
        // Translate it and reset the transform
        match self.wl_transform.unwrap() {
            WEnum::Value(transform) => {
                match transform {
                    Transform::_90 => {
                        // Swap width/height
                        let (width, height) = (self.width, self.height);
                        self.width = height;
                        self.height = width;
                    },
                    Transform::_270 => {
                        // Swap width/height
                        let (width, height) = (self.width, self.height);
                        self.width = height;
                        self.height = width;
                    },
                    Transform::Flipped90 => {
                        // Swap width/height
                        let (width, height) = (self.width, self.height);
                        self.width = height;
                        self.height = width;
                    },
                    Transform::Flipped270 => {
                        // Swap width/height
                        let (width, height) = (self.width, self.height);
                        self.width = height;
                        self.height = width;
                    },
                    _ => {}
                }
            },
            WEnum::Unknown(_) => {}, // ? no idea what to do here
        }
        // So if another event comes in it doesn't try to parse again
        self.wl_transform = None; 
    }
    fn scale_resolution(&mut self) {
        self.width /= self.scale as u16;
        self.height /= self.scale as u16;
    }
}

#[derive(Deserialize)]
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
            refresh_rate: None,
            wl_transform: None
        }
    }

    fn style(&self, config: &Configuration, max_title_size: u64) -> String {
        let title_color: &CrabFetchColor = config.displays.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.displays.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.displays.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.displays.separator.as_ref().unwrap_or(&config.separator);

        let title: String = config.displays.title.clone().replace("{name}", &self.name)
            .replace("{make}", &self.make)
            .replace("{model}", &self.model);
        let value: String = self.replace_color_placeholders(&self.replace_placeholders(config));

        Self::default_style(config, max_title_size, &title, title_color, title_bold, title_italic, separator, &value)
    }
    fn unknown_output(config: &Configuration, max_title_size: u64) -> String { 
        let title_color: &CrabFetchColor = config.displays.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.displays.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.displays.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.displays.separator.as_ref().unwrap_or(&config.separator);

        let title: String = config.displays.title.clone().replace("{name}", "Unknown")
            .replace("{make}", "Unknown")
            .replace("{model}", "Unknown");

        Self::default_style(config, max_title_size, &title, title_color, title_bold, title_italic, separator, "Unknown")
    }

    fn replace_placeholders(&self, config: &Configuration) -> String {
        let refresh_rate: String = match self.refresh_rate {
            Some(r) => r.to_string(),
            None => "N/A".to_string(),
        };
        config.displays.format.replace("{name}", &self.name)
            .replace("{make}", &self.make)
            .replace("{model}", &self.model)
            .replace("{width}", &self.width.to_string())
            .replace("{height}", &self.height.to_string())
            .replace("{refresh_rate}", &refresh_rate)
    }

    fn gen_info_flags(&self, config: &Configuration) -> u32 {
        todo!()
    }
}
impl DisplayInfo {
    // Used by calc_max_title_length
    pub fn get_title_size(&self, config: &Configuration) -> u64 {
        config.displays.title.clone()
            .replace("{name}", &self.name)
            .replace("{make}", &self.make)
            .replace("{model}", &self.model)
            .chars().count() as u64
    }
}

pub fn get_displays(config: &Configuration) -> Result<Vec<DisplayInfo>, ModuleError> {
    // Good news, during my college final deadline hell over the past 2 months, I learned how to
    // use a display server connection!
    
    // Instead of relying on XDG_SESSION_TYPE line Desktop, I simply just check the sockets as it
    // can report any string and break if someone's dumb enough to do that
    if env::var("WAYLAND_DISPLAY").is_ok() {
        fetch_wayland(config)
    } else if env::var("DISPLAY").is_ok() {
        fetch_xorg()
    } else {
        Err(ModuleError::new("Display", "Could not identify desktop session type.".to_string()))
    }
}


fn fetch_xorg() -> Result<Vec<DisplayInfo>, ModuleError> {
    // This has really opened my eyes as to why more pieces of software haven't swapped over to
    // Wayland yet, it's so much more convoluted at times compared to X11
    let (conn, screen_num) = match x11rb::connect(None) {
        Ok(r) => r,
        Err(e) => return Err(ModuleError::new("Display", format!("Can't connect to X11 server: {}", e))),
    };

    let screen: &Screen = &x11rb::connection::Connection::setup(&conn).roots[screen_num];
    let win_id: Window = match x11rb::connection::Connection::generate_id(&conn) {
        Ok(r) => r,
        Err(e) => return Err(ModuleError::new("Display", format!("Can't create new X11 identifier: {}", e))),
    };
    match conn.create_window(COPY_DEPTH_FROM_PARENT, win_id, screen.root,
        0, 0,
        1, 1,
        0,
        WindowClass::INPUT_OUTPUT, 0, &CreateWindowAux::new()) 
    {
        Ok(_) => {},
        Err(e) => return Err(ModuleError::new("Display", format!("Can't create new X11 window: {}", e))),
    };

    if conn.extension_information(randr::X11_EXTENSION_NAME).is_err() {
        return Err(ModuleError::new("Display", "X11 compositor doesn't have required 'randr' extension.".to_string()));
    }

    let monitors: Vec<MonitorInfo> = match randr::get_monitors(&conn, win_id, true) {
        Ok(r) => match r.reply() {
            Ok(r) => r.monitors,
            Err(e) => return Err(ModuleError::new("Display", format!("Failed to get monitors from randr: {}", e))),
        },
        Err(e) => return Err(ModuleError::new("Display", format!("Failed to get monitors from randr: {}", e))),
    };
    let mut displays: Vec<DisplayInfo> = Vec::new();
    for monitor in monitors {
        // Get the DRM name 
        let drm_name: String = match xproto::get_atom_name(&conn, monitor.name) {
            Ok(r) => match r.reply() {
                Ok(r) => String::from_utf8(r.name).unwrap(),
                Err(e) => return Err(ModuleError::new("Display", format!("Failed to get atomic name for monitor {}: {}", monitor.name, e))),
            },
            Err(e) => return Err(ModuleError::new("Display", format!("Failed to get atomic name for monitor {}: {}", monitor.name, e))),
        };
        // Find the make/model from the EDID
        let (make, model): (String, String) = match get_edid_makemodel(&drm_name) {
            Ok(r) => r,
            Err(e) => return Err(ModuleError::new("Display", format!("Failed to get make/model for monitor {}: {}", monitor.name, e))),
        };

        let display = DisplayInfo {
            name: drm_name,
            make,
            model,
            width: monitor.width,
            height: monitor.height,
            scale: 1,
            refresh_rate: None, // Can't get on X11, or at least if you can I don't know how
            wl_transform: None
        };

        displays.push(display);
    }

    // Not error checked as it's not a huge problem if this fails, moreso just a "would be nice to do" thing
    let _ = conn.destroy_window(win_id);
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
        Err(e) => return Err(format!("Unable to open /sys/class/drm: {}", e)),
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
        let edid_bytes: Vec<u8> = match fs::read(format!("/sys/class/drm/{}/edid", dir_name)) {
            Ok(r) => r,
            Err(e) => return Err(format!("Unable to open /sys/class/drm/{}/edid: {}", dir_name, e)),
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
        
        let manuid: u16 = ((edid_bytes[8] as u16) << 8) | (edid_bytes[9] as u16);
        // + 64 to convert em to uppercase ascii
        let char1: char = (((manuid & 0b011111_00000000) >> 10) as u8 + 64) as char;
        let char2: char = (((manuid & 0b00000011_11100000) >> 5) as u8 + 64) as char;
        let char3: char = ((manuid & 0b00000000_00011111) as u8 + 64) as char;
        make = format!("{char1}{char2}{char3}");

        // Now to scower the display descriptors
        // Byte 48 is where this starts
        let mut starting_byte: usize = 54;
        for _ in 0..3 {
            let is_display: u16 = ((edid_bytes[starting_byte] as u16) << 8) | edid_bytes[starting_byte + 1] as u16;
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
            model = format!("{}{:X}", make, edid_bytes[10] as u16 | (edid_bytes[11] as u16) << 8);
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
            display.wl_transform = Some(*transform);
        }
        if let wl_output::Event::Mode { width, height, refresh, .. } = &event {
            display.width = width.to_string().parse::<u16>().unwrap_or(0);
            display.height = height.to_string().parse::<u16>().unwrap_or(0);
            display.refresh_rate = match (*refresh as f32 / 1000.0).round().to_string().parse::<u16>() {
                Ok(r) => Some(r),
                // There's no real "error handling" here so just set it to 0 so it's not None and letting us get stuck in an infinite loop
                // Clearly your compositor is very very very dumb
                Err(_) => Some(0)
            };
        }

        if !display.name.is_empty() && display.width != 0 && display.height != 0 && display.refresh_rate.is_some() && display.wl_transform.is_some() {
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
fn fetch_wayland(config: &Configuration) -> Result<Vec<DisplayInfo>, ModuleError> {
    let conn: Connection = match Connection::connect_to_env() {
        Ok(r) => r,
        Err(e) => {
            let msg: &str = match e {
                ConnectError::NoWaylandLib => "Unable to load the Wayland library.",
                ConnectError::NoCompositor => "Unable to find a Wayland compositor.",
                ConnectError::InvalidFd => "Found a Wayland compositor, but the socket contained garbage."
            };
            return Err(ModuleError::new("Display", format!("Failed to connect to Wayland compositor: {}", msg)));
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
            Err(e) => return Err(ModuleError::new("Display", format!("Compositor roundtrip returned error: {}", e)))
        };
        loops += 1;
        if loops > 1000 {
            return Err(ModuleError::new("Display", "Wayland compositor took too long to respond; over 1000 event loops have passed.".to_string()));
        }
    }

    let mut displays: Vec<DisplayInfo> = data.outputs.into_iter()
        .map(|x| x.1.clone())
        .collect();

    displays.iter_mut().for_each(|x| {
        x.wl_calc_transform();
        if config.displays.scale_size {
            x.scale_resolution();
        }
        
        (x.make, x.model) = match get_edid_makemodel(&x.name) {
            Ok(r) => r,
            Err(_) => return, // We're in a closure, can't return an error
        };
    });

    displays.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(displays)
}
