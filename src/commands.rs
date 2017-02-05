use clap::{Arg, ArgMatches, App, SubCommand};
use empd;
use serde_json;
use std::path::PathBuf;
use types::*;
use xdg::BaseDirectories;

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

pub struct ListCommand;

impl Command for ListCommand {
    fn build_subcommand<'a, 'b>(&self) -> App<'a, 'b> {
        SubCommand::with_name("list").about("List all available playlists")
    }

    fn name(&self) -> &str{
        "list"
    }

    fn run(&self, _: ArgMatches) -> () {
        for file in get_playlist_filenames() {
            println!("{}", file.display());
        }
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

    fn run(&self, _: ArgMatches) -> () {
        let res = empd::current_playlist().
            and_then(|playlist| serde_json::to_string_pretty(&playlist).map_err(|e| From::from(e))).
            and_then(|s| Ok(println!("{}", s))
            );
        println!("{:?}", res)
    }
}
    }
}
