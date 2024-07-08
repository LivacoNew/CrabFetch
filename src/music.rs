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
        let mut title_color: &CrabFetchColor = &config.title_color;
        if (&config.music.title_color).is_some() {
            title_color = &config.music.title_color.as_ref().unwrap();
        }

        let mut title_bold: bool = config.title_bold;
        if config.music.title_bold.is_some() {
            title_bold = config.music.title_bold.unwrap();
        }
        let mut title_italic: bool = config.title_italic;
        if config.music.title_italic.is_some() {
            title_italic = config.music.title_italic.unwrap();
        }

        let mut seperator: &str = config.seperator.as_str();
        if config.music.seperator.is_some() {
            seperator = config.music.seperator.as_ref().unwrap();
        }

        let mut value: String = self.replace_placeholders(config);
        value = self.replace_color_placeholders(&value);

        Self::default_style(config, max_title_size, &config.music.title, title_color, title_bold, title_italic, &seperator, &value)
    }
    fn unknown_output(config: &Configuration, max_title_size: u64) -> String { 
        let mut title_color: &CrabFetchColor = &config.title_color;
        if (config.music.title_color).is_some() {
            title_color = config.music.title_color.as_ref().unwrap();
        }

        let mut title_bold: bool = config.title_bold;
        if config.music.title_bold.is_some() {
            title_bold = config.music.title_bold.unwrap();
        }
        let mut title_italic: bool = config.title_italic;
        if config.music.title_italic.is_some() {
            title_italic = config.music.title_italic.unwrap();
        }

        let mut seperator: &str = config.seperator.as_str();
        if config.music.seperator.is_some() {
            seperator = config.music.seperator.as_ref().unwrap();
        }

        Self::default_style(config, max_title_size, &config.music.title, title_color, title_bold, title_italic, &seperator, "Unknown")
    }

    fn replace_placeholders(&self, config: &Configuration) -> String {
        let mut album_artists: String = String::new();
        self.album_artists
            .iter()
            .for_each(|x| album_artists.push_str(x));

        let mut track_artists: String = String::new();
        self.track_artists
            .iter()
            .for_each(|x| track_artists.push_str(x));

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
    if player_dest.ends_with(".") {
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
