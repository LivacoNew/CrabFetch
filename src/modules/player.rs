use std::time::Duration;

use dbus::{arg, blocking::{stdintf::org_freedesktop_dbus::Properties, Connection, Proxy}};
use serde::Deserialize;

use crate::{config_manager::Configuration, formatter::CrabFetchColor, module::Module, util::is_flag_set_u32, ModuleError};

pub struct PlayerInfo {
    player: String,
    album: String,
    album_artists: Vec<String>,
    track: String,
    track_artists: Vec<String>,
    status: String,
}
#[derive(Deserialize)]
pub struct PlayerConfiguration {
    pub title: String,
    pub ignore: Vec<String>,
    pub format: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub separator: Option<String>,
}
impl Module for PlayerInfo {
    fn new() -> PlayerInfo {
        PlayerInfo {
            // No "unknowns" here as it could just be empty from what I can gleam from the docs
            player: String::new(),
            album: String::new(),
            album_artists: Vec::new(),
            track: String::new(),
            track_artists: Vec::new(),
            status: String::new(),
        }
    }

    fn style(&self, config: &Configuration) -> (String, String) {
        let title_color: &CrabFetchColor = config.player.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.player.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.player.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.player.separator.as_ref().unwrap_or(&config.separator);

        let title: String = self.replace_placeholders(&config.player.title, config);
        let value: String = self.replace_color_placeholders(&self.replace_placeholders(&config.player.format, config));

        Self::default_style(config, &title, title_color, title_bold, title_italic, separator, &value)
    }
    fn unknown_output(config: &Configuration) -> (String, String) {
        let title_color: &CrabFetchColor = config.player.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.player.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.player.title_italic.unwrap_or(config.title_italic);
        let separator: &str = config.player.separator.as_ref().unwrap_or(&config.separator);

        let title: String = config.player.title
            .replace("{track}", "Unknown")
            .replace("{album}", "Unknown")
            .replace("{album_artists}", "Unknown")
            .replace("{track_artists}", "Unknown")
            .replace("{player}", "Unknown")
            .replace("{status}", "Unknown");

        Self::default_style(config, &title, title_color, title_bold, title_italic, separator, "Unknown")
    }

    fn replace_placeholders(&self, text: &str, _: &Configuration) -> String {
        text.replace("{track}", &self.track)
            .replace("{album}", &self.album)
            .replace("{album_artists}", &self.album_artists.join(" "))
            .replace("{track_artists}", &self.track_artists.join(" "))
            .replace("{player}", &self.player)
            .replace("{status}", &self.status)
    }

    fn gen_info_flags(format: &str) -> u32 {
        let mut info_flags: u32 = 0;

        if format.contains("{track}") || format.contains("{album}") || format.contains("{track_artists}") || format.contains("{album_artists}") {
            info_flags |= PLAYER_INFOFLAG_METADATA;
        }
        if format.contains("{player}") {
            info_flags |= PLAYER_INFOFLAG_PLAYER;
        }
        if format.contains("{status}") {
            info_flags |= PLAYER_INFOFLAG_STATUS;
        }

        info_flags
    }
}

const PLAYER_INFOFLAG_METADATA: u32 = 1;
const PLAYER_INFOFLAG_PLAYER: u32 = 2;
const PLAYER_INFOFLAG_STATUS: u32 = 4;

