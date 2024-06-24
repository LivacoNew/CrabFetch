use core::str;
use std::{collections::HashMap, env};

use serde::Deserialize;
use wayland_client::{protocol::{wl_output, wl_registry}, ConnectError, Connection, Dispatch, QueueHandle};
use x11rb::{connection::RequestConnection, protocol::{randr::{self, MonitorInfo}, xproto::{ConnectionExt, CreateWindowAux, Screen, Window, WindowClass}}, COPY_DEPTH_FROM_PARENT};

use crate::{config_manager::{Configuration, CrabFetchColor}, Module, ModuleError};

#[derive(Clone)]
pub struct DisplayInfo {
    name: String,
    width: u16,
    height: u16,
    refresh_rate: Option<u16>
}
#[derive(Deserialize)]
pub struct DisplayConfiguration {
    pub title: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub seperator: Option<String>,
    pub format: String,
}
impl Module for DisplayInfo {
    fn new() -> DisplayInfo {
        DisplayInfo {
            name: "".to_string(),
            width: 0,
            height: 0,
            refresh_rate: None
        }
    }

    fn style(&self, config: &Configuration, max_title_size: u64) -> String {
        let mut title_color: &CrabFetchColor = &config.title_color;
        if (&config.displays.title_color).is_some() {
            title_color = config.displays.title_color.as_ref().unwrap();
        }

        let mut title_bold: bool = config.title_bold;
        if config.displays.title_bold.is_some() {
            title_bold = config.displays.title_bold.unwrap();
        }
        let mut title_italic: bool = config.title_italic;
        if config.displays.title_italic.is_some() {
            title_italic = config.displays.title_italic.unwrap();
        }

        let mut seperator: &str = config.seperator.as_str();
        if config.displays.seperator.is_some() {
            seperator = config.displays.seperator.as_ref().unwrap();
        }

        let mut title: String = config.displays.title.clone();
        title = title.replace("{name}", &self.name);

        self.default_style(config, max_title_size, &title, title_color, title_bold, title_italic, &seperator)
    }

    fn replace_placeholders(&self, config: &Configuration) -> String {
        let refresh_rate: String = match self.refresh_rate {
            Some(r) => r.to_string(),
            None => "N/A".to_string(),
        };
        config.displays.format.replace("{name}", &self.name)
            .replace("{width}", &self.width.to_string())
            .replace("{height}", &self.height.to_string())
            .replace("{refresh_rate}", &refresh_rate)
    }
}

pub fn get_displays() -> Result<Vec<DisplayInfo>, ModuleError> {
    // Good news, during my college final deadline hell over the past 2 months, I learned how to
    // use a display server connection!
    
    // Instead of relying on XDG_SESSION_TYPE line Desktop, I simply just check the sockets as it
    // can report any string and break if someone's dumb enough to do that
    if env::var("WAYLAND_DISPLAY").is_ok() {
        return fetch_wayland();
    } else if env::var("DISPLAY").is_ok() {
        return fetch_xorg();
    } else {
        return Err(ModuleError::new("Display", format!("Could not identify desktop session type.")));
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

    if !conn.extension_information(randr::X11_EXTENSION_NAME).is_ok() {
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
        let display = DisplayInfo {
            name: monitor.name.to_string(),
            width: monitor.width,
            height: monitor.height,
            refresh_rate: None, // Can't get on X11, or at least if you can I don't know how
        };
        displays.push(display);
    }

    // Not error checked as it's not a huge problem if this fails, moreso just a "would be nice to do" thing
    let _ = conn.destroy_window(win_id);
    Ok(displays)
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
                reg.bind::<wl_output::WlOutput, _, _>(name, version, &qh, ());
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
        if let wl_output::Event::Mode { width, height, refresh, .. } = &event {
            display.width = width.to_string().parse::<u16>().unwrap();
            display.height = height.to_string().parse::<u16>().unwrap();
            display.refresh_rate = match (*refresh as f32 / 1000.0).round().to_string().parse::<u16>() {
                Ok(r) => Some(r),
                // There's no real "error handling" here so just set it to 0 so it's not None and letting us get stuck in an infinite loop
                Err(_) => Some(0)
            };
        }

        if !display.name.is_empty() && display.width != 0 && display.height != 0 && display.refresh_rate.is_some() {
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
fn fetch_wayland() -> Result<Vec<DisplayInfo>, ModuleError> {
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
            Err(e) => {
                return Err(ModuleError::new("Display", format!("Compositor roundtrip returned error: {}", e)));
            }
        };
        loops += 1;
        if loops > 1000 {
            return Err(ModuleError::new("Display", "Wayland compositor took too long to respond; over 1000 event loops have passed.".to_string()));
        }
    }

    Ok(data.outputs.into_iter().map(|x| x.1.clone()).collect())
}
