use std::time::Duration;

use dbus::{arg, blocking::{stdintf::org_freedesktop_dbus::Properties, Connection, Proxy}};
use serde::Deserialize;

use crate::{config_manager::Configuration, formatter::CrabFetchColor, Module, ModuleError};

pub struct MusicInfo {
    album: String,
    album_artists: Vec<String>,
    track: String,
    track_artists: Vec<String>,
}
#[derive(Deserialize)]
pub struct MusicConfiguration {
    pub title: String,
    pub title_color: Option<CrabFetchColor>,
    pub title_bold: Option<bool>,
    pub title_italic: Option<bool>,
    pub seperator: Option<String>,
    pub format: String,
    pub player: String,
}
impl Module for MusicInfo {
    fn new() -> MusicInfo {
        MusicInfo {
            album: String::new(),
            album_artists: Vec::new(),
            track: String::new(),
            track_artists: Vec::new(),
        }
    }

    fn style(&self, config: &Configuration, max_title_size: u64) -> String {
        let title_color: &CrabFetchColor = config.music.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.music.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.music.title_italic.unwrap_or(config.title_italic);
        let seperator: &str = config.music.seperator.as_ref().unwrap_or(&config.seperator);

        let value: String = self.replace_color_placeholders(&self.replace_placeholders(config));

        Self::default_style(config, max_title_size, &config.music.title, title_color, title_bold, title_italic, seperator, &value)
    }
    fn unknown_output(config: &Configuration, max_title_size: u64) -> String { 
        let title_color: &CrabFetchColor = config.music.title_color.as_ref().unwrap_or(&config.title_color);
        let title_bold: bool = config.music.title_bold.unwrap_or(config.title_bold);
        let title_italic: bool = config.music.title_italic.unwrap_or(config.title_italic);
        let seperator: &str = config.music.seperator.as_ref().unwrap_or(&config.seperator);

        Self::default_style(config, max_title_size, &config.music.title, title_color, title_bold, title_italic, seperator, "Unknown")
    }

    fn replace_placeholders(&self, config: &Configuration) -> String {
        let album_artists: String = self.album_artists.join(" ");
        let track_artists: String = self.track_artists.join(" ");

        config.music.format.replace("{track}", &self.track)
            .replace("{album}", &self.album)
            .replace("{album_artists}", &album_artists)
            .replace("{track_artists}", &track_artists)
    }
}

pub fn get_music(player: &str) -> Result<MusicInfo, ModuleError> {
    let mut music: MusicInfo = MusicInfo::new();

    let conn: Connection = match Connection::new_session() {
        Ok(r) => r,
        Err(e) => return Err(ModuleError::new("Music", format!("Unable to connect to DBus: {}", e)))
    };
    let player_dest: String = format!("org.mpris.MediaPlayer2.{}", player.to_lowercase());
    if player_dest.ends_with('.') {
        // Leaving the player string empty can cause a crash without this check
        return Err(ModuleError::new("Music", "You have specified your player in an invalid way, tried to use a invalid DBus destination.".to_string()))
    }
    let proxy: Proxy<'_, &Connection> = conn.with_proxy(&player_dest, "/org/mpris/MediaPlayer2", Duration::from_secs(1));
    
    let player_metadata: arg::PropMap = match proxy.get("org.mpris.MediaPlayer2.Player", "Metadata") {
        Ok(r) => r,
        Err(e) => {
            if e.to_string() == "The name is not activatable" {
                // Likelyhood is they've placed in a player that can't be found
                return Err(ModuleError::new("Music", "You have specified a player that doesn't exist, isn't active, or doesn't implement MPRIS.".to_string()))
            }
            return Err(ModuleError::new("Music", format!("Unable to request player property: {}", e)))
        },
    };

    music.album = match arg::prop_cast::<String>(&player_metadata, "xesam:album") {
        Some(r) => r.to_string(),
        None => "Unknown".to_string(),
    };
    music.track = match arg::prop_cast::<String>(&player_metadata, "xesam:title") {
        Some(r) => r.to_string(),
        None => "Unknown".to_string(),
    };
    music.track_artists = match arg::prop_cast::<Vec<String>>(&player_metadata, "xesam:artist") {
        Some(r) => r.to_vec(),
        None => vec!("Unknown".to_string()),
    };
    music.album_artists = match arg::prop_cast::<Vec<String>>(&player_metadata, "xesam:albumArtist") {
        Some(r) => r.to_vec(),
        None => vec!("Unknown".to_string()),
    };

    Ok(music)
}
