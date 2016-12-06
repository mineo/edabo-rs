extern crate clap;
extern crate edabo;
extern crate mpd;
extern crate serde;
extern crate serde_json;
extern crate xdg;

use clap::{App, SubCommand};
use mpd::Client;
use std::path::PathBuf;
use xdg::BaseDirectories;
use edabo::types::*;

fn get_playlist_dir() -> PathBuf {
    let xdg_dirs = BaseDirectories::with_prefix("edabo").unwrap();
    xdg_dirs.find_data_file("playlists").unwrap()
}

fn get_playlist_filenames() -> Vec<PathBuf> {
    let playlist_dir = get_playlist_dir();
    let entries = playlist_dir.read_dir().unwrap();
    // TODO: the size of filenames should be the same as the size of entries
    let mut filenames: Vec<PathBuf> = vec![];
    for entry in entries {
        filenames.push(entry.unwrap().path())
    }
    filenames
}

fn print_playlist_filenames() -> () {
    for file in get_playlist_filenames() {
        println!("{}", file.display());
    }
}

fn print_playlist_json() -> () {
    let mut conn = Client::connect("127.0.0.1:6600").unwrap();
    let playqueue = conn.queue().unwrap();
    let mut tracks = Vec::new();
    for song in &playqueue {
        let track = Track::from_song(song);
        if let Ok(val) = track {
            tracks.push(val);
        }
    }
    let playlist = Playlist::new("Current".to_string(),
                                 Some("The current playlist".to_string()),
                                 tracks);
    println!("{}", serde_json::to_string_pretty(&playlist).unwrap());
}

fn make_clap_parser<'a, 'b>() -> App<'a, 'b> {
    App::new("Edabo")
        .version("1.0")
        .author("Wieland Hoffmann")
        .subcommand(SubCommand::with_name("list").about("List all available playlists"))
        .subcommand(SubCommand::with_name("listplaylist").about("Print the playlist, JSON-style!"))
}

fn main() {
    let matches = make_clap_parser().get_matches();
    if let Some(_) = matches.subcommand_matches("list") {
        print_playlist_filenames();
    }
    if let Some(_) = matches.subcommand_matches("listplaylist") {
        print_playlist_json()
    }
}