pub fn get_players(config: &Configuration) -> Result<Vec<PlayerInfo>, ModuleError> {
    let mut players: Vec<PlayerInfo> = Vec::new();
    // title is tagged onto the end here to account for the title placeholders
    let info_flags: u32 = PlayerInfo::gen_info_flags(&format!("{}{}", config.player.format, config.player.title));

    let conn: Connection = match Connection::new_session() {
        Ok(r) => r,
        Err(e) => return Err(ModuleError::new("Player", format!("Unable to connect to DBus: {}", e)))
    };

    let found_players: Vec<String> = match detect_current_players(&conn) {
        Some(r) => r,
        None => return Err(ModuleError::new("Player", "Unable to find any players".to_string())),
    };

    for player in found_players {
        let name: String = player.split('.').last().unwrap().to_string();
        if config.player.ignore.contains(&name) {
            continue // ignored
        }

        let proxy: Proxy<'_, &Connection> = conn.with_proxy(&player, "/org/mpris/MediaPlayer2", Duration::from_secs(1));
    
        let player_metadata: Option<arg::PropMap> = if is_flag_set_u32(info_flags, PLAYER_INFOFLAG_METADATA) {
            match req_player_property(&proxy, "Metadata") {
                Ok(r) => Some(r),
                Err(e) => return Err(ModuleError::new("Player", format!("Unable to fetch metadata for player: {}", e)))
            }
        } else {
            None
        };

        // mess
        let info: PlayerInfo = PlayerInfo {
            player: if is_flag_set_u32(info_flags, PLAYER_INFOFLAG_PLAYER) {
                match req_player_identity(&proxy) {
                    Ok(r) => r.to_string(),
                    Err(_) => "Unknown".to_string(),
                }
            } else {"Unknown".to_string()},
            album: if is_flag_set_u32(info_flags, PLAYER_INFOFLAG_METADATA) && player_metadata.is_some() {
                match arg::prop_cast::<String>(player_metadata.as_ref().unwrap(), "xesam:album") {
                    Some(r) => r.to_string(),
                    None => "Unknown".to_string(),
                }
            } else {"Unknown".to_string()},
            track: if is_flag_set_u32(info_flags, PLAYER_INFOFLAG_METADATA) && player_metadata.is_some() {
                match arg::prop_cast::<String>(player_metadata.as_ref().unwrap(), "xesam:title") {
                    Some(r) => r.to_string(),
                    None => "Unknown".to_string(),
                }
            } else {"Unknown".to_string()},
            track_artists: if is_flag_set_u32(info_flags, PLAYER_INFOFLAG_METADATA) && player_metadata.is_some() {
                match arg::prop_cast::<Vec<String>>(player_metadata.as_ref().unwrap(), "xesam:artist") {
                    Some(r) => r.to_vec(),
                    None => vec!["Unknown".to_string()],
                }
            } else {vec!["Unknown".to_string()]},
            album_artists: if is_flag_set_u32(info_flags, PLAYER_INFOFLAG_METADATA) && player_metadata.is_some() {
                match arg::prop_cast::<Vec<String>>(player_metadata.as_ref().unwrap(), "xesam:albumArtist") {
                    Some(r) => r.to_vec(),
                    None => vec!["Unknown".to_string()],
                }
            } else {vec!["Unknown".to_string()]},
            status: if is_flag_set_u32(info_flags, PLAYER_INFOFLAG_STATUS) {
                match req_player_property::<String>(&proxy, "PlaybackStatus") {
                    Ok(r) => r,
                    Err(_) => "Unknown".to_string(),
                }
            } else {"Unknown".to_string()},
        };
        players.push(info);
    }

    Ok(players)
}

fn detect_current_players(conn: &Connection) -> Option<Vec<String>>{
    let proxy: Proxy<'_, &Connection> = conn.with_proxy("org.freedesktop.DBus", "/", Duration::from_secs(1));
    let (names,): (Vec<String>,) = proxy.method_call("org.freedesktop.DBus", "ListNames", ()).expect("a");

    let mut players: Vec<String> = Vec::new();
    for name in names {
        if !name.starts_with("org.mpris.MediaPlayer2") {
            continue
        }

        players.push(name);
    }

    if players.is_empty() {
        // shit
        return None;
    }

    Some(players)
}

// I will be perfectly honest I have no idea what this generic type does, rust's compiler made it
// as a suggestion and it worked
fn req_player_property<T: for<'b> dbus::arg::Get<'b> + 'static>(player_proxy: &Proxy<'_, &Connection>, property: &str) -> Result<T, String> {
    match player_proxy.get("org.mpris.MediaPlayer2.Player", property) {
        Ok(r) => Ok(r),
        Err(e) => Err(e.to_string()),
    }
}
// It's in a different method
fn req_player_identity(player_proxy: &Proxy<'_, &Connection>) -> Result<String, String> {
    match player_proxy.get("org.mpris.MediaPlayer2", "Identity") {
        Ok(r) => Ok(r),
        Err(e) => Err(e.to_string()),
    }
}
