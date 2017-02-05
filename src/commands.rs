use clap::{Arg, ArgMatches, App, SubCommand};
use empd;
use serde_json;
use std::convert::From;
use std::path::PathBuf;
use types::*;
use xdg::BaseDirectories;

pub fn get_playlist_dir() -> Result<PathBuf, EdaboError> {
    BaseDirectories::with_prefix("edabo").
        map_err(From::from).
        and_then(|dirs| dirs.place_data_file("playlists").map_err(From::from))
}

fn get_playlist_filenames() -> Result<Vec<PathBuf>, EdaboError> {
    get_playlist_dir().
        and_then(|dir| dir.read_dir().map_err(From::from)).
        and_then(|files|
                 files.map(|file|
                           file.map(|f|
                                    f.path()).
                           map_err(From::from)
                 ).collect())
}

pub struct ListCommand;

impl Command for ListCommand {
    fn build_subcommand<'a, 'b>(&self) -> App<'a, 'b> {
        SubCommand::with_name("list").about("List all available playlists")
    }

    fn name(&self) -> &str{
        "list"
    }

    fn run(&self, _: &ArgMatches) -> Result<(), EdaboError>{
        get_playlist_filenames().
            and_then(|filenames| {
                for filename in filenames {
                    println!("{}", filename.display())
                };
                Ok(())
            })
    }
}

pub struct PrintCommand;

impl Command for PrintCommand {
    fn build_subcommand<'a, 'b>(&self) -> App<'a, 'b> {
        SubCommand::with_name(self.name()).about("Print the current playlist as JSON")
    }

    fn name(&self) -> &str{
        "print"
    }

    fn run(&self, _: &ArgMatches) -> Result<(), EdaboError> {
        empd::current_playlist().
            and_then(|playlist| serde_json::to_string_pretty(&playlist).map_err(From::from)).
            and_then(|s| Ok(println!("{}", s))
            )
    }
}

pub struct AddCommand;

impl Command for AddCommand {
    fn build_subcommand<'a, 'b>(&self) -> App<'a, 'b> {
        SubCommand::with_name(self.name()).
            about("Add something to a playlist").
            arg(Arg::with_name("playlist").
                help("The name of the playlist").
                required(true)).
            arg(Arg::with_name("all").
                long("all").
                short("a").
                help("Add all tracks from the current playlist")
            )
    }

    fn name(&self) -> &str{
        "add"
    }

    fn run(&self, args: &ArgMatches) -> Result<(), EdaboError> {
        args.value_of("playlist").
            ok_or_else(|| EdaboError {
                kind: ErrorKind::ArgumentError,
                detail: Some(String::from("Playlist argument not given, although it is required"))
            }).
            and_then(|name| Playlist::from_name(name)).
            and_then(|mut playlist_to_modify|
                     match args.is_present("all") {
                         true => empd::current_playlist().
                             and_then(|pl| {
                                 for (uuid, track) in pl.tracklist {
                                     playlist_to_modify.tracklist.insert(uuid, track);
                                 }
                                 match playlist_to_modify.to_file() {
                                     Some(e) => Err(e),
                                     None => Ok(())
                                 }
                             }
                             ),
                         false => empd::current_track().
                             and_then(|track| {
                                 playlist_to_modify.tracklist.insert(track.recording_id, track);
                                 match playlist_to_modify.to_file() {
                                     Some(e) => Err(e),
                                     None => Ok(())
                                 }
                             })
                     }
            )
    }
}
